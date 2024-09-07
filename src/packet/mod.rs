use std::io::Read;

use ambassador::delegatable_trait;
use bytes::{Buf, BufMut};
use num_traits::{FromPrimitive, ToPrimitive};
use thiserror::Error;
use uuid::Uuid;

use crate::connection::ConnectionState;

const VARINT_SEGMENT_BITS: u8 = 0b01111111;
const VARINT_CONTINUE_BIT: u8 = 0b10000000;

pub mod client;
pub mod server;

pub trait BufExt: Buf {
    fn get_varint(&mut self) -> Result<i32, VarIntError>;
    fn get_varint_with_at_most(&mut self, bytes: usize) -> Result<i32, VarIntError>;
    fn try_get_varint_with_at_most(&mut self, bytes: usize) -> Result<Option<i32>, VarIntError>;
    fn get_enum<T>(&mut self) -> PacketDecodeResult<T>
    where
        T: FromPrimitive;
    fn get_string(&mut self) -> Result<String, StringError>;
    fn get_uuid(&mut self) -> Uuid;
    fn get_bool(&mut self) -> bool;
}

pub trait BufMutExt: BufMut {
    fn put_varint(&mut self, varint: i32);
    fn put_enum<T>(&mut self, value: T)
    where
        T: ToPrimitive;
    fn put_string<S>(&mut self, string: S)
    where
        S: AsRef<str>;
    fn put_uuid(&mut self, uuid: &Uuid);
    fn put_bool(&mut self, boolean: bool);
}

impl<B: Buf> BufExt for B {
    fn get_varint(&mut self) -> Result<i32, VarIntError> {
        self.get_varint_with_at_most(4)
    }

    fn get_varint_with_at_most(&mut self, bytes: usize) -> Result<i32, VarIntError> {
        let mut value = 0;
        let mut position = 0;

        loop {
            let byte = self.get_u8();
            value |= ((byte & VARINT_SEGMENT_BITS) as i32) << position;

            if byte & VARINT_CONTINUE_BIT == 0 {
                break;
            }

            position += 7;

            if position >= bytes * 8 {
                return Err(VarIntError::TooBig);
            }
        }

        Ok(value)
    }

    fn try_get_varint_with_at_most(&mut self, bytes: usize) -> Result<Option<i32>, VarIntError> {
        let mut value = 0;
        let mut position = 0;

        loop {
            if self.remaining() < 1 {
                return Ok(None);
            }
            let byte = self.get_u8();
            value |= ((byte & VARINT_SEGMENT_BITS) as i32) << position;

            if byte & VARINT_CONTINUE_BIT == 0 {
                break;
            }

            position += 7;

            if position >= bytes * 8 {
                return Err(VarIntError::TooBig);
            }
        }

        Ok(Some(value))
    }

    fn get_enum<T>(&mut self) -> PacketDecodeResult<T>
    where
        T: FromPrimitive,
    {
        let varint = self.get_varint()?;
        FromPrimitive::from_i32(varint).ok_or(PacketDecodeError::InvalidEnumValue(varint))
    }

    fn get_string(&mut self) -> Result<String, StringError> {
        let len = self.get_varint()?.try_into().unwrap();
        let mut string = String::with_capacity(len);
        self.take(len).reader().read_to_string(&mut string)?;
        Ok(string)
    }

    fn get_uuid(&mut self) -> Uuid {
        Uuid::from_u128(self.get_u128())
    }

    fn get_bool(&mut self) -> bool {
        self.get_u8() == 0x01
    }
}

impl<B: BufMut> BufMutExt for B {
    fn put_varint(&mut self, mut varint: i32) {
        loop {
            if varint & !(i32::from(VARINT_SEGMENT_BITS)) == 0 {
                self.put_u8((varint & 0xFF) as u8);
                return;
            }

            self.put_u8((((varint & 0xFF) as u8) & VARINT_SEGMENT_BITS) | VARINT_CONTINUE_BIT);

            varint = ((varint as u32) >> 7) as i32;
        }
    }

    fn put_enum<T>(&mut self, value: T)
    where
        T: ToPrimitive,
    {
        todo!()
    }

    fn put_string<S>(&mut self, string: S)
    where
        S: AsRef<str>,
    {
        self.put_varint(string.as_ref().len().try_into().unwrap());
        self.put_slice(string.as_ref().as_bytes());
    }

    fn put_uuid(&mut self, uuid: &Uuid) {
        self.put_u128(uuid.as_u128());
    }

    fn put_bool(&mut self, boolean: bool) {
        self.put_u8(match boolean {
            true => 0x01,
            false => 0x00,
        })
    }
}

#[derive(Error, Debug)]
pub enum VarIntError {
    #[error("varint is too big")]
    TooBig,
}

#[derive(Error, Debug)]
pub enum StringError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    VarInt(#[from] VarIntError),
}

pub fn check_packet<B>(buf: &mut B) -> PacketDecodeResult<PacketCheckOutcome>
where
    B: BufExt,
{
    if let Some(len) = buf.try_get_varint_with_at_most(3)? {
        if buf.remaining() < len.try_into().unwrap() {
            Ok(PacketCheckOutcome::Incomplete)
        } else {
            let packet_id = buf.get_varint()?;
            Ok(PacketCheckOutcome::Ok {
                len: len.try_into().unwrap(),
                packet_id,
            })
        }
    } else {
        Ok(PacketCheckOutcome::Incomplete)
    }
}

#[delegatable_trait]
pub trait Packet {
    fn get_id(&self) -> i32;
    fn encode(&self) -> PacketEncodeResult<Vec<u8>>;
}

pub trait PacketDecoder
where
    Self: Sized,
{
    fn decode<B>(
        connection_state: ConnectionState,
        len: u16,
        packet_id: i32,
        buf: &mut B,
    ) -> PacketDecodeResult<Self>
    where
        B: BufExt;
}

pub enum PacketCheckOutcome {
    Ok { len: u16, packet_id: i32 },
    Incomplete,
}

pub type PacketDecodeResult<T> = Result<T, PacketDecodeError>;

#[derive(Error, Debug)]
pub enum PacketDecodeError {
    #[error("wrong opcode (expected {expected:#04X}, found {found:#04X})")]
    WrongOpcode { expected: u8, found: u8 },
    #[error("invalid enum value {0}")]
    InvalidEnumValue(i32),
    #[error("packet specific error: {0}")]
    Specific(&'static str),
    #[error(transparent)]
    VarInt(#[from] VarIntError),
    #[error(transparent)]
    String(#[from] StringError),
}

pub type PacketEncodeResult<T> = Result<T, PacketEncodeError>;

#[derive(Error, Debug)]
pub enum PacketEncodeError {
    #[error("error while serializing JSON: {0}")]
    JSON(#[from] serde_json::Error),
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
            buf.put_varint(varint);
            assert_eq!(&buf[0..bytes.len()], bytes);
            assert_eq!((&buf[0..bytes.len()]).get_varint().unwrap(), varint);
            buf.clear();
        }
    }
}
