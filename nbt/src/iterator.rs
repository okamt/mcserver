//! Iterator over NBT containers.

use std::{borrow::Cow, marker::PhantomData};

use super::*;

/// An iterator over an NBT compound or list. Uses the `Container` typestate to provide different [`Iterator`] implementations.
pub struct NbtIterator<'source, 'nbt, Container>
where
    'source: 'nbt,
    Container: NbtContainer,
{
    parser: &'nbt NbtParser<'source>,
    tape_pos: usize,
    finished: bool,
    _phantom: PhantomData<Container>,
}

impl<'source, 'nbt, Container> NbtIterator<'source, 'nbt, Container>
where
    Container: NbtContainer,
{
    /// Makes an [`NbtIterator`] over the specified [`NbtNode`].
    pub fn from_node(node: &NbtNode<'source, 'nbt, Container>) -> Option<Self> {
        node.value().map(|value| Self {
            parser: node.parser,
            tape_pos: value.tape_pos() + 1,
            finished: false,
            _phantom: PhantomData,
        })
    }
}

impl<'source, 'nbt> NbtIterator<'source, 'nbt, NbtCompound> {
    /// Makes an [`NbtIterator`] over the root compound of `nbt`.
    pub fn from_root(nbt: &'nbt NbtParser<'source>) -> Self {
        Self {
            parser: nbt,
            tape_pos: 1,
            finished: false,
            _phantom: PhantomData,
        }
    }
}

impl<'source, 'nbt> Iterator for NbtIterator<'source, 'nbt, NbtCompound> {
    type Item = (Cow<'nbt, str>, NbtValue);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let parse_result = self.parser.parse_at(self.tape_pos);
        match parse_result.item {
            Some(item) => {
                let name = self.parser.get_name_at(self.tape_pos);
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

impl<'source, 'nbt> Iterator for NbtIterator<'source, 'nbt, NbtList> {
    type Item = NbtValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let parse_result = self.parser.parse_at(self.tape_pos);
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
