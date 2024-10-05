//! Visitor API.

use std::fmt::Write;

use super::*;

pub trait NbtVisitor<'source, 'nbt> {
    type Ok;
    type Err;

    fn visit_value(
        &mut self,
        value: &NbtNode<'source, 'nbt, NbtValue>,
    ) -> Result<Self::Ok, Self::Err>;
    fn enter_compound(
        &mut self,
        compound: &NbtNode<'source, 'nbt, NbtCompound>,
    ) -> Result<Self::Ok, Self::Err>;
    fn leave_compound(
        &mut self,
        compound: &NbtNode<'source, 'nbt, NbtCompound>,
    ) -> Result<Self::Ok, Self::Err>;
    fn enter_list(&mut self, list: &NbtNode<'source, 'nbt, NbtList>)
        -> Result<Self::Ok, Self::Err>;
    fn leave_list(&mut self, list: &NbtNode<'source, 'nbt, NbtList>)
        -> Result<Self::Ok, Self::Err>;
}

pub struct NbtPrettyPrinter<'w, W>
where
    W: Write,
{
    writer: &'w mut W,
    indent_size: usize,
    indent_level: usize,
}

impl<'w, W> NbtPrettyPrinter<'w, W>
where
    W: Write,
{
    pub fn new(writer: &'w mut W, indent_size: usize) -> Self {
        Self {
            writer,
            indent_size,
            indent_level: 0,
        }
    }

    fn write_indent(&mut self) -> std::fmt::Result {
        for _ in 0..(self.indent_size * self.indent_level) {
            _ = self.writer.write_char(' ')?;
        }
        Ok(())
    }
}

impl<'w, W> NbtVisitor<'_, '_> for NbtPrettyPrinter<'w, W>
where
    W: Write,
{
    type Ok = ();
    type Err = std::fmt::Error;

    fn visit_value(&mut self, value: &NbtNode<'_, '_, NbtValue>) -> Result<Self::Ok, Self::Err> {
        self.write_indent()?;
        if let Some(name) = value.name() {
            write!(self.writer, "{}: ", name)?;
        }
        write!(self.writer, "{}\n", value)
    }

    fn enter_compound(
        &mut self,
        compound: &NbtNode<'_, '_, NbtCompound>,
    ) -> Result<Self::Ok, Self::Err> {
        self.write_indent()?;
        if let Some(name) = compound.name() {
            if name.len() > 0 {
                write!(self.writer, "{} ", name)?;
            }
        }
        self.indent_level += 1;
        write!(self.writer, "{{\n")
    }

    fn leave_compound(
        &mut self,
        _compound: &NbtNode<'_, '_, NbtCompound>,
    ) -> Result<Self::Ok, Self::Err> {
        self.indent_level -= 1;
        self.write_indent()?;
        write!(self.writer, "}}\n")
    }

    fn enter_list(&mut self, list: &NbtNode<'_, '_, NbtList>) -> Result<Self::Ok, Self::Err> {
        self.write_indent()?;
        self.indent_level += 1;
        write!(self.writer, "{} [\n", list.name().unwrap())
    }

    fn leave_list(&mut self, _list: &NbtNode<'_, '_, NbtList>) -> Result<Self::Ok, Self::Err> {
        self.indent_level -= 1;
        self.write_indent()?;
        write!(self.writer, "]\n")
    }
}

pub trait NbtVisitorStrategy {
    fn step<'source, 'nbt, V: NbtVisitor<'source, 'nbt>>(
        &mut self,
        parser: &'nbt NbtParser<'source>,
        visitor: &mut V,
    ) -> Option<Result<V::Ok, V::Err>>
    where
        'source: 'nbt;
}

pub struct NbtVisitorStrategySerial {
    container_tape_pos: usize,
    tape_pos: usize,
    finished: bool,
}

impl NbtVisitorStrategySerial {
    pub fn from_root() -> Self {
        Self {
            container_tape_pos: 0,
            tape_pos: 0,
            finished: false,
        }
    }

    pub fn from_container<C>(container: C) -> Self
    where
        C: NbtContainer,
    {
        Self {
            container_tape_pos: container.tape_pos(),
            tape_pos: container.tape_pos(),
            finished: false,
        }
    }
}

impl NbtVisitorStrategy for NbtVisitorStrategySerial {
    fn step<'source, 'nbt, V: NbtVisitor<'source, 'nbt>>(
        &mut self,
        parser: &'nbt NbtParser<'source>,
        visitor: &mut V,
    ) -> Option<Result<V::Ok, V::Err>>
    where
        'source: 'nbt,
    {
        if self.finished {
            return None;
        }

        let result = match parser.parse_at(self.tape_pos).value {
            NbtValue::End => {
                let tape_item = parser.tape_item(self.tape_pos);
                let start_tag_tape_pos = tape_item.get_data() as usize;
                let start_tape_item = parser.tape_item(start_tag_tape_pos);

                if start_tag_tape_pos == self.container_tape_pos {
                    self.finished = true;
                }

                match start_tape_item.get_tag() {
                    Tag::Compound => visitor.leave_compound(&NbtNode::new(
                        parser,
                        NbtCompound(start_tag_tape_pos),
                        Some(start_tag_tape_pos),
                    )),
                    Tag::List => visitor.leave_list(&NbtNode::new(
                        parser,
                        NbtList(start_tag_tape_pos),
                        Some(start_tag_tape_pos),
                    )),
                    _ => unreachable!(),
                }
            }
            NbtValue::Compound(compound) => {
                visitor.enter_compound(&NbtNode::new(parser, compound, Some(self.tape_pos)))
            }
            NbtValue::List(list) => {
                visitor.enter_list(&NbtNode::new(parser, list, Some(self.tape_pos)))
            }
            value => visitor.visit_value(
                &NbtNodeRef {
                    tape_pos: Some(self.tape_pos),
                    value,
                }
                .bind(parser),
            ),
        };
        self.tape_pos += 1;
        if result.is_err() {
            self.finished = true;
        }
        Some(result)
    }
}

pub struct NbtVisitorIterator<'nbt, 'source, V, S>
where
    V: NbtVisitor<'source, 'nbt>,
    S: NbtVisitorStrategy,
{
    visitor: V,
    strategy: S,
    parser: &'nbt NbtParser<'source>,
}

impl<'nbt, 'source, V, S> NbtVisitorIterator<'nbt, 'source, V, S>
where
    V: NbtVisitor<'source, 'nbt>,
    S: NbtVisitorStrategy,
{
    pub fn new(visitor: V, strategy: S, parser: &'nbt NbtParser<'source>) -> Self {
        Self {
            visitor,
            strategy,
            parser,
        }
    }
}

impl<'nbt, 'source, V> NbtVisitorIterator<'nbt, 'source, V, NbtVisitorStrategySerial>
where
    V: NbtVisitor<'source, 'nbt>,
{
    pub fn with_serial_strategy(visitor: V, parser: &'nbt NbtParser<'source>) -> Self {
        Self {
            visitor,
            strategy: NbtVisitorStrategySerial::from_root(),
            parser,
        }
    }
}

impl<'nbt, 'source, V, S> Iterator for NbtVisitorIterator<'nbt, 'source, V, S>
where
    V: NbtVisitor<'source, 'nbt>,
    S: NbtVisitorStrategy,
{
    type Item = Result<V::Ok, V::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        self.strategy.step(self.parser, &mut self.visitor)
    }
}
