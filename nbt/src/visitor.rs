//! Visitor API.

use std::{fmt::Write, ops::ControlFlow};

use super::*;

pub trait NbtVisitor<B, C = (), R = ()> {
    fn visit_value(&mut self, value: &NbtNode<'_, '_, NbtValue>) -> ControlFlow<B, C>;
    fn enter_compound(&mut self, compound: &NbtNode<'_, '_, NbtCompound>) -> ControlFlow<B, C>;
    fn leave_compound(&mut self, compound: &NbtNode<'_, '_, NbtCompound>) -> ControlFlow<B, C>;
    fn enter_list(&mut self, list: &NbtNode<'_, '_, NbtList>) -> ControlFlow<B, C>;
    fn leave_list(&mut self, list: &NbtNode<'_, '_, NbtList>) -> ControlFlow<B, C>;
    fn result(&mut self) -> R;
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

    fn write_indent(&mut self) {
        for _ in 0..(self.indent_size * self.indent_level) {
            _ = self.writer.write_char(' ');
        }
    }
}

impl<'w, W> NbtVisitor<()> for NbtPrettyPrinter<'w, W>
where
    W: Write,
{
    fn visit_value(&mut self, value: &NbtNode<'_, '_, NbtValue>) -> ControlFlow<(), ()> {
        self.write_indent();
        if let Some(name) = value.name() {
            _ = write!(self.writer, "{}: ", name);
        }
        _ = write!(self.writer, "{}\n", value);
        ControlFlow::Continue(())
    }

    fn enter_compound(&mut self, compound: &NbtNode<'_, '_, NbtCompound>) -> ControlFlow<(), ()> {
        self.write_indent();
        if let Some(name) = compound.name() {
            _ = write!(self.writer, "{} ", name);
        }
        _ = write!(self.writer, "{{\n");
        self.indent_level += 1;
        ControlFlow::Continue(())
    }

    fn leave_compound(&mut self, _compound: &NbtNode<'_, '_, NbtCompound>) -> ControlFlow<(), ()> {
        self.indent_level -= 1;
        self.write_indent();
        _ = write!(self.writer, "}}\n");
        ControlFlow::Continue(())
    }

    fn enter_list(&mut self, list: &NbtNode<'_, '_, NbtList>) -> ControlFlow<(), ()> {
        self.write_indent();
        _ = write!(self.writer, "{} [\n", list.name().unwrap());
        self.indent_level += 1;
        ControlFlow::Continue(())
    }

    fn leave_list(&mut self, _list: &NbtNode<'_, '_, NbtList>) -> ControlFlow<(), ()> {
        self.indent_level -= 1;
        self.write_indent();
        _ = write!(self.writer, "]\n");
        ControlFlow::Continue(())
    }

    fn result(&mut self) -> () {
        ()
    }
}

pub trait NbtVisitorStrategy<V: NbtVisitor<B, C, R>, B, C = (), R = ()> {
    fn next(&mut self, parser: &NbtParser, visitor: &mut V) -> Option<ControlFlow<B, C>>;
}

pub struct NbtVisitorStrategySerial {
    container_tape_pos: usize,
    tape_pos: usize,
    finished: bool,
}

impl NbtVisitorStrategySerial {
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

impl<V, B, C, R> NbtVisitorStrategy<V, B, C, R> for NbtVisitorStrategySerial
where
    V: NbtVisitor<B, C, R>,
{
    fn next(&mut self, parser: &NbtParser, visitor: &mut V) -> Option<ControlFlow<B, C>> {
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
        Some(result)
    }
}
