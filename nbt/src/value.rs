//! NBT value representations and related traits.

use super::*;

/// A small representation of an NBT value. Integers and floats are stored directly, while other bigger types are stored as references (indices).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(NbtByteArray),
    String(NbtString),
    List(NbtList),
    Compound(NbtCompound),
    IntArray(NbtIntArray),
    LongArray(NbtLongArray),
}

/// Part of [`NbtValue`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtByteArray(pub(crate) usize);
/// Part of [`NbtValue`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtString(pub(crate) usize);
/// Part of [`NbtValue`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtList(pub(crate) usize);
/// Part of [`NbtValue`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtCompound(pub(crate) usize);
/// Part of [`NbtValue`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtIntArray(pub(crate) usize);
/// Part of [`NbtValue`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtLongArray(pub(crate) usize);

/// A reference to a big NBT value.
pub trait NbtRef: NbtValueOrRef {
    type Output<'nbt>;

    /// Gets the position of this value in the `Tape` (internal use).
    fn tape_pos(&self) -> usize;

    /// Parses the NBT value, or retrieves it from the cache.
    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'nbt>
    where
        'source: 'nbt;
}

/// An NBT associative array (Compound, List, ByteArray, IntArray, LongArray)
pub trait NbtMapRef: NbtRef {
    type Key: ?Sized; // We only use a reference to Key, so being unsized/DST is OK.
    type Value<'nbt>: NbtValueOutput;

    /// Gets the [`Value`](Self::Value) associated to this [`Key`](Self::Key).
    fn get<'source, 'nbt>(
        &self,
        nbt: &NbtParser<'source>,
        key: impl Borrow<Self::Key>,
    ) -> Option<Self::Value<'nbt>>
    where
        'source: 'nbt;
}

impl NbtRef for NbtByteArray {
    type Output<'nbt> = &'nbt [i8];

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'nbt>
    where
        'source: 'nbt,
    {
        let tape_item = &nbt.tape()[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        let array =
            &nbt.source()[source_start_pos..source_start_pos + (tape_item.get_data() as usize)];
        bytemuck::cast_slice(array)
    }
}

impl NbtMapRef for NbtByteArray {
    type Value<'nbt> = i8;
    type Key = usize;

    fn get<'source, 'nbt>(
        &self,
        nbt: &NbtParser<'source>,
        index: impl Borrow<usize>,
    ) -> Option<Self::Value<'nbt>>
    where
        'source: 'nbt,
    {
        let index = *index.borrow();
        let tape_item = &nbt.tape()[self.0];
        if index >= tape_item.get_data() as usize {
            None
        } else {
            Some(nbt.source()[tape_item.get_source_payload_pos() + index] as i8)
        }
    }
}

impl NbtRef for NbtString {
    type Output<'nbt> = &'nbt str;

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'nbt>
    where
        'source: 'nbt,
    {
        nbt.with_cache(|inner_nbt, cache| {
            cache.strings.insert(self.0, |_| {
                let tape_item = &inner_nbt.tape[self.0];
                let source_start_pos = tape_item.get_source_payload_pos();
                let len = tape_item.get_data() as usize;
                simd_cesu8::decode_lossy(
                    &inner_nbt.source[source_start_pos..source_start_pos + len],
                )
            })
        })
    }
}

impl NbtRef for NbtList {
    type Output<'nbt> = &'nbt [NbtValue];

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'nbt>
    where
        'source: 'nbt,
    {
        nbt.with_cache(|inner_nbt, cache| {
            cache.lists.insert(self.0, |_| {
                let tape_item = &inner_nbt.tape[self.0];
                let list_len = tape_item.get_list_len();

                let mut vec = Vec::with_capacity(list_len as usize);
                let mut tape_pos = self.0 + 1;
                loop {
                    let NbtParseResult {
                        item,
                        next_tape_pos,
                    } = inner_nbt.parse_at(tape_pos);
                    match item {
                        Some(item) => {
                            vec.push(item);
                        }
                        None => break,
                    }
                    tape_pos = next_tape_pos;
                }

                vec
            })
        })
    }
}

impl NbtMapRef for NbtList {
    type Value<'nbt> = NbtValue;
    type Key = usize;

    fn get<'source, 'nbt>(
        &self,
        nbt: &NbtParser<'source>,
        index: impl Borrow<usize>,
    ) -> Option<Self::Value<'nbt>>
    where
        'source: 'nbt,
    {
        let index = *index.borrow();
        let tape_item = &nbt.tape()[self.0];
        let list_len = tape_item.get_list_len() as usize;

        if index >= list_len {
            None
        } else {
            nbt.parse_at(self.0 + index).item
        }
    }
}

impl NbtRef for NbtCompound {
    type Output<'nbt> = &'nbt HashMap<Cow<'nbt, str>, NbtValue>;

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'nbt>
    where
        'source: 'nbt,
    {
        nbt.with_cache(|inner_nbt, cache| {
            cache.compounds.insert(self.0, |_| {
                let mut map = HashMap::new();

                let mut tape_pos = self.0 + 1;
                loop {
                    let NbtParseResult {
                        item,
                        next_tape_pos,
                    } = inner_nbt.parse_at(tape_pos);
                    match item {
                        Some(item) => {
                            map.insert(inner_nbt.get_name_at(tape_pos), item);
                        }
                        None => break,
                    }
                    tape_pos = next_tape_pos;
                }

                Box::new(map)
            })
        })
    }
}

impl NbtMapRef for NbtCompound {
    type Value<'nbt> = NbtValue;
    type Key = str;

    /// Gets the first entry in this compound with name `name`.
    fn get<'source, 'nbt>(
        &self,
        nbt: &NbtParser<'source>,
        name: impl Borrow<str>,
    ) -> Option<NbtValue>
    where
        'source: 'nbt,
    {
        let name = name.borrow();
        self.parse(nbt).get(name).copied()
    }
}

impl NbtRef for NbtIntArray {
    type Output<'nbt> = &'nbt [i32];

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'nbt>
    where
        'source: 'nbt,
    {
        let tape_item = &nbt.tape()[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        let array =
            &nbt.source()[source_start_pos..source_start_pos + (4 * tape_item.get_data() as usize)];
        bytemuck::cast_slice(array)
    }
}

impl NbtMapRef for NbtIntArray {
    type Value<'nbt> = i32;
    type Key = usize;

    fn get<'source, 'nbt>(
        &self,
        nbt: &NbtParser<'source>,
        index: impl Borrow<usize>,
    ) -> Option<Self::Value<'nbt>>
    where
        'source: 'nbt,
    {
        let index = *index.borrow();
        let tape_item = &nbt.tape()[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        if index >= tape_item.get_data() as usize {
            None
        } else {
            Some(i32::from_be_bytes(
                (&nbt.source()[source_start_pos + index..source_start_pos + index + 4])
                    .try_into()
                    .unwrap(),
            ))
        }
    }
}

impl NbtRef for NbtLongArray {
    type Output<'nbt> = &'nbt [i64];

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'nbt>
    where
        'source: 'nbt,
    {
        let tape_item = &nbt.tape()[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        let array =
            &nbt.source()[source_start_pos..source_start_pos + (8 * tape_item.get_data() as usize)];
        bytemuck::cast_slice(array)
    }
}

impl NbtMapRef for NbtLongArray {
    type Value<'nbt> = i64;
    type Key = usize;

    fn get<'source, 'nbt>(
        &self,
        nbt: &NbtParser<'source>,
        index: impl Borrow<usize>,
    ) -> Option<Self::Value<'nbt>>
    where
        'source: 'nbt,
    {
        let index = *index.borrow();
        let tape_item = &nbt.tape()[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        if index >= tape_item.get_data() as usize {
            None
        } else {
            Some(i64::from_be_bytes(
                (&nbt.source()[source_start_pos + index..source_start_pos + index + 8])
                    .try_into()
                    .unwrap(),
            ))
        }
    }
}
