use std::io::Read;

use bytes::Buf;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use thiserror::Error;

use crate::connection::ConnectionState;

const VARINT_SEGMENT_BITS: u8 = 0b01111111;
const VARINT_CONTINUE_BIT: u8 = 0b10000000;

pub trait BufMinecraft {
    fn get_varint(&mut self) -> Result<i32, VarIntError>;
    fn get_varint_with_at_most(&mut self, bytes: usize) -> Result<i32, VarIntError>;
    fn try_get_varint_with_at_most(&mut self, bytes: usize) -> Result<Option<i32>, VarIntError>;
    fn get_enum<T>(&mut self) -> PacketDecodeResult<T>
    where
        T: FromPrimitive;
    fn get_string<'a>(&mut self) -> Result<String, StringError>;
}

impl<B: Buf> BufMinecraft for B {
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

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Packet {
    Handshake {
        protocol_version: i32,
        server_address: String,
        server_port: u16,
        next_state: ConnectionState,
    } = 0x00,
    CustomReportDetails {
        details: Vec<ReportDetail>,
    } = 0x7A,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReportDetail {
    title: String,
    description: String,
}

impl Packet {
    pub fn check<B>(buf: &mut B) -> PacketDecodeResult<PacketCheckOutcome>
    where
        B: Buf + BufMinecraft,
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

    pub fn decode<B>(len: u16, packet_id: i32, buf: &mut B) -> PacketDecodeResult<Packet>
    where
        B: Buf + BufMinecraft,
    {
        match packet_id {
            0x00 => {
                let protocol_version = buf.get_varint()?;
                let server_address = buf.get_string()?;
                let server_port = buf.get_u16();
                let next_state = buf.get_enum()?;

                if ![
                    ConnectionState::Status,
                    ConnectionState::Login,
                    ConnectionState::Configuration,
                ]
                .contains(&next_state)
                {
                    return Err(PacketDecodeError::Specific("handshake: invalid next state"));
                }

                Ok(Packet::Handshake {
                    protocol_version,
                    server_address,
                    server_port,
                    next_state,
                })
            }
            _ => unimplemented!("packet id {:#04X}", packet_id),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        todo!()
    }
}

pub enum PacketCheckOutcome {
    Ok { len: u16, packet_id: i32 },
    Incomplete,
}

type PacketDecodeResult<T> = Result<T, PacketDecodeError>;

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
