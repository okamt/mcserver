//! **A type-safe, ergonomic NBT value representation.**

use std::borrow::Borrow;
use std::fmt::Display;
use std::ops::{ControlFlow, Deref};

use visitor::{NbtPrettyPrinter, NbtVisitor, NbtVisitorStrategy, NbtVisitorStrategySerial};

use super::*;

/// An [`NbtNode`] decoupled from its [`NbtParser`]. Call [`NbtNodeRef::bind`] to get an [`NbtNode`].
///
/// For when managing lifetimes becomes tricky.
#[derive(Clone, Copy)]
pub struct NbtNodeRef<Value>
where
    Value: NbtValueRepr,
{
    /// The position of this element in the `Tape`. Might be `None` if not represented on the tape (e.g. the ints in an [`NbtIntArray`]).
    pub(crate) tape_pos: Option<usize>,
    /// The value.
    pub(crate) value: Value,
}

impl<Value> NbtNodeRef<Value>
where
    Value: NbtValueRepr,
{
    /// Bind this [`NbtNodeRef`] to a [`NbtParser`].
    pub fn bind<'source, 'nbt>(
        &self,
        parser: &'nbt NbtParser<'source>,
    ) -> NbtNode<'source, 'nbt, Value> {
        NbtNode {
            parser,
            inner: Some(self.clone()),
        }
    }
}

impl<Value> Debug for NbtNodeRef<Value>
where
    Value: NbtValueRepr,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<Value> Deref for NbtNodeRef<Value>
where
    Value: NbtValueRepr,
{
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// **A type-safe, ergonomic NBT value representation.**
///
/// Allows chaining fallible access operations with extra compile time type safety, and stores parsing information for more ergonomic usage.
pub struct NbtNode<'source, 'nbt, Value>
where
    Value: NbtValueRepr,
{
    /// The [`NbtParser`] associated to this node.
    parser: &'nbt NbtParser<'source>,
    inner: Option<NbtNodeRef<Value>>,
}

impl<Value> Debug for NbtNode<'_, '_, Value>
where
    Value: NbtValueRepr,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'source, 'nbt, Value> NbtNode<'source, 'nbt, Value>
where
    Value: NbtValueRepr,
{
    pub fn new(parser: &'nbt NbtParser<'source>, value: Value, tape_pos: Option<usize>) -> Self {
        Self {
            parser,
            inner: Some(NbtNodeRef { value, tape_pos }),
        }
    }

    pub fn value(&self) -> Option<Value> {
        Some(self.inner?.value)
    }

    pub(crate) fn tape_pos(&self) -> Option<usize> {
        self.inner?.tape_pos
    }

    pub fn is_list_item(&self) -> bool {
        match self.inner {
            Some(NbtNodeRef {
                tape_pos: Some(tape_pos),
                ..
            }) => self.parser.tape_item(tape_pos).is_list_item(),
            _ => false,
        }
    }

    pub fn name(&self) -> Option<Cow<'nbt, str>> {
        if self.is_list_item() {
            None
        } else {
            Some(self.parser.get_name_at(self.tape_pos()?))
        }
    }

    pub fn unwrap(&self) -> Value {
        self.inner.unwrap().value
    }

    pub fn map<U, F>(&self, f: F) -> Option<U>
    where
        F: FnOnce(Value) -> U,
    {
        self.value().map(f)
    }

    pub fn and_then<U, F>(&self, f: F) -> Option<U>
    where
        F: FnOnce(Value) -> Option<U>,
    {
        self.value().and_then(f)
    }

    pub(crate) fn parser(&self) -> &'nbt NbtParser<'source> {
        self.parser
    }
}

macro_rules! item_getter {
    ($name:ident, $variant:ident, $ret:ty) => {
        pub fn $name(&self, key: impl Borrow<<Container as NbtMapRef>::Key>) -> Option<$ret> {
            self.value()?
                .get(self.parser, key.borrow())
                .and_then(|v| match v.value {
                    NbtValue::$variant(v) => Some(v),
                    _ => None,
                })
        }
    };
}

macro_rules! item_getter_ref {
    ($name:ident, $variant:ident, $ret:ty) => {
        pub fn $name(&self, key: impl Borrow<<Container as NbtMapRef>::Key>) -> Option<$ret> {
            self.value()?
                .get(self.parser, key.borrow())
                .and_then(|v| match v.value {
                    NbtValue::$variant(v) => Some(v.parse(self.parser)),
                    _ => None,
                })
        }
    };
}

macro_rules! node_getter {
    ($name:ident, $variant:ident, $ret:ty) => {
        pub fn $name(
            &self,
            key: impl Borrow<<Container as NbtMapRef>::Key>,
        ) -> NbtNode<'source, 'nbt, $ret> {
            let inner = self
                .value()
                .and_then(|v| v.get(self.parser, key.borrow()))
                .and_then(|v| match v.value {
                    NbtValue::$variant(value) => Some(NbtNodeRef {
                        value,
                        tape_pos: v.tape_pos,
                    }),
                    _ => None,
                });
            NbtNode {
                parser: self.parser,
                inner,
            }
        }
    };
}

impl<'source, 'nbt, Container> NbtNode<'source, 'nbt, Container>
where
    Container: NbtContainer,
{
    item_getter!(byte, Byte, i8);
    item_getter!(short, Short, i16);
    item_getter!(int, Int, i32);
    item_getter!(long, Long, i64);
    item_getter!(float, Float, f32);
    item_getter!(double, Double, f64);
    item_getter_ref!(byte_array, ByteArray, &'nbt [i8]);
    item_getter_ref!(string, String, &'nbt str);
    node_getter!(list, List, NbtList);
    item_getter_ref!(int_array, IntArray, &'nbt [i32]);
    item_getter_ref!(long_array, LongArray, &'nbt [i64]);
    node_getter!(compound, Compound, NbtCompound);

    pub fn get(&self, key: impl Borrow<<Container as NbtMapRef>::Key>) -> Option<NbtValue> {
        Some(self.value()?.get(self.parser, key.borrow())?.value)
    }

    pub fn get_node(
        &self,
        key: impl Borrow<<Container as NbtMapRef>::Key>,
    ) -> Option<NbtNode<'source, 'nbt, NbtValue>> {
        Some(
            self.value()?
                .get(self.parser, key.borrow())?
                .bind(self.parser),
        )
    }

    pub fn iter(&self) -> Option<NbtIterator<'source, 'nbt, Container>> {
        NbtIterator::from_node(&self)
    }

    /// Visits this [`NbtContainer`] with an [`NbtVisitor`], using an [`NbtVisitorStrategy`].
    pub fn visit_with_strategy<V, S, R, B, C>(
        &self,
        mut visitor: V,
        mut strategy: S,
    ) -> Result<R, B>
    where
        V: NbtVisitor<B, C, R>,
        S: NbtVisitorStrategy<V, B, C, R>,
    {
        while let Some(flow) = strategy.next(self.parser, &mut visitor) {
            match flow {
                ControlFlow::Continue(_) => {}
                ControlFlow::Break(b) => return Err(b),
            }
        }
        Ok(visitor.result())
    }

    /// Visits this [`NbtContainer`] with an [`NbtVisitor`], using [`NbtVisitorStrategySerial`].
    pub fn visit<V, R, B, C>(&self, visitor: V) -> Result<R, B>
    where
        V: NbtVisitor<B, C, R>,
    {
        let strategy = NbtVisitorStrategySerial::from_container(self.value().unwrap());
        self.visit_with_strategy(visitor, strategy)
    }
}

impl<'source, 'nbt, Container> Display for NbtNode<'source, 'nbt, Container>
where
    Container: NbtContainer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        _ = self.visit(NbtPrettyPrinter::new(f, 4));
        Ok(())
    }
}

macro_rules! value_getter {
    ($name:ident, $variant:ident, $ret:ty) => {
        pub fn $name(&self) -> Option<$ret> {
            self.and_then(|v| match v {
                NbtValue::$variant(v) => Some(v),
                _ => None,
            })
        }
    };
}

impl<'source, 'nbt> NbtNode<'source, 'nbt, NbtValue> {
    value_getter!(byte, Byte, i8);
    value_getter!(short, Short, i16);
    value_getter!(int, Int, i32);
    value_getter!(long, Long, i64);
    value_getter!(float, Float, f32);
    value_getter!(double, Double, f64);
}

impl<'source, 'nbt> Display for NbtNode<'source, 'nbt, NbtValue> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = self.value() {
            match value {
                NbtValue::End => write!(f, ""),
                NbtValue::Byte(value) => write!(f, "{}", value),
                NbtValue::Short(value) => write!(f, "{}", value),
                NbtValue::Int(value) => write!(f, "{}", value),
                NbtValue::Long(value) => write!(f, "{}", value),
                NbtValue::Float(value) => write!(f, "{}", value),
                NbtValue::Double(value) => write!(f, "{}", value),
                NbtValue::ByteArray(byte_array) => {
                    write!(f, "{:?}", byte_array.parse(self.parser))
                }
                NbtValue::String(string) => {
                    write!(f, "{:?}", string.parse(self.parser))
                }
                NbtValue::List(list) => {
                    write!(f, "{:?}", list.parse(self.parser))
                }
                NbtValue::Compound(compound) => {
                    write!(f, "{:?}", compound.parse(self.parser))
                }
                NbtValue::IntArray(int_array) => {
                    write!(f, "{:?}", int_array.parse(self.parser))
                }
                NbtValue::LongArray(long_array) => {
                    write!(f, "{:?}", long_array.parse(self.parser))
                }
            }
        } else {
            Ok(())
        }
    }
}

impl<'source, 'nbt, Array> NbtNode<'source, 'nbt, Array>
where
    Array: NbtArray,
{
    pub fn at(
        &self,
        key: impl Borrow<<Array as NbtMapRef>::Key>,
    ) -> Option<<Array as NbtMapRef>::Value> {
        Some(self.value()?.get(self.parser, key)?.value)
    }

    pub fn node_at(
        &self,
        key: impl Borrow<<Array as NbtMapRef>::Key>,
    ) -> Option<NbtNode<'source, 'nbt, <Array as NbtMapRef>::Value>> {
        Some(self.value()?.get(self.parser, key)?.bind(self.parser))
    }
}
