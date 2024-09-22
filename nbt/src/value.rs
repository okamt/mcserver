//! NBT value representations and related traits.

use super::*;

/// A small representation of an NBT value. Integers and floats are stored directly, while other bigger types are stored as references (indices).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NbtValue {
    End,
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
pub trait NbtRef: NbtValueRepr {
    type Output<'source, 'nbt>
    where
        'source: 'nbt;

    /// Gets the position of this value in the `Tape` (internal use).
    fn tape_pos(&self) -> usize;

    /// Parses the NBT value, or retrieves it from the cache.
    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'source, 'nbt>
    where
        'source: 'nbt;
}

/// An NBT associative array (Compound, List, ByteArray, IntArray, LongArray)
pub trait NbtMapRef: NbtRef {
    type Key: ?Sized; // We only use a reference to Key, so being unsized/DST is OK.
    type Value: NbtValueRepr;

    /// Gets the [`Value`](Self::Value) associated to this [`Key`](Self::Key).
    fn get(
        &self,
        nbt: &NbtParser<'_>,
        key: impl Borrow<Self::Key>,
    ) -> Option<NbtNodeRef<Self::Value>>;
}

impl NbtRef for NbtByteArray {
    type Output<'source, 'nbt> = &'nbt [i8] where 'source: 'nbt;

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'source, 'nbt>
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
    type Value = i8;
    type Key = usize;

    fn get(
        &self,
        nbt: &NbtParser<'_>,
        index: impl Borrow<usize>,
    ) -> Option<NbtNodeRef<Self::Value>> {
        let index = *index.borrow();
        let tape_item = &nbt.tape()[self.0];

        if index >= tape_item.get_data() as usize {
            None
        } else {
            Some(NbtNodeRef {
                tape_pos: None,
                value: nbt.source()[tape_item.get_source_payload_pos() + index] as i8,
            })
        }
    }
}

impl NbtRef for NbtString {
    type Output<'source, 'nbt> = &'nbt str where 'source: 'nbt;

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'source, 'nbt>
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
    type Output<'source, 'nbt> = &'nbt [NbtNodeRef<NbtValue>] where 'source: 'nbt;

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'source, 'nbt>
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
                        value,
                        next_tape_pos,
                    } = inner_nbt.parse_at(tape_pos);
                    match value {
                        NbtValue::End => break,
                        value => {
                            vec.push(NbtNodeRef {
                                tape_pos: Some(tape_pos),
                                value,
                            });
                        }
                    }
                    tape_pos = next_tape_pos;
                }

                vec
            })
        })
    }
}

impl NbtMapRef for NbtList {
    type Value = NbtValue;
    type Key = usize;

    fn get(
        &self,
        nbt: &NbtParser<'_>,
        index: impl Borrow<usize>,
    ) -> Option<NbtNodeRef<Self::Value>> {
        let index = *index.borrow();
        let tape_item = &nbt.tape()[self.0];
        let list_len = tape_item.get_list_len() as usize;

        if index >= list_len {
            None
        } else {
            Some(NbtNodeRef {
                tape_pos: Some(self.0 + index),
                value: nbt.parse_at(self.0 + index).value,
            })
        }
    }
}

impl NbtRef for NbtCompound {
    type Output<'source, 'nbt> = &'nbt HashMap<Cow<'nbt, str>, NbtNodeRef<NbtValue>> where 'source: 'nbt;

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'source, 'nbt>
    where
        'source: 'nbt,
    {
        nbt.with_cache(|inner_nbt, cache| {
            cache.compounds.insert(self.0, |_| {
                let mut map = HashMap::new();

                let mut tape_pos = self.0 + 1;
                loop {
                    let NbtParseResult {
                        value,
                        next_tape_pos,
                    } = inner_nbt.parse_at(tape_pos);
                    match value {
                        NbtValue::End => break,
                        value => {
                            map.insert(
                                inner_nbt.get_name_at(tape_pos),
                                NbtNodeRef {
                                    tape_pos: Some(tape_pos),
                                    value,
                                },
                            );
                        }
                    }
                    tape_pos = next_tape_pos;
                }

                Box::new(map)
            })
        })
    }
}

impl NbtMapRef for NbtCompound {
    type Value = NbtValue;
    type Key = str;

    /// Gets the first entry in this compound with name `name`.
    fn get(&self, nbt: &NbtParser<'_>, name: impl Borrow<str>) -> Option<NbtNodeRef<Self::Value>> {
        let name = name.borrow();
        self.parse(nbt).get(name).copied()
    }
}

impl NbtRef for NbtIntArray {
    type Output<'source, 'nbt> = &'nbt [i32] where 'source: 'nbt;

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'source, 'nbt>
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
    type Value = i32;
    type Key = usize;

    fn get(
        &self,
        nbt: &NbtParser<'_>,
        index: impl Borrow<usize>,
    ) -> Option<NbtNodeRef<Self::Value>> {
        let index = *index.borrow();
        let tape_item = &nbt.tape()[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();

        if index >= tape_item.get_data() as usize {
            None
        } else {
            Some(NbtNodeRef {
                tape_pos: None,
                value: i32::from_be_bytes(
                    (&nbt.source()[source_start_pos + index..source_start_pos + index + 4])
                        .try_into()
                        .unwrap(),
                ),
            })
        }
    }
}

impl NbtRef for NbtLongArray {
    type Output<'source, 'nbt> = &'nbt [i64] where 'source: 'nbt;

    fn tape_pos(&self) -> usize {
        self.0
    }

    fn parse<'source, 'nbt>(&self, nbt: &'nbt NbtParser<'source>) -> Self::Output<'source, 'nbt>
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
    type Value = i64;
    type Key = usize;

    fn get(
        &self,
        nbt: &NbtParser<'_>,
        index: impl Borrow<usize>,
    ) -> Option<NbtNodeRef<Self::Value>> {
        let index = *index.borrow();
        let tape_item = &nbt.tape()[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();

        if index >= tape_item.get_data() as usize {
            None
        } else {
            Some(NbtNodeRef {
                tape_pos: None,
                value: i64::from_be_bytes(
                    (&nbt.source()[source_start_pos + index..source_start_pos + index + 8])
                        .try_into()
                        .unwrap(),
                ),
            })
        }
    }
}
