use std::convert::Infallible;

use bytes::Buf;
use protocol::{
    buf::{self},
    ConnectionState, Decodable, DecodeError, Encodable,
};
use thiserror::Error;

pub mod client;
pub mod server;

pub trait Packet: Encodable<Context = (), Error = Infallible> + Decodable {
    fn get_id(&self) -> i32;
}

#[derive(Debug)]
pub struct PacketDecodeContext {
    pub connection_state: ConnectionState,
    pub packet_id: i32,
}

#[derive(Debug, Error)]
pub enum PacketDecodeError {
    #[error("invalid packet id {0:#04X}")]
    InvalidPacketId(i32),
}

pub fn check_packet<B, E>(buf: &mut B) -> Result<PacketCheckOutcome, DecodeError<E>>
where
    B: Buf,
{
    if let Some(len) = buf::try_get_varint_with_at_most(buf, 3)? {
        if buf.remaining() < len.try_into().unwrap() {
            Ok(PacketCheckOutcome::Incomplete)
        } else {
            let packet_id = buf::get_varint(buf)?;
            Ok(PacketCheckOutcome::Ok {
                len: len.try_into().unwrap(),
                packet_id,
            })
        }
    } else {
        Ok(PacketCheckOutcome::Incomplete)
    }
}

#[derive(Debug)]
pub enum PacketCheckOutcome {
    Ok { len: u16, packet_id: i32 },
    Incomplete,
}

#[macro_export]
macro_rules! packets {
    ( $enum_name:ident $($name:ident { $($(#[$meta:meta])? $field:ident : $ftype:ty),* $(,)? } = $discrim:expr)* ) => {
        $(
            #[derive(Debug, Clone, Eq, PartialEq, protocol_derive::Protocol)]
            pub struct $name {
                $($(#[$meta])? pub $field: $ftype),*
            }

            impl packet::Packet for $name {
                fn get_id(&self) -> i32 {
                    $discrim
                }
            }
        )*

        #[derive(Debug, Clone, Eq, PartialEq, num_derive::ToPrimitive, derive_more::From, packet_derive::Packet)]
        #[repr(i32)]
        pub enum $enum_name {
            $(
                $name($name) = $discrim,
            )*
        }
    };
}
