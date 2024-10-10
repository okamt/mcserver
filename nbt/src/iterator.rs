//! Iterator over NBT containers.

use std::marker::PhantomData;

use super::*;

/// An iterator over an NBT compound or list. Uses the `Container` typestate to provide different [`Iterator`] implementations.
pub struct NbtIterator<'source, 'nbt, Container>
where
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
            parser: node.parser(),
            tape_pos: value.tape_pos() + 1,
            finished: false,
            _phantom: PhantomData,
        })
    }
}

impl<'source, 'nbt> NbtIterator<'source, 'nbt, NbtCompound> {
    /// Makes an [`NbtIterator`] over the root compound of the [`NbtParser`].
    pub fn from_root(parser: &'nbt NbtParser<'source>) -> Self {
        Self {
            parser,
            tape_pos: 1,
            finished: false,
            _phantom: PhantomData,
        }
    }
}

impl<'source, 'nbt> Iterator for NbtIterator<'source, 'nbt, NbtCompound> {
    type Item = (&'nbt str, NbtNode<'source, 'nbt, NbtValue>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let parse_result = self.parser.parse_at(self.tape_pos);
        match parse_result.value {
            NbtValue::End => {
                self.finished = true;
                None
            }
            value => {
                let name = self.parser.get_name_at(self.tape_pos);
                let result = Some((name, NbtNode::new(self.parser, value, Some(self.tape_pos))));
                self.tape_pos = parse_result.next_tape_pos;
                result
            }
        }
    }
}

impl<'source, 'nbt> Iterator for NbtIterator<'source, 'nbt, NbtList> {
    type Item = NbtNode<'source, 'nbt, NbtValue>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let parse_result = self.parser.parse_at(self.tape_pos);
        match parse_result.value {
            NbtValue::End => {
                self.finished = true;
                None
            }
            value => {
                let result = Some(NbtNode::new(self.parser, value, Some(self.tape_pos)));
                self.tape_pos = parse_result.next_tape_pos;
                result
            }
        }
    }
}
