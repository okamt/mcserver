use std::borrow::Borrow;
use std::{borrow::Cow, collections::HashMap, fmt::Debug, ops::Index};

use crate::marker::*;
use crate::parse::*;
use crate::tag::*;
use crate::value::*;
use once_map::OnceMap;
use self_cell::self_cell;
use thiserror::Error;

pub mod iterator;
pub(crate) mod marker;
pub mod node;
pub(crate) mod parse;
pub(crate) mod tag;
pub mod value;

pub use iterator::*;
pub use node::*;
pub use value::*;

/// An NBT file representation, with a compound as root tag.
struct Nbt<'source> {
    source: Cow<'source, [u8]>,
    tape: Tape,
}

impl<'source> Nbt<'source> {
    /// Gets the name of the `tape_item`.
    pub(crate) fn get_name<'nbt>(&'nbt self, tape_item: &TapeItem) -> Cow<'nbt, str> {
        let name_len = tape_item.get_name_len();
        let source_pos = tape_item.get_source_pos() + 3;
        simd_cesu8::decode_lossy(&self.source[source_pos..source_pos + (name_len as usize)])
    }

    pub(crate) fn get_name_at<'nbt>(&'nbt self, tape_pos: usize) -> Cow<'nbt, str> {
        self.get_name(&self.tape[tape_pos])
    }

    pub(crate) fn parse_at(&self, tape_pos: usize) -> NbtParseResult {
        let tape_item = &self.tape[tape_pos];

        let value = match tape_item.get_tag() {
            Tag::End => NbtValue::End,
            Tag::Byte => NbtValue::Byte(tape_item.get_data() as i8),
            Tag::Short => NbtValue::Short(tape_item.get_data() as i16),
            Tag::Int => NbtValue::Int(tape_item.get_data() as i32),
            Tag::Long => NbtValue::Long(tape_item.get_data() as i64),
            Tag::Float => NbtValue::Float(f32::from_bits(tape_item.get_data() as u32)),
            Tag::Double => NbtValue::Double(f64::from_bits(tape_item.get_data())),
            Tag::ByteArray => NbtValue::ByteArray(NbtByteArray(tape_pos)),
            Tag::String => NbtValue::String(NbtString(tape_pos)),
            Tag::List => NbtValue::List(NbtList(tape_pos)),
            Tag::Compound => NbtValue::Compound(NbtCompound(tape_pos)),
            Tag::IntArray => NbtValue::IntArray(NbtIntArray(tape_pos)),
            Tag::LongArray => NbtValue::LongArray(NbtLongArray(tape_pos)),
        };
        let next_tape_pos = match value {
            NbtValue::Compound(_) | NbtValue::List(_) => 1 + tape_item.get_data() as usize,
            NbtValue::End => 0,
            _ => tape_pos + 1,
        };

        NbtParseResult {
            value,
            next_tape_pos,
        }
    }
}

/// NBT parsing cache using `OnceMap`.
struct NbtCache<'nbt> {
    // Needs Box for StableDeref.
    pub(crate) compounds: OnceMap<usize, Box<HashMap<Cow<'nbt, str>, NbtNodeRef<NbtValue>>>>,
    pub(crate) lists: OnceMap<usize, Vec<NbtNodeRef<NbtValue>>>,
    pub(crate) strings: OnceMap<usize, Cow<'nbt, str>>,
}

self_cell!(
    /// Auto-generated, see [`NbtParser`].
    ///
    /// [`NbtCache`] needs to store references to [`Nbt::source`]. To avoid having the user juggle both structs,
    /// we make a self-referential struct using `safe_cell`.
    ///
    /// However, `safe_cell` generates a bunch of public methods we don't necessarily want to show to the end user,
    /// so we wrap it in another struct that implements all the important methods ([`NbtParser`]).
    struct NbtParserInner<'source> {
        owner: Nbt<'source>,
        #[not_covariant]
        dependent: NbtCache,
    }
);

/// **A lazy NBT parser that caches parsed compounds, lists and strings.**
pub struct NbtParser<'source>(NbtParserInner<'source>);

impl<'source> NbtParser<'source> {
    /// Creates an [`NbtParser`] from a `source`.
    pub fn parse<S>(source: S, is_network_nbt: bool) -> Result<NbtParser<'source>, NbtParseError>
    where
        S: Into<Cow<'source, [u8]>>,
    {
        let source = source.into();
        let tape = Tape::parse(&source, is_network_nbt)?;
        Ok(Self(NbtParserInner::new(Nbt { source, tape }, move |_| {
            NbtCache {
                compounds: OnceMap::new(),
                lists: OnceMap::new(),
                strings: OnceMap::new(),
            }
        })))
    }

    pub(crate) fn source(&self) -> &[u8] {
        &self.0.borrow_owner().source
    }

    pub(crate) fn tape(&self) -> &Tape {
        &self.0.borrow_owner().tape
    }

    pub(crate) fn tape_item(&self, pos: usize) -> &TapeItem {
        &self.tape().0[pos]
    }

    pub(crate) fn with_cache<'outer_fn, Ret>(
        &'outer_fn self,
        func: impl for<'_q> FnOnce(&'_q Nbt<'source>, &'outer_fn NbtCache<'_q>) -> Ret,
    ) -> Ret {
        self.0.with_dependent(func)
    }

    /// Gets an [`NbtNode`] representing the root compound.
    pub fn root<'nbt>(&'nbt self) -> NbtNode<'source, 'nbt, NbtCompound> {
        NbtNode::new(&self, NbtCompound(0), Some(0))
    }

    /// Returns an [`NbtIterator`] over the root compound.
    pub fn iter<'nbt>(&'nbt self) -> NbtIterator<'nbt, 'source, NbtCompound> {
        NbtIterator::from_root(&self)
    }

    pub(crate) fn get_name<'nbt>(&'nbt self, tape_item: &TapeItem) -> Cow<'nbt, str> {
        self.0.borrow_owner().get_name(tape_item)
    }

    pub(crate) fn get_name_at<'nbt>(&'nbt self, tape_pos: usize) -> Cow<'nbt, str> {
        self.0.borrow_owner().get_name_at(tape_pos)
    }

    pub(crate) fn parse_at(&self, tape_pos: usize) -> NbtParseResult {
        self.0.borrow_owner().parse_at(tape_pos)
    }
}

/// The result of parsing an [`NbtValue`] at a specific tape position.
///
/// This is only ever used internally.
#[derive(Debug)]
pub(crate) struct NbtParseResult {
    pub value: NbtValue,
    pub next_tape_pos: usize,
}

/// An error during the initial parsing stage.
///
/// Can only occur in [`NbtParser::parse`], all subsequent parsing is infallible.
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
}

#[cfg(test)]
mod test {
    use std::io::Read;

    use super::*;
    use bytes::Buf;
    use flate2::read::GzDecoder;

    fn get_bigtest_parser<'source>() -> NbtParser<'source> {
        const BYTES: &[u8] = include_bytes!("bigtest.nbt");
        let mut decoder = GzDecoder::new(BYTES.reader());
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();

        NbtParser::parse(buf, false).unwrap()
    }

    #[test]
    fn bigtest() {
        // TODO: Make this an actual test
        let parser = get_bigtest_parser();

        for (name, item) in parser.root().iter().unwrap() {
            println!("{:?} {:?}", name, item);
        }

        dbg!(parser.root().list("nested compound test").get(0));
    }

    #[test]
    fn bigtest_cache() {
        let parser = get_bigtest_parser();

        parser.with_cache(|_nbt, cache| assert_eq!(cache.compounds.read_only_view().len(), 0));
        assert_eq!(parser.root().byte("byteTest"), Some(127));
        parser.with_cache(|_nbt, cache| {
            assert_eq!(cache.compounds.read_only_view().len(), 1);
            assert!(cache.compounds.get(&0).is_some());
        });
    }
}
