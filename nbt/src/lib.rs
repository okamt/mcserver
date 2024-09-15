use std::{borrow::Cow, collections::HashMap, fmt::Debug, marker::PhantomData, ops::Index};

use crate::marker::*;
use crate::parse::*;
use crate::tag::*;
use private::Sealed;
use thiserror::Error;

mod private {
    pub trait Sealed {}
}

pub(crate) mod marker;
pub(crate) mod parse;
pub(crate) mod tag;

/// A reference to a big NBT value.
pub trait NbtRef: Sealed {
    type Output<'source>;

    /// Parses the NBT value.
    fn parse<'source>(&self, nbt: &Nbt<'source>) -> Self::Output<'source>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtByteArray(usize);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtString(usize);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtList(usize);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtCompound(usize);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtIntArray(usize);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NbtLongArray(usize);

impl Sealed for NbtByteArray {}
impl Sealed for NbtString {}
impl Sealed for NbtList {}
impl Sealed for NbtCompound {}
impl Sealed for NbtIntArray {}
impl Sealed for NbtLongArray {}

impl NbtRef for NbtByteArray {
    type Output<'source> = &'source [i8];

    fn parse<'source>(&self, nbt: &Nbt<'source>) -> Self::Output<'source> {
        let tape_item = &nbt.tape[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        let array =
            &nbt.source[source_start_pos..source_start_pos + (tape_item.get_data() as usize)];
        bytemuck::cast_slice(array)
    }
}

impl NbtByteArray {
    pub fn get(&self, nbt: &Nbt<'_>, index: usize) -> Option<i8> {
        let tape_item = &nbt.tape[self.0];
        if index >= tape_item.get_data() as usize {
            None
        } else {
            Some(nbt.source[tape_item.get_source_payload_pos() + index] as i8)
        }
    }
}

impl NbtRef for NbtString {
    type Output<'source> = Cow<'source, str>;

    fn parse<'source>(&self, nbt: &Nbt<'source>) -> Self::Output<'source> {
        let tape_item = &nbt.tape[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        let len = tape_item.get_data() as usize;
        simd_cesu8::decode_lossy(&nbt.source[source_start_pos..source_start_pos + len])
    }
}

impl NbtRef for NbtList {
    type Output<'source> = Vec<NbtItem>;

    fn parse<'source>(&self, nbt: &Nbt<'source>) -> Self::Output<'source> {
        let tape_item = &nbt.tape[self.0];
        let list_len = tape_item.get_list_len();

        let mut vec = Vec::with_capacity(list_len as usize);
        let mut tape_pos = self.0 + 1;
        loop {
            let NbtParseResult {
                item,
                next_tape_pos,
            } = nbt.parse_at(tape_pos);
            match item {
                Some(item) => {
                    vec.push(item);
                }
                None => break,
            }
            tape_pos = next_tape_pos;
        }

        vec
    }
}

impl NbtRef for NbtCompound {
    type Output<'source> = HashMap<Cow<'source, str>, NbtItem>;

    fn parse<'source>(&self, nbt: &Nbt<'source>) -> Self::Output<'source> {
        let mut map = HashMap::new();

        let mut tape_pos = self.0 + 1;
        loop {
            let NbtParseResult {
                item,
                next_tape_pos,
            } = nbt.parse_at(tape_pos);
            match item {
                Some(item) => {
                    map.insert(nbt.get_name_at(tape_pos), item);
                }
                None => break,
            }
            tape_pos = next_tape_pos;
        }

        map
    }
}

impl NbtCompound {
    /// Gets the first entry in this compound with name `name`.
    pub fn get<'source, S>(&self, nbt: &Nbt<'source>, name: S) -> Option<NbtItem>
    where
        S: AsRef<str>,
    {
        let key = name.as_ref();

        let mut tape_pos = self.0 + 1;
        loop {
            let NbtParseResult {
                item,
                next_tape_pos,
            } = nbt.parse_at(tape_pos);
            match item {
                Some(item) => {
                    if nbt.get_name_at(tape_pos) == key {
                        return Some(item);
                    }
                }
                None => return None,
            }
            tape_pos = next_tape_pos;
        }
    }
}

impl NbtRef for NbtIntArray {
    type Output<'source> = &'source [i32];

    fn parse<'source>(&self, nbt: &Nbt<'source>) -> Self::Output<'source> {
        let tape_item = &nbt.tape[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        let array =
            &nbt.source[source_start_pos..source_start_pos + (4 * tape_item.get_data() as usize)];
        bytemuck::cast_slice(array)
    }
}

impl NbtIntArray {
    pub fn get(&self, nbt: &Nbt<'_>, index: usize) -> Option<i32> {
        let tape_item = &nbt.tape[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        if index >= tape_item.get_data() as usize {
            None
        } else {
            Some(i32::from_ne_bytes(
                (&nbt.source[source_start_pos + index..source_start_pos + index + 4])
                    .try_into()
                    .unwrap(),
            ))
        }
    }
}

impl NbtRef for NbtLongArray {
    type Output<'source> = &'source [i64];

    fn parse<'source>(&self, nbt: &Nbt<'source>) -> Self::Output<'source> {
        let tape_item = &nbt.tape[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        let array =
            &nbt.source[source_start_pos..source_start_pos + (8 * tape_item.get_data() as usize)];
        bytemuck::cast_slice(array)
    }
}

impl NbtLongArray {
    pub fn get(&self, nbt: &Nbt<'_>, index: usize) -> Option<i64> {
        let tape_item = &nbt.tape[self.0];
        let source_start_pos = tape_item.get_source_payload_pos();
        if index >= tape_item.get_data() as usize {
            None
        } else {
            Some(i64::from_ne_bytes(
                (&nbt.source[source_start_pos + index..source_start_pos + index + 8])
                    .try_into()
                    .unwrap(),
            ))
        }
    }
}

// A small representation of an NBT item. Integers and floats are stored directly, while other bigger types are stored as references (indices).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NbtItem {
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

/// An NBT file representation, with a Compound as root tag.
pub struct Nbt<'source> {
    pub(crate) tape: Tape,
    pub(crate) source: &'source [u8],
}

impl<'source> Nbt<'source> {
    pub fn from(
        source: &'source [u8],
        is_network_nbt: bool,
    ) -> Result<Nbt<'source>, NbtParseError> {
        Ok(Self {
            tape: Tape::parse(source, is_network_nbt)?,
            source,
        })
    }

    /// Gets the root compound.
    pub fn root(&self) -> NbtCompound {
        NbtCompound(0)
    }

    /// Returns an `NbtCompoundIterator` over the root compound's children.
    pub fn iter(&self) -> NbtIterator<CompoundMarker> {
        NbtIterator::from_root(self)
    }

    /// Gets the name of the `tape_item`.
    pub(crate) fn get_name(&self, tape_item: &TapeItem) -> Cow<'source, str> {
        let name_len = tape_item.get_name_len();
        let source_pos = tape_item.get_source_pos() + 3;
        simd_cesu8::decode_lossy(&self.source[source_pos..source_pos + (name_len as usize)])
    }

    pub(crate) fn get_name_at(&self, tape_pos: usize) -> Cow<'source, str> {
        self.get_name(&self.tape[tape_pos])
    }

    pub(crate) fn parse_at(&self, tape_pos: usize) -> NbtParseResult {
        let tape_item = &self.tape[tape_pos];

        let item = (|| {
            Some(match tape_item.get_tag() {
                Tag::End => return None,
                Tag::Byte => NbtItem::Byte(tape_item.get_data() as i8),
                Tag::Short => NbtItem::Short(tape_item.get_data() as i16),
                Tag::Int => NbtItem::Int(tape_item.get_data() as i32),
                Tag::Long => NbtItem::Long(tape_item.get_data() as i64),
                Tag::Float => NbtItem::Float(f32::from_bits(tape_item.get_data() as u32)),
                Tag::Double => NbtItem::Double(f64::from_bits(tape_item.get_data())),
                Tag::ByteArray => NbtItem::ByteArray(NbtByteArray(tape_pos)),
                Tag::String => NbtItem::String(NbtString(tape_pos)),
                Tag::List => NbtItem::List(NbtList(tape_pos)),
                Tag::Compound => NbtItem::Compound(NbtCompound(tape_pos)),
                Tag::IntArray => NbtItem::IntArray(NbtIntArray(tape_pos)),
                Tag::LongArray => NbtItem::LongArray(NbtLongArray(tape_pos)),
            })
        })();
        let next_tape_pos = match item.as_ref() {
            Some(NbtItem::Compound(_) | NbtItem::List(_)) => 1 + tape_item.get_data() as usize,
            Some(_) => tape_pos + 1,
            None => 0,
        };

        NbtParseResult {
            item,
            next_tape_pos,
        }
    }
}

#[derive(Debug)]
pub struct NbtParseResult {
    pub item: Option<NbtItem>,
    pub next_tape_pos: usize,
}

pub struct NbtCursor<'source, 'nbt>
where
    'nbt: 'source,
{
    nbt: &'nbt Nbt<'source>,
    pub(crate) tape_pos: usize,
}

impl<'source, 'nbt> NbtCursor<'source, 'nbt> {
    #[inline]
    pub fn new(nbt: &'nbt Nbt<'source>) -> Self {
        Self { nbt, tape_pos: 0 }
    }

    /// Parses an `NbtItem` at the current tape position. If `lazy`, will not decode or parse compounds, lists or strings.
    pub fn parse_current(&mut self) -> Option<NbtItem> {
        self.nbt.parse_at(self.tape_pos).item
    }
}

impl<'source, 'nbt> IntoIterator for NbtCursor<'source, 'nbt> {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = NbtIterator<'source, CompoundMarker>;

    fn into_iter(self) -> Self::IntoIter {
        NbtIterator::from_root(&self.nbt)
    }
}

/// An iterator over an NBT compound or list. Uses the `Marker` typestate to provide different `Iterator` implementations.
pub struct NbtIterator<'source, Marker>
where
    Marker: IteratorMarker,
{
    nbt: &'source Nbt<'source>,
    tape_pos: usize,
    finished: bool,
    container_marker: PhantomData<Marker>,
}

impl<'source> NbtIterator<'source, CompoundMarker> {
    /// Makes an `NbtCompoundIterator` over the root compound of `nbt`.
    pub fn from_root(nbt: &'source Nbt) -> Self {
        Self {
            nbt,
            tape_pos: 1,
            finished: false,
            container_marker: PhantomData,
        }
    }
}

impl<'source> Iterator for NbtIterator<'source, CompoundMarker> {
    type Item = (Cow<'source, str>, NbtItem);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let parse_result = self.nbt.parse_at(self.tape_pos);
        match parse_result.item {
            Some(item) => {
                let name = self.nbt.get_name_at(self.tape_pos);
                self.tape_pos = parse_result.next_tape_pos;
                Some((name, item))
            }
            None => {
                self.finished = true;
                None
            }
        }
    }
}

impl<'source> Iterator for NbtIterator<'source, ListMarker> {
    type Item = NbtItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let parse_result = self.nbt.parse_at(self.tape_pos);
        match parse_result.item {
            Some(item) => {
                self.tape_pos = parse_result.next_tape_pos;
                Some(item)
            }
            None => {
                self.finished = true;
                None
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum NbtParseError {
    #[error("wrong starting NBT tag {tag:?}, expected {expected:?}")]
    WrongStartingTag { tag: Tag, expected: Tag },
    #[error("invalid NBT tag {value} at position {pos}")]
    InvalidTag { value: u8, pos: usize },
    #[error("invalid NBT list type {tag:?}")]
    InvalidListType { tag: Tag },
    #[error("unexpected NBT compound end tag at position {pos}")]
    UnexpectedEnd { pos: usize },
    #[error("sudden end of data, expected NBT compound end tag")]
    SuddenEnd,
    #[error("NBT string decoding error")]
    StringDecoding(#[from] simd_cesu8::DecodingError),
}

#[cfg(test)]
mod test {
    use std::io::Read;

    use super::*;
    use bytes::Buf;
    use flate2::read::GzDecoder;

    #[test]
    fn bigtest() {
        // TODO: Make this an actual test
        let bytes = include_bytes!("bigtest.nbt");
        let mut decoder = GzDecoder::new(bytes.reader());
        let mut buf = Vec::with_capacity(bytes.len());
        decoder.read_to_end(&mut buf).unwrap();

        let nbt = Nbt::from(&buf, false).unwrap();
        println!("{:#?}", &nbt.tape.0);

        println!("{:?}", nbt.root().get(&nbt, "nested compound test"));

        for (name, item) in nbt.iter() {
            println!("{:?} {:?}", name, item);
        }
    }
}
