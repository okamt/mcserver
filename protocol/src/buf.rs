use bytes::{Buf, BufMut};

use std::{borrow::Cow, convert::Infallible, io::Read};

use num_enum::TryFromPrimitive;
use thiserror::Error;
use uuid::Uuid;

use crate::identifier::{Identifier, IdentifierError};

const VARINT_SEGMENT_BITS: u8 = 0b01111111;
const VARINT_CONTINUE_BIT: u8 = 0b10000000;

pub trait Encodable {
    type Context;
    type Error;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>>;
}

pub trait Decodable {
    type Context;
    type Error;

    fn decode(buf: &mut dyn Buf, ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized;
}

#[derive(Error, Debug)]
pub enum EncodeError<E> {
    #[error("error while serializing JSON: {0}")]
    JSON(#[from] serde_json::Error),
    #[error(transparent)]
    Other(E),
}

#[derive(Error, Debug)]
pub enum DecodeError<E> {
    #[error("{0}")]
    Specific(&'static str),
    #[error(transparent)]
    VarInt(#[from] GetVarIntError),
    #[error(transparent)]
    Enum(#[from] GetEnumError),
    #[error(transparent)]
    String(#[from] GetStringError),
    #[error(transparent)]
    Identifier(#[from] GetIdentifierError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Other(E),
}

impl EncodeError<Infallible> {
    pub fn expand<E>(self) -> EncodeError<E> {
        match self {
            EncodeError::JSON(e) => EncodeError::JSON(e),
            EncodeError::Other(_) => unreachable!(),
        }
    }
}

impl DecodeError<Infallible> {
    pub fn expand<E>(self) -> DecodeError<E> {
        match self {
            DecodeError::Specific(e) => DecodeError::Specific(e),
            DecodeError::VarInt(e) => DecodeError::VarInt(e),
            DecodeError::Enum(e) => DecodeError::Enum(e),
            DecodeError::String(e) => DecodeError::String(e),
            DecodeError::Identifier(e) => DecodeError::Identifier(e),
            DecodeError::Json(e) => DecodeError::Json(e),
            DecodeError::Other(_) => unreachable!(),
        }
    }
}

pub trait ResultExpandExt<T> {
    type Error<E>;

    fn expand<E>(self) -> Result<T, Self::Error<E>>;
}

impl<T> ResultExpandExt<T> for Result<T, EncodeError<Infallible>> {
    type Error<E> = EncodeError<E>;

    fn expand<E>(self) -> Result<T, Self::Error<E>> {
        self.map_err(|e| e.expand())
    }
}

impl<T> ResultExpandExt<T> for Result<T, DecodeError<Infallible>> {
    type Error<E> = DecodeError<E>;

    fn expand<E>(self) -> Result<T, Self::Error<E>> {
        self.map_err(|e| e.expand())
    }
}

macro_rules! basic_encode_decode {
    ( $($ty:ty => $e:path, $d:path;)* ) => {
        $(
            impl Encodable for $ty {
                type Context = ();
                type Error = Infallible;

                fn encode(&self, buf: &mut dyn BufMut, _ctx: Self::Context) -> Result<(), EncodeError<Self::Error>> {
                    $e(buf, *self);
                    Ok(())
                }
            }

            impl Decodable for $ty {
                type Context = ();
                type Error = Infallible;

                fn decode(buf: &mut dyn Buf, _ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
                where
                    Self: Sized,
                {
                    Ok($d(buf))
                }
            }
        )*
    };
}

basic_encode_decode! {
    u8 => BufMut::put_u8, Buf::get_u8;
    u16 => BufMut::put_u16, Buf::get_u16;
    u32 => BufMut::put_u32, Buf::get_u32;
    u64 => BufMut::put_u64, Buf::get_u64;
    u128 => BufMut::put_u128, Buf::get_u128;

    i8 => BufMut::put_i8, Buf::get_i8;
    i16 => BufMut::put_i16, Buf::get_i16;
    i32 => BufMut::put_i32, Buf::get_i32;
    i64 => BufMut::put_i64, Buf::get_i64;
    i128 => BufMut::put_i128, Buf::get_i128;
}

impl Encodable for bool {
    type Context = ();
    type Error = Infallible;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        _ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>> {
        put_bool(buf, *self);
        Ok(())
    }
}

impl Decodable for bool {
    type Context = ();
    type Error = Infallible;

    fn decode(buf: &mut dyn Buf, _ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        Ok(get_bool(buf))
    }
}

impl<'a> Encodable for Cow<'a, str> {
    type Context = ();
    type Error = Infallible;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        _ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>> {
        put_string(buf, self);
        Ok(())
    }
}

impl<'a> Decodable for Cow<'a, str> {
    type Context = ();
    type Error = Infallible;

    fn decode(buf: &mut dyn Buf, _ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        Ok(get_string(buf)?.into())
    }
}

// Temporarily disabled to make sure we use Cow<str> instead.
/*impl Encodable for String {
    type Context = ();
    type Error = Infallible;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        _ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>> {
        put_string(buf, self);
        Ok(())
    }
}

impl Decodable for String {
    type Context = ();
    type Error = Infallible;

    fn decode(buf: &mut dyn Buf, _ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        Ok(get_string(buf)?)
    }
}*/

impl Encodable for Uuid {
    type Context = ();
    type Error = Infallible;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        _ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>> {
        buf.put_u128(self.as_u128());
        Ok(())
    }
}

impl Decodable for Uuid {
    type Context = ();
    type Error = Infallible;

    fn decode(buf: &mut dyn Buf, _ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        Ok(Uuid::from_u128(buf.get_u128()))
    }
}

impl Encodable for Identifier<'_> {
    type Context = IdentifierProtocolContext;
    type Error = Infallible;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>> {
        match ctx {
            IdentifierProtocolContext::SingleString => put_identifier(buf, self),
            IdentifierProtocolContext::DoubleString => {
                put_string(buf, &self.namespace());
                put_string(buf, &self.value());
            }
        }
        Ok(())
    }
}

impl Decodable for Identifier<'_> {
    type Context = IdentifierProtocolContext;
    type Error = Infallible;

    fn decode(buf: &mut dyn Buf, ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        Ok(get_identifier(buf)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierProtocolContext {
    /// Identifier is encoded as one string.
    SingleString,
    /// Identifier is encoded as two consecutive strings.
    DoubleString,
}

// Temporarily disabled to make sure we use Cow<[T]> instead.
/*impl<T: Encodable<Context = ()>> Encodable for Vec<T> {
    type Context = ArrayProtocolContext;
    type Error = <T as Encodable>::Error;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        ctx: Self::Context,
    ) -> Result<(), EncodeError<<T as Encodable>::Error>> {
        match ctx {
            ArrayProtocolContext::LengthPrefixed => put_varint(buf, self.len().try_into().unwrap()),
            ArrayProtocolContext::Remaining => {}
            ArrayProtocolContext::FixedLength(_) => {}
        }
        for item in self {
            item.encode(buf, ())?;
        }
        Ok(())
    }
}

impl<T: Decodable<Context = ()>> Decodable for Vec<T> {
    type Context = ArrayProtocolContext;
    type Error = <T as Decodable>::Error;

    fn decode(buf: &mut dyn Buf, ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        let len = match ctx {
            ArrayProtocolContext::Remaining => buf.remaining(),
            ArrayProtocolContext::LengthPrefixed => get_varint(buf)?.try_into().unwrap(),
            ArrayProtocolContext::FixedLength(len) => len,
        };
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(<T as Decodable>::decode(buf, ())?);
        }
        Ok(vec)
    }
}*/

impl<'a, T: Encodable<Context = ()>> Encodable for Cow<'a, [T]>
where
    [T]: ToOwned,
{
    type Context = ArrayProtocolContext;
    type Error = <T as Encodable>::Error;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>> {
        match ctx {
            ArrayProtocolContext::LengthPrefixed => put_varint(buf, self.len().try_into().unwrap()),
            ArrayProtocolContext::Remaining => {}
            ArrayProtocolContext::FixedLength(_) => {}
        }
        for item in self.iter() {
            item.encode(buf, ())?;
        }
        Ok(())
    }
}

impl<'a, T: Decodable<Context = ()>> Decodable for Cow<'a, [T]>
where
    [T]: ToOwned,
    <[T] as ToOwned>::Owned: From<Vec<T>>, // <[T] as ToOwned>::Owned can only ever be Vec<T> (https://github.com/rust-lang/rust/issues/20041)
{
    type Context = ArrayProtocolContext;
    type Error = <T as Decodable>::Error;

    fn decode(buf: &mut dyn Buf, ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        let len = match ctx {
            ArrayProtocolContext::Remaining => buf.remaining(),
            ArrayProtocolContext::LengthPrefixed => get_varint(buf)?.try_into().unwrap(),
            ArrayProtocolContext::FixedLength(len) => len,
        };
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(<T as Decodable>::decode(buf, ())?);
        }
        Ok(Cow::Owned(vec.into()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayProtocolContext {
    /// Vec length is based on the remaining byte count.
    Remaining,
    /// Vec is prefixed by a VarInt length value.
    LengthPrefixed,
    /// Vec is a known, fixed length.
    FixedLength(usize),
}

impl<T: Encodable<Context = ()>> Encodable for Option<T> {
    type Context = OptionProtocolContext;
    type Error = <T as Encodable>::Error;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>> {
        match ctx {
            OptionProtocolContext::BoolPrefixed => put_bool(buf, self.is_some()),
            OptionProtocolContext::Remaining => {}
        }
        match self {
            Some(value) => value.encode(buf, ()),
            None => Ok(()),
        }
    }
}

impl<T: Decodable<Context = ()>> Decodable for Option<T> {
    type Context = OptionProtocolContext;
    type Error = <T as Decodable>::Error;

    fn decode(buf: &mut dyn Buf, ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        let is_present = match ctx {
            OptionProtocolContext::BoolPrefixed => get_bool(buf),
            OptionProtocolContext::Remaining => buf.has_remaining(),
        };
        match is_present {
            true => <T as Decodable>::decode(buf, ()).map(Some),
            false => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionProtocolContext {
    /// Option is present based on the remaining byte count.
    Remaining,
    /// Option is prefixed by a bool, is present if true.
    BoolPrefixed,
}

pub fn get_varint<B: Buf + ?Sized>(buf: &mut B) -> Result<i32, GetVarIntError> {
    get_varint_with_at_most(buf, 4)
}

pub fn get_varint_with_at_most<B: Buf + ?Sized>(
    buf: &mut B,
    bytes: usize,
) -> Result<i32, GetVarIntError> {
    let mut value = 0;
    let mut position = 0;

    loop {
        let byte = buf.get_u8();
        value |= ((byte & VARINT_SEGMENT_BITS) as i32) << position;

        if byte & VARINT_CONTINUE_BIT == 0 {
            break;
        }

        position += 7;

        if position >= bytes * 8 {
            return Err(GetVarIntError::TooBig);
        }
    }

    Ok(value)
}

pub fn try_get_varint_with_at_most<B: Buf + ?Sized>(
    buf: &mut B,
    bytes: usize,
) -> Result<Option<i32>, GetVarIntError> {
    let mut value = 0;
    let mut position = 0;

    loop {
        if buf.remaining() < 1 {
            return Ok(None);
        }
        let byte = buf.get_u8();
        value |= ((byte & VARINT_SEGMENT_BITS) as i32) << position;

        if byte & VARINT_CONTINUE_BIT == 0 {
            break;
        }

        position += 7;

        if position >= bytes * 8 {
            return Err(GetVarIntError::TooBig);
        }
    }

    Ok(Some(value))
}

pub fn get_enum<B: Buf + ?Sized, T>(buf: &mut B) -> Result<T, GetEnumError>
where
    T: TryFromPrimitive<Primitive = i32>,
{
    let varint = get_varint(buf)?;
    <T as TryFromPrimitive>::try_from_primitive(varint)
        .map_err(|_| GetEnumError::InvalidValue(varint))
}

pub fn get_string<B: Buf + ?Sized>(buf: &mut B) -> Result<String, GetStringError> {
    let len = get_varint(buf)?.try_into().unwrap();
    let mut string = String::with_capacity(len);
    buf.take(len).reader().read_to_string(&mut string)?;
    Ok(string)
}

pub fn get_uuid<B: Buf + ?Sized>(buf: &mut B) -> Uuid {
    Uuid::from_u128(buf.get_u128())
}

pub fn get_bool<B: Buf + ?Sized>(buf: &mut B) -> bool {
    buf.get_u8() == 0x01
}

pub fn get_identifier<B: Buf + ?Sized>(
    buf: &mut B,
) -> Result<Identifier<'static>, GetIdentifierError> {
    Ok(get_string(buf)?.try_into()?)
}

pub fn get_unsized_byte_array<B: Buf + ?Sized>(buf: &mut B) -> Vec<u8> {
    let mut vec = Vec::with_capacity(buf.remaining());
    while buf.has_remaining() {
        vec.push(buf.get_u8());
    }
    vec
}

pub fn get_sized_byte_array<B: Buf + ?Sized>(buf: &mut B) -> Result<Vec<u8>, GetVarIntError> {
    let len = get_varint(buf)?;
    let mut vec = Vec::with_capacity(len.try_into().unwrap());
    for _ in 0..len {
        vec.push(buf.get_u8());
    }
    Ok(vec)
}

pub fn put_varint<B: BufMut + ?Sized>(buf: &mut B, mut varint: i32) {
    loop {
        if varint & !(i32::from(VARINT_SEGMENT_BITS)) == 0 {
            buf.put_u8((varint & 0xFF) as u8);
            return;
        }

        buf.put_u8((((varint & 0xFF) as u8) & VARINT_SEGMENT_BITS) | VARINT_CONTINUE_BIT);

        varint = ((varint as u32) >> 7) as i32;
    }
}

pub fn put_enum<B: BufMut + ?Sized>(buf: &mut B, value: impl Into<i32>) {
    put_varint(buf, value.into());
}

pub fn put_string<B: BufMut + ?Sized>(buf: &mut B, string: &dyn AsRef<str>) {
    put_varint(buf, string.as_ref().len().try_into().unwrap());
    buf.put_slice(string.as_ref().as_bytes());
}

pub fn put_uuid<B: BufMut + ?Sized>(buf: &mut B, uuid: &Uuid) {
    buf.put_u128(uuid.as_u128());
}

pub fn put_bool<B: BufMut + ?Sized>(buf: &mut B, boolean: bool) {
    buf.put_u8(match boolean {
        true => 0x01,
        false => 0x00,
    })
}

pub fn put_identifier<B: BufMut + ?Sized>(buf: &mut B, identifier: &Identifier) {
    let namespace = identifier.namespace();
    let value = identifier.value();
    put_varint(buf, (namespace.len() + 1 + value.len()).try_into().unwrap());
    buf.put_slice(namespace.as_bytes());
    buf.put_slice(b":");
    buf.put_slice(value.as_bytes());
}

pub fn put_unsized_byte_array<B: BufMut + ?Sized>(buf: &mut B, byte_array: &[u8]) {
    buf.put_slice(byte_array);
}

pub fn put_sized_byte_array<B: BufMut + ?Sized>(buf: &mut B, byte_array: &[u8]) {
    put_varint(buf, byte_array.len().try_into().unwrap());
    buf.put_slice(byte_array);
}

#[derive(Error, Debug)]
pub enum GetVarIntError {
    #[error("varint is too big")]
    TooBig,
}

#[derive(Error, Debug)]
pub enum GetStringError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    VarInt(#[from] GetVarIntError),
}

#[derive(Error, Debug)]
pub enum GetEnumError {
    #[error("invalid enum value: {0}")]
    InvalidValue(i32),
    #[error(transparent)]
    VarInt(#[from] GetVarIntError),
}

#[derive(Error, Debug)]
pub enum GetIdentifierError {
    #[error(transparent)]
    String(#[from] GetStringError),
    #[error(transparent)]
    Identifier(#[from] IdentifierError),
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn varint() {
        let mut buf = BytesMut::new();

        let tests: &[(i32, &[u8])] = &[
            (0, &[0x00]),
            (1, &[0x01]),
            (127, &[0x7F]),
            (128, &[0x80, 0x01]),
            (255, &[0xFF, 0x01]),
            (25565, &[0xDD, 0xC7, 0x01]),
            (2097151, &[0xFF, 0xFF, 0x7F]),
            (2147483647, &[0xFF, 0xFF, 0xFF, 0xFF, 0x07]),
            (-1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
            (-2147483648, &[0x80, 0x80, 0x80, 0x80, 0x08]),
        ];

        for &(varint, bytes) in tests {
            put_varint(&mut buf, varint);
            assert_eq!(&buf[0..bytes.len()], bytes);
            assert_eq!(
                get_varint(&mut (&buf[0..]).copy_to_bytes(bytes.len())).unwrap(),
                varint
            );
            buf.clear();
        }
    }
}
