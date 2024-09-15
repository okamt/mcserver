use bytes::Buf;

use crate::*;

/// An item in a `Tape`.
#[repr(C)]
pub struct TapeItem {
    /// `LLLLLLLLTT##NNNN` where:
    ///
    /// - If is list:
    ///     - `L` is the list len (`i32`)
    ///     - `T` is the list tag (`u8`)
    /// - If is list item:
    ///     - `T` is `0xFF`
    /// - `#` is the Tag (`u8`)
    /// - `N` is the name length (`u16`)
    list_data_tag_and_name_len: u64,
    /// The position in the NBT source.
    source_pos: u64,
    /// Extra data, depends on the Tag.
    data: u64,
}

impl TapeItem {
    #[inline]
    pub fn new(
        tag: Tag,
        name_len: u16,
        source_pos: usize,
        data: u64,
        is_list_item: bool,
        list_data: Option<(Tag, i32)>,
    ) -> Self {
        let (list_tag, list_len) = list_data
            .map(|ld| (if is_list_item { 0xFF } else { ld.0.to_u8() }, ld.1))
            .unwrap_or((0, 0));

        Self {
            list_data_tag_and_name_len: ((list_len as u64) << 32)
                | ((list_tag as u64) << 24)
                | ((tag.to_u8() as u64) << 16)
                | u64::from(name_len),
            source_pos: source_pos as u64,
            data,
        }
    }

    #[inline]
    pub fn is_list_item(&self) -> bool {
        ((self.list_data_tag_and_name_len >> 24) as u8) == 0xFF
    }

    #[inline]
    pub fn get_list_len(&self) -> i32 {
        (self.list_data_tag_and_name_len >> 32) as i32
    }

    #[inline]
    pub fn get_list_tag(&self) -> Tag {
        let mut value = (self.list_data_tag_and_name_len >> 24) as u8;
        if value == 0xFF {
            value = 0;
        }
        // SAFETY: `self.list_data_tag_and_name_len` is private, `new` accepts `Tag`,
        // and the `0xFF` case is checked, so it will always be valid.
        unsafe { Tag::from_u8_unchecked(value) }
    }

    #[inline]
    pub fn get_tag(&self) -> Tag {
        // SAFETY: `self.list_data_tag_and_name_len` is private, `new` and `set_tag` both accept `Tag` so it will always be valid.
        unsafe { Tag::from_u8_unchecked((self.list_data_tag_and_name_len >> 16) as u8) }
    }

    #[inline]
    pub fn get_name_len(&self) -> u16 {
        self.list_data_tag_and_name_len as u16
    }

    #[inline]
    pub fn get_source_pos(&self) -> usize {
        self.source_pos as usize
    }

    #[inline]
    pub fn get_source_payload_pos(&self) -> usize {
        self.get_source_pos()
            + match self.get_tag() {
                Tag::End => 1,
                _ => {
                    if self.is_list_item() {
                        0
                    } else {
                        3 + self.get_name_len() as usize
                    }
                }
            }
    }

    #[inline]
    pub fn get_data(&self) -> u64 {
        self.data
    }

    #[inline]
    pub fn set_data(&mut self, data: u64) {
        self.data = data;
    }
}

impl Debug for TapeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TapeItem(tag={:?}, name_len={}, source_pos={}, data={}, list_len={}, list_tag={:?})",
            self.get_tag(),
            self.get_name_len(),
            self.get_source_pos(),
            self.get_data(),
            self.get_list_len(),
            self.get_list_tag()
        )
    }
}

/// NBT list data compressed into a `u64`. `##000000LLLLLLLL` where `#` is the list tag and `L` is the list length.
/// This is not what is actually stored in `TapeItem.data`, instead it's used for the parsing stack.
#[repr(transparent)]
#[derive(Clone, Copy)]
struct ListData(u64);

impl ListData {
    #[inline]
    fn new(tag: Tag, len: i32) -> Self {
        debug_assert!(len > 0);
        Self((u64::from(tag.to_u8()) << 56) | u64::from(len as u32))
    }

    #[inline]
    fn get_tag(&self) -> Tag {
        // SAFETY: `self.0` is private and `new` accepts `Tag`, so it will always be valid.
        unsafe { Tag::from_u8_unchecked((self.0 >> 56) as u8) }
    }

    #[inline]
    fn get_len(&self) -> u32 {
        self.0 as u32
    }

    #[inline]
    fn set_len(&mut self, len: u32) {
        self.0 = (self.0 & 0xFFFFFFFF00000000) | (u64::from(len))
    }
}

impl Into<u64> for ListData {
    #[inline]
    fn into(self) -> u64 {
        // SAFETY: Transmuting into the representation type is safe with #[repr(transparent)].
        // https://doc.rust-lang.org/nomicon/other-reprs.html#reprtransparent
        unsafe { std::mem::transmute(self) }
    }
}

impl From<u64> for ListData {
    #[inline]
    fn from(value: u64) -> Self {
        // SAFETY: Transmuting from the representation type is safe with #[repr(transparent)].
        // https://doc.rust-lang.org/nomicon/other-reprs.html#reprtransparent
        unsafe { std::mem::transmute(value) }
    }
}

/// A flat representation of NBT data for easier traversal and better spatial locality.
/// This is an intermediate format, end users should look at `Nbt`.
///
/// Contains indices to the underlying NBT data source, so must be always used with the same source.
///
/// For more information on the "tape" idea:
///
/// - <https://simdjson.org/api/0.4.0/md_doc_tape.html>
/// - <https://nickb.dev/blog/parsing-performance-improvement-with-tapes-and-spacial-locality/>
pub struct Tape(pub(crate) Vec<TapeItem>);

impl<Idx> Index<Idx> for Tape
where
    Vec<TapeItem>: Index<Idx>,
{
    type Output = <Vec<TapeItem> as Index<Idx>>::Output;

    #[inline]
    fn index(&self, index: Idx) -> &Self::Output {
        <Vec<TapeItem> as Index<Idx>>::index(&self.0, index)
    }
}

impl Tape {
    pub fn parse(source: &[u8], is_network_nbt: bool) -> Result<Self, NbtParseError> {
        /// Represents an open compound/list scope.
        struct StackItem {
            /// The position in the tape of the compound tag or the list tag (the start of the scope).
            /// Once the parser reaches the end of the scope, it writes the position of the end tag (if list scope, adds a fake end tag)
            /// to `tape[start_tag_tape_pos].data`.
            start_tag_tape_pos: usize,
            /// If list scope, the ListData of the list item.
            list_data: Option<ListData>,
        }

        // The `&[u8]` is modified by the `Buf` trait functions (we're passing a &mut &[u8]).
        // For example, calling `source.get_u8` will move the pointer one byte forward and decrease the len by 1.
        let mut source = source;
        // Arbitrary capacities to avoid reallocation.
        let mut tape = Vec::<TapeItem>::with_capacity(128);
        // Compound/list scope stack.
        let mut stack = Vec::<StackItem>::with_capacity(128);

        let full_size = source.remaining();
        let pos = |source: &[u8]| full_size - source.remaining();

        while source.has_remaining() {
            let tag_pos = pos(source);
            let mut tag_tape_pos = tape.len();

            let tag;
            let name_len;
            let is_list_item;

            let mut stack_top = stack.last_mut();

            if let Some(StackItem {
                start_tag_tape_pos,
                list_data: Some(list_data),
            }) = stack_top
            {
                let list_len = list_data.get_len();
                if list_len == 0 {
                    // List is finished, add fake end tag and link it with start tag.
                    tape.push(TapeItem::new(
                        Tag::End,
                        0,
                        tag_pos,
                        *start_tag_tape_pos as u64,
                        true,
                        None,
                    ));
                    tape[*start_tag_tape_pos].set_data(tag_tape_pos as u64);
                    tag_tape_pos += 1;
                    stack.pop();
                    stack_top = stack.last_mut();
                } else {
                    list_data.set_len(list_len - 1);
                }
            }

            match stack_top {
                Some(StackItem {
                    list_data: Some(list_data),
                    ..
                }) => {
                    // If we are in a list scope, the tag is implied and there is no name or name len.
                    // Parsing starts directly at payload (data).
                    tag = list_data.get_tag();
                    name_len = 0;
                    is_list_item = true;
                }
                _ => {
                    tag =
                        source
                            .get_u8()
                            .try_into()
                            .map_err(|value| NbtParseError::InvalidTag {
                                value,
                                pos: tag_pos,
                            })?;
                    name_len = match tag {
                        Tag::End => 0,
                        // The root compound tag in Network NBT has no name or name len.
                        _ if is_network_nbt && tag_tape_pos == 0 => 0,
                        _ => {
                            let name_len = source.get_u16();
                            source.advance(name_len as usize);
                            name_len
                        }
                    };
                    is_list_item = false;
                }
            }

            if tag_tape_pos == 0 {
                if tag != Tag::Compound {
                    return Err(NbtParseError::WrongStartingTag {
                        tag,
                        expected: Tag::Compound,
                    });
                }

                tape.push(TapeItem::new(Tag::Compound, name_len, 0, 0, false, None));
                stack.push(StackItem {
                    start_tag_tape_pos: 0,
                    list_data: None,
                });

                continue;
            }

            let mut list_data: Option<(Tag, i32)> = None;

            let data: u64 = match tag {
                Tag::End => {
                    let StackItem {
                        start_tag_tape_pos: compound_tag_tape_pos,
                        ..
                    } = match stack.pop() {
                        Some(value) => value,
                        None => return Err(NbtParseError::UnexpectedEnd { pos: pos(source) }),
                    };
                    tape[compound_tag_tape_pos].set_data(tag_tape_pos as u64);
                    compound_tag_tape_pos as u64
                }
                Tag::Byte => source.get_u8().into(),
                Tag::Short => source.get_u16().into(),
                Tag::Int => source.get_u32().into(),
                Tag::Long => source.get_u64(),
                Tag::Float => f32::to_bits(source.get_f32()).into(),
                Tag::Double => f64::to_bits(source.get_f64()),
                Tag::ByteArray => {
                    let len = source.get_u32();
                    source.advance(len as usize);
                    len.into()
                }
                Tag::String => {
                    let len = source.get_u16();
                    source.advance(len as usize);
                    len.into()
                }
                Tag::List => {
                    let tag = source.get_u8();
                    let len = source.get_i32();
                    let list_tag: Tag =
                        tag.try_into().map_err(|value| NbtParseError::InvalidTag {
                            value,
                            pos: tag_pos,
                        })?;
                    list_data = Some((list_tag, len));
                    let list_data = ListData::new(list_tag, len);

                    if len > 0 {
                        match list_tag {
                            // List tag may be end tag if len == 0.
                            Tag::End => {
                                return Err(NbtParseError::InvalidListType { tag: list_tag })
                            }
                            _ => {}
                        }

                        stack.push(StackItem {
                            start_tag_tape_pos: tag_tape_pos,
                            list_data: Some(list_data),
                        });
                    }

                    // Data will be filled by end tag later.
                    0
                }
                Tag::Compound => {
                    stack.push(StackItem {
                        start_tag_tape_pos: tag_tape_pos,
                        list_data: None,
                    });

                    // Data will be filled by end tag later.
                    0
                }
                Tag::IntArray => {
                    let len = source.get_u32();
                    source.advance(4 * len as usize);
                    len.into()
                }
                Tag::LongArray => {
                    let len = source.get_u32();
                    source.advance(8 * len as usize);
                    len.into()
                }
            };

            tape.push(TapeItem::new(
                tag,
                name_len,
                tag_pos,
                data,
                is_list_item,
                list_data,
            ));
        }

        // All the compound/list scopes must be appropriately closed.
        if !stack.is_empty() {
            return Err(NbtParseError::SuddenEnd);
        }

        Ok(Self(tape))
    }
}
