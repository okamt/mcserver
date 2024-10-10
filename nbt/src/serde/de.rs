use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Unexpected, Visitor};
use serde::{forward_to_deserialize_any, Deserialize};

use crate::{
    NbtCompound, NbtList, NbtNode, NbtParser, NbtRef, NbtValue, NbtVisitor, NbtVisitorStrategy,
    NbtVisitorStrategySerial,
};

use super::*;

pub struct Deserializer<'source, 'nbt> {
    parser: &'nbt NbtParser<'source>,
    strategy: NbtVisitorStrategySerial,
    visit_this_first: Option<Visit<'source, 'nbt>>,
    stack: Vec<StackItem>,
}

impl<'source, 'nbt> Deserializer<'source, 'nbt> {
    pub fn from_parser(parser: &'nbt NbtParser<'source>) -> Self {
        Self {
            parser,
            strategy: NbtVisitorStrategySerial::from_root(),
            visit_this_first: None,
            stack: Vec::with_capacity(64),
        }
    }

    fn visit(&mut self) -> Result<Visit<'source, 'nbt>> {
        if self.visit_this_first.is_some() {
            let mut this = None;
            std::mem::swap(&mut self.visit_this_first, &mut this);
            return Ok(this.unwrap());
        }

        self.strategy
            .step(&self.parser, &mut DeserializerVisitor)
            .ok_or(Error::Eof)?
    }
}

pub fn from_parser<'nbt, T>(parser: &'nbt NbtParser<'_>) -> Result<T>
where
    T: Deserialize<'nbt>,
{
    let mut deserializer = Deserializer::from_parser(parser);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

/// An [`NbtVisitor`] that returns [`VisitorCall`]s.
struct DeserializerVisitor;

/// Reified calls to [`NbtVisitor`] functions.
#[derive(Debug)]
enum Visit<'source, 'nbt> {
    VisitValue(NbtNode<'source, 'nbt, NbtValue>),
    EnterCompound(NbtNode<'source, 'nbt, NbtCompound>),
    LeaveCompound(NbtNode<'source, 'nbt, NbtCompound>),
    EnterList(NbtNode<'source, 'nbt, NbtList>),
    LeaveList(NbtNode<'source, 'nbt, NbtList>),
}

impl<'source, 'nbt> NbtVisitor<'source, 'nbt> for DeserializerVisitor
where
    'source: 'nbt,
{
    type Ok = Visit<'source, 'nbt>;
    type Err = Error;

    fn visit_value(
        &mut self,
        value: &NbtNode<'source, 'nbt, NbtValue>,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        Ok(Visit::VisitValue(value.clone()))
    }

    fn enter_compound(
        &mut self,
        compound: &NbtNode<'source, 'nbt, NbtCompound>,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        Ok(Visit::EnterCompound(compound.clone()))
    }

    fn leave_compound(
        &mut self,
        compound: &NbtNode<'source, 'nbt, NbtCompound>,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        Ok(Visit::LeaveCompound(compound.clone()))
    }

    fn enter_list(
        &mut self,
        list: &NbtNode<'source, 'nbt, NbtList>,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        Ok(Visit::EnterList(list.clone()))
    }

    fn leave_list(
        &mut self,
        list: &NbtNode<'source, 'nbt, NbtList>,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        Ok(Visit::LeaveList(list.clone()))
    }
}

impl<'source, 'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'source, 'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.visit_this_first.is_none() {
            if let Some(StackItem::Compound) = self.stack.last() {
                return self.deserialize_identifier(visitor);
            }
        }

        match self.visit()? {
            Visit::VisitValue(node) => match node.value().unwrap() {
                NbtValue::End => self.deserialize_any(visitor),
                NbtValue::Byte(value) => visitor.visit_i8(value),
                NbtValue::Short(value) => visitor.visit_i16(value),
                NbtValue::Int(value) => visitor.visit_i32(value),
                NbtValue::Long(value) => visitor.visit_i64(value),
                NbtValue::Float(value) => visitor.visit_f32(value),
                NbtValue::Double(value) => visitor.visit_f64(value),
                NbtValue::ByteArray(_) => {
                    visitor.visit_borrowed_bytes(bytemuck::cast_slice(node.byte_array().unwrap()))
                }
                NbtValue::String(_) => visitor.visit_borrowed_str(node.string().unwrap()),
                NbtValue::List(_) => unreachable!(),
                NbtValue::Compound(_) => unreachable!(),
                NbtValue::IntArray(_) => {
                    visitor.visit_seq(ArraySeq::new(node.int_array().unwrap()))
                }
                NbtValue::LongArray(_) => {
                    visitor.visit_seq(ArraySeq::new(node.long_array().unwrap()))
                }
            },
            Visit::EnterCompound(_) => {
                self.stack.push(StackItem::Compound);
                visitor.visit_map(self)
            }
            Visit::LeaveCompound(_) => self.deserialize_any(visitor),
            Visit::EnterList(_) => {
                self.stack.push(StackItem::List);
                visitor.visit_seq(self)
            }
            Visit::LeaveList(_) => Err(Error::NoMoreValues),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let visit = self.visit()?;
        self.visit_this_first = Some(visit);
        match self.visit_this_first.as_ref().unwrap() {
            Visit::VisitValue(node) => visitor.visit_str(&node.name().ok_or(Error::NoName)?),
            Visit::EnterCompound(node) => visitor.visit_str(&node.name().ok_or(Error::NoName)?),
            Visit::EnterList(node) => visitor.visit_str(&node.name().ok_or(Error::NoName)?),
            _ => {
                self.visit_this_first = None;
                Err(Error::NoMoreValues)
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.visit()? {
            Visit::VisitValue(node) => match node.value().unwrap() {
                NbtValue::Byte(value) => visitor.visit_bool(value > 0),
                _ => Err(de::Error::invalid_type(
                    node.as_unexpected().unwrap(),
                    &"a byte (bool)",
                )),
            },
            _ => Err(Error::NoMoreValues),
        }
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum ignored_any
    }
}

impl<'de, 'a> MapAccess<'de> for &'a mut Deserializer<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match seed.deserialize(&mut **self) {
            Err(Error::NoMoreValues) => {
                self.stack.pop();
                Ok(None)
            }
            v => v.map(Some),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut **self)
    }
}

impl<'de, 'a> SeqAccess<'de> for &'a mut Deserializer<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match seed.deserialize(&mut **self) {
            Err(Error::NoMoreValues) => {
                self.stack.pop();
                Ok(None)
            }
            v => v.map(Some),
        }
    }
}

struct ArraySeq<'nbt, T> {
    array: &'nbt [T],
    pos: usize,
}

impl<'de, T> ArraySeq<'de, T> {
    pub fn new(array: &'de [T]) -> Self {
        Self { array, pos: 0 }
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut ArraySeq<'de, i32> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.array.get(self.pos) {
            Some(value) => {
                self.pos += 1;
                visitor.visit_i32(*value)
            }
            None => Err(Error::NoMoreValues),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for ArraySeq<'de, i32> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(Some)
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut ArraySeq<'de, i64> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.array.get(self.pos) {
            Some(value) => {
                self.pos += 1;
                visitor.visit_i64(*value)
            }
            None => Err(Error::NoMoreValues),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for ArraySeq<'de, i64> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(Some)
    }
}

trait AsUnexpected {
    fn as_unexpected(&self) -> Option<Unexpected>;
}

impl<'source, 'nbt> AsUnexpected for NbtNode<'source, 'nbt, NbtValue> {
    fn as_unexpected(&self) -> Option<Unexpected> {
        self.value().and_then(|v| {
            Some(match v {
                NbtValue::End => return None,
                NbtValue::Byte(value) => Unexpected::Signed(value.into()),
                NbtValue::Short(value) => Unexpected::Signed(value.into()),
                NbtValue::Int(value) => Unexpected::Signed(value.into()),
                NbtValue::Long(value) => Unexpected::Signed(value.into()),
                NbtValue::Float(value) => Unexpected::Float(value.into()),
                NbtValue::Double(value) => Unexpected::Float(value.into()),
                NbtValue::ByteArray(value) => {
                    Unexpected::Bytes(bytemuck::cast_slice(value.parse(self.parser())))
                }
                NbtValue::String(value) => Unexpected::Str(value.parse(self.parser())),
                NbtValue::List(_) => Unexpected::Seq,
                NbtValue::Compound(_) => Unexpected::Map,
                NbtValue::IntArray(_) => Unexpected::Seq,
                NbtValue::LongArray(_) => Unexpected::Seq,
            })
        })
    }
}
