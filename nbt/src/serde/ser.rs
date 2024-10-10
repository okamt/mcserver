use std::borrow::Cow;

use super::*;
use crate::Tag;
use bytes::BufMut;
use serde::{ser, Serialize};

pub struct Serializer<'source> {
    stack: Vec<StackItem>,
    current_name: Cow<'source, str>,
    output: Vec<u8>,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'source> Serializer<'source> {
    fn new() -> Self {
        Self {
            stack: Vec::with_capacity(64),
            current_name: "".into(),
            output: Vec::with_capacity(64),
        }
    }

    fn serialize_tag(&mut self, tag: Tag) {
        match tag {
            Tag::End => {
                self.output.put_u8(tag.into());
                self.stack.pop();
                return;
            }
            Tag::Compound => self.stack.push(StackItem::Compound),
            Tag::List => self.stack.push(StackItem::List),
            _ => {}
        }

        match self.stack.last() {
            Some(StackItem::Compound) => {
                self.output.put_u8(tag.into());
                self.output.put_u16(self.current_name.len() as u16);
                self.output.put_slice(self.current_name.as_bytes());
            }
            _ => {}
        }
    }
}

impl<'a, 'source> ser::Serializer for &'a mut Serializer<'source> {
    type Ok = Tag;
    type Error = Error;

    type SerializeSeq = SerializerSeq<'a, 'source>;
    type SerializeTuple = SerializerTuple<'a, 'source>;
    type SerializeTupleStruct = SerializerTuple<'a, 'source>;
    type SerializeTupleVariant = SerializerTuple<'a, 'source>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.serialize_i8(if v { 1 } else { 0 })
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_tag(Tag::Byte);
        self.output.put_i8(v);
        Ok(Tag::Byte)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_tag(Tag::Short);
        self.output.put_i16(v);
        Ok(Tag::Short)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_tag(Tag::Int);
        self.output.put_i32(v);
        Ok(Tag::Int)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.serialize_tag(Tag::Long);
        self.output.put_i64(v);
        Ok(Tag::Long)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_i8(v as i8)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_i16(v as i16)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_i32(v as i32)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_tag(Tag::Float);
        self.output.put_f32(v);
        Ok(Tag::Float)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.serialize_tag(Tag::Double);
        self.output.put_f64(v);
        Ok(Tag::Double)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.serialize_tag(Tag::String);
        let string = simd_cesu8::mutf8::encode(v);
        self.output.put_u16(string.len() as u16);
        self.output.put_slice(&string);
        Ok(Tag::String)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.serialize_tag(Tag::ByteArray);
        self.output.put_i32(v.len() as i32);
        self.output.put_slice(v);
        Ok(Tag::ByteArray)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(Tag::End)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(Tag::End)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(Tag::End)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_u32(variant_index)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_tag(Tag::Compound);
        self.current_name = variant.into();
        value.serialize(&mut *self)?;
        self.serialize_tag(Tag::End);
        Ok(Tag::End)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        // Don't serialize start of List just yet, might be an IntArray or LongArray instead.
        // We wait until the call to serialize the first element to figure it out.
        Ok(SerializerSeq::new(self, len))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        // There's no canonical way of encoding tuples, so we'll just make up one.
        // Serializes tuple as Compound where the name of each item is its index in the tuple.
        self.serialize_tag(Tag::Compound);
        Ok(SerializerTuple::new(self))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_tag(Tag::Compound);
        self.current_name = variant.into();
        self.serialize_tag(Tag::Compound);
        Ok(SerializerTuple::new(self).add_extra_end_tag())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.serialize_tag(Tag::Compound);
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_tag(Tag::Compound);
        self.current_name = variant.into();
        self.serialize_tag(Tag::Compound);
        Ok(self)
    }
}

pub struct SerializerSeq<'a, 'source> {
    serializer: &'a mut Serializer<'source>,
    list_len: Option<usize>,
    list_tag: Tag,
    list_elem_tag: Tag,
    list_len_pos: Option<usize>,
    list_len_count: i32,
}

impl<'a, 'source> SerializerSeq<'a, 'source> {
    fn new(serializer: &'a mut Serializer<'source>, list_len: Option<usize>) -> Self {
        Self {
            serializer,
            list_len,
            list_tag: Tag::List,
            list_elem_tag: Tag::End,
            list_len_pos: None,
            list_len_count: 0,
        }
    }
}

impl ser::SerializeSeq for SerializerSeq<'_, '_> {
    type Ok = Tag;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.list_elem_tag == Tag::End {
            // Figure out the type (tag) of this first element.
            let mut output_hold = Vec::<u8>::new();
            std::mem::swap(&mut self.serializer.output, &mut output_hold);
            let tag = value.serialize(&mut *self.serializer)?;
            self.list_elem_tag = tag;
            std::mem::swap(&mut self.serializer.output, &mut output_hold);

            // Properly start the list.
            match tag {
                Tag::Byte => {
                    self.list_tag = Tag::ByteArray;
                    self.serializer.serialize_tag(self.list_tag);
                }
                Tag::Int => {
                    self.list_tag = Tag::IntArray;
                    self.serializer.serialize_tag(self.list_tag);
                }
                Tag::Long => {
                    self.list_tag = Tag::LongArray;
                    self.serializer.serialize_tag(self.list_tag);
                }
                _ => {
                    self.list_tag = Tag::List;
                    self.serializer.serialize_tag(self.list_tag);
                    self.serializer.output.put_u8(tag.to_u8());
                }
            }
            match self.list_len {
                Some(list_len) => {
                    self.serializer.output.put_i32(list_len as i32);
                }
                None => {
                    // Fill it in later, when we know the length.
                    self.list_len_pos = Some(self.serializer.output.len());
                    self.serializer.output.put_i32(0);
                }
            }

            // Properly write the first element.
            self.serializer.output.append(&mut output_hold);
        } else {
            value.serialize(&mut *self.serializer)?;
        }

        self.list_len_count += 1;

        Ok(())
    }

    fn end(self) -> Result<Tag> {
        // Check if we need to fill in the length.
        if let Some(list_len_pos) = self.list_len_pos {
            let mut start = &mut self.serializer.output[list_len_pos..];
            start.put_i32(self.list_len_count);
        }
        Ok(self.list_tag)
    }
}

pub struct SerializerTuple<'a, 'source> {
    serializer: &'a mut Serializer<'source>,
    tuple_idx_count: usize,
    extra_end_tags: usize,
}

impl<'a, 'source> SerializerTuple<'a, 'source> {
    fn new(serializer: &'a mut Serializer<'source>) -> Self {
        Self {
            serializer,
            tuple_idx_count: 0,
            extra_end_tags: 0,
        }
    }

    fn add_extra_end_tag(mut self) -> Self {
        self.extra_end_tags += 1;
        self
    }
}

impl ser::SerializeTuple for SerializerTuple<'_, '_> {
    type Ok = Tag;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serializer.current_name = itoa::Buffer::new()
            .format(self.tuple_idx_count)
            .to_string()
            .into();
        value.serialize(&mut *self.serializer)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.serializer.serialize_tag(Tag::End);
        for _ in 0..self.extra_end_tags {
            self.serializer.serialize_tag(Tag::End);
        }
        Ok(Tag::Compound)
    }
}

impl ser::SerializeTupleStruct for SerializerTuple<'_, '_> {
    type Ok = Tag;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        use serde::ser::SerializeTuple;
        self.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok> {
        <Self as ser::SerializeTuple>::end(self)
    }
}

impl ser::SerializeTupleVariant for SerializerTuple<'_, '_> {
    type Ok = Tag;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        use serde::ser::SerializeTuple;
        self.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok> {
        <Self as ser::SerializeTuple>::end(self)
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer<'_> {
    type Ok = Tag;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer_name = SerializerName::default();
        key.serialize(&mut serializer_name)?;
        self.current_name = serializer_name.output.into();
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.serialize_tag(Tag::End);
        Ok(Tag::Compound)
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer<'_> {
    type Ok = Tag;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.current_name = key.into();
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.serialize_tag(Tag::End);
        Ok(Tag::Compound)
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer<'_> {
    type Ok = Tag;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        <Self as ser::SerializeStruct>::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.serialize_tag(Tag::End);
        self.serialize_tag(Tag::End);
        Ok(Tag::Compound)
    }
}

#[derive(Default)]
struct SerializerName {
    output: String,
}

impl SerializerName {
    fn err<T>(&mut self) -> Result<T> {
        Err(Error::Serde("todo".into()))
    }
}

impl<'a> ser::Serializer for &'a mut SerializerName {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_i8(self, __v: i8) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_i16(self, _v: i16) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_i32(self, _v: i32) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_i64(self, _v: i64) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_u8(self, _v: u8) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_u16(self, _v: u16) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_u32(self, _v: u32) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_u64(self, _v: u64) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_f32(self, _v: f32) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_f64(self, _v: f64) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_char(self, _v: char) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_str(self, v: &str) -> std::result::Result<Self::Ok, Self::Error> {
        self.output = v.to_string();
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_some<T>(self, _value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.err()
    }

    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.err()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.err()
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.err()
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        self.err()
    }

    fn serialize_tuple(
        self,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        self.err()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        self.err()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        self.err()
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        self.err()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        self.err()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        self.err()
    }
}
