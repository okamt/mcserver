//! **A type-safe, ergonomic NBT value representation.**

use std::borrow::Borrow;
use std::ops::Deref;

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
    /// The value. Is `None` if end tag.
    pub(crate) value: Option<Value>,
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
            tape_item: self.tape_pos.map(|tape_pos| parser.tape_item(tape_pos)),
            value: self.value,
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
    type Target = Option<Value>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<Value> Into<Option<Value>> for NbtNodeRef<Value>
where
    Value: NbtValueRepr,
{
    fn into(self) -> Option<Value> {
        self.value
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
    pub(crate) parser: &'nbt NbtParser<'source>,
    /// The `TapeItem` that represents this value. Might be `None` if not represented on the tape (e.g. the ints in an [`NbtIntArray`]).
    pub(crate) tape_item: Option<&'nbt TapeItem>,
    /// The value. Is `None` if end tag.
    pub(crate) value: Option<Value>,
}

impl<Value> Debug for NbtNode<'_, '_, Value>
where
    Value: NbtValueRepr,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<Value> Deref for NbtNode<'_, '_, Value>
where
    Value: NbtValueRepr,
{
    type Target = Option<Value>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<Value> Into<Option<Value>> for NbtNode<'_, '_, Value>
where
    Value: NbtValueRepr,
{
    fn into(self) -> Option<Value> {
        self.value
    }
}

impl<'source, 'nbt, Value> NbtNode<'source, 'nbt, Value>
where
    Value: NbtValueRepr,
{
    pub fn value(&self) -> Option<Value> {
        self.value
    }

    pub fn is_list_item(&self) -> bool {
        match self.tape_item {
            Some(tape_item) => tape_item.is_list_item(),
            None => false,
        }
    }
}

macro_rules! item_getter {
    ($name:ident, $variant:ident, $ret:ty) => {
        pub fn $name(&self, key: impl Borrow<<Container as NbtMapRef>::Key>) -> Option<$ret> {
            self.value
                .and_then(|v| v.get(self.parser, key.borrow()).value)
                .and_then(|v| match v {
                    NbtValue::$variant(v) => Some(v),
                    _ => None,
                })
        }
    };
}

macro_rules! item_getter_ref {
    ($name:ident, $variant:ident, $ret:ty) => {
        pub fn $name(&self, key: impl Borrow<<Container as NbtMapRef>::Key>) -> Option<$ret> {
            self.value
                .and_then(|v| v.get(self.parser, key.borrow()).value)
                .and_then(|v| match v {
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
            let (value, tape_item) = match self
                .value
                .map(|v| v.get(self.parser, key.borrow()))
                .and_then(|v| match v.value {
                    Some(NbtValue::$variant(v_inner)) => Some((v_inner, v.tape_pos)),
                    _ => None,
                }) {
                Some((value, tape_pos)) => {
                    (Some(value), Some(self.parser.tape_item(tape_pos.unwrap())))
                }
                None => (None, None),
            };
            NbtNode {
                parser: self.parser,
                tape_item,
                value,
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
        self.value
            .and_then(|container| container.get(self.parser, key.borrow()).value)
    }

    pub fn get_node(
        &self,
        key: impl Borrow<<Container as NbtMapRef>::Key>,
    ) -> Option<NbtNode<'source, 'nbt, NbtValue>> {
        self.value
            .map(|container| container.get(self.parser, key.borrow()).bind(self.parser))
    }

    pub fn iter(&self) -> Option<NbtIterator<'source, 'nbt, Container>> {
        NbtIterator::from_node(&self)
    }
}

macro_rules! value_getter {
    ($name:ident, $variant:ident, $ret:ty) => {
        pub fn $name(&self) -> Option<$ret> {
            self.value.and_then(|v| match v {
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

impl<'source, 'nbt, Array> NbtNode<'source, 'nbt, Array>
where
    Array: NbtArray,
{
    pub fn at(
        &self,
        key: impl Borrow<<Array as NbtMapRef>::Key>,
    ) -> Option<<Array as NbtMapRef>::Value> {
        self.value
            .and_then(|array| array.get(self.parser, key).value)
    }

    pub fn node_at(
        &self,
        key: impl Borrow<<Array as NbtMapRef>::Key>,
    ) -> Option<NbtNode<'source, 'nbt, <Array as NbtMapRef>::Value>> {
        self.value
            .map(|array| array.get(self.parser, key).bind(self.parser))
    }
}
