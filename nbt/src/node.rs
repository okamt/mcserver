//! **A type-safe, ergonomic NBT value representation.**

use std::borrow::Borrow;
use std::ops::Deref;

use super::*;

/// **A type-safe, ergonomic NBT value representation.**
pub struct NbtNode<'source, 'nbt, Value>
where
    Value: NbtValueOrRef,
{
    pub(crate) parser: &'nbt NbtParser<'source>,
    value: Option<Value>,
}

impl<'source, 'nbt, Value> Deref for NbtNode<'source, 'nbt, Value>
where
    Value: NbtValueOrRef,
{
    type Target = Option<Value>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'source, 'nbt, Value> Into<Option<Value>> for NbtNode<'source, 'nbt, Value>
where
    Value: NbtValueOrRef,
{
    fn into(self) -> Option<Value> {
        self.value
    }
}

impl<'source, 'nbt, Value> NbtNode<'source, 'nbt, Value>
where
    Value: NbtValueOrRef,
{
    pub fn new(parser: &'nbt NbtParser<'source>, value: Value) -> Self {
        Self {
            parser,
            value: Some(value),
        }
    }

    pub fn value(&self) -> Option<Value> {
        self.value
    }
}

macro_rules! item_getter {
    ($name:ident, $variant:ident, $ret:ty) => {
        pub fn $name(&self, key: impl Borrow<<Container as NbtMapRef>::Key>) -> Option<$ret> {
            self.value
                .and_then(|v| v.get(self.parser, key.borrow()))
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
                .and_then(|v| v.get(self.parser, key.borrow()))
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
            let value = self
                .value
                .and_then(|v| v.get(self.parser, key.borrow()))
                .and_then(|v| match v {
                    NbtValue::$variant(v) => Some(v),
                    _ => None,
                });
            NbtNode {
                parser: self.parser,
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

    pub fn item(&self, key: impl Borrow<<Container as NbtMapRef>::Key>) -> Option<NbtValue> {
        self.value.and_then(|v| v.get(self.parser, key.borrow()))
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
    Array: NbtMapRef<Output<'nbt>: ?NbtRef>,
{
    pub fn get(
        &self,
        key: impl Borrow<<Array as NbtMapRef>::Key>,
    ) -> Option<<Array as NbtMapRef>::Value<'_>> {
        self.value.and_then(|v| v.get(self.parser, key))
    }
}
