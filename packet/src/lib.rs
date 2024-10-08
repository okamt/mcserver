use std::{borrow::Cow, convert::Infallible};

use bytes::Buf;
use client::ClientPacket;
use delegate_display::DelegateDebug;
use derive_more::derive::From;
use packet_derive::Packet;
use protocol::{
    buf::{self, IdentifierProtocolContext},
    identifier::Identifier,
    ConnectionState, Decodable, DecodeError, Encodable,
};
use protocol_derive::Protocol;
use server::ServerPacket;
use thiserror::Error;

pub mod client;
pub mod server;

pub trait Packet: Encodable<Context = (), Error = Infallible> + Decodable {
    fn get_id(&self) -> i32;
}

#[derive(From, DelegateDebug)]
pub enum AnyPacket<'a> {
    Client(ClientPacket<'a>),
    Server(ServerPacket<'a>),
}

impl<'a> Encodable for AnyPacket<'a> {
    type Context = ();
    type Error = Infallible;

    fn encode(
        &self,
        buf: &mut dyn bytes::BufMut,
        ctx: Self::Context,
    ) -> Result<(), buf::EncodeError<Self::Error>> {
        match self {
            AnyPacket::Client(packet) => packet.encode(buf, ctx),
            AnyPacket::Server(packet) => packet.encode(buf, ctx),
        }
    }
}

impl<'a> Decodable for AnyPacket<'a> {
    type Context = PacketDecodeContext;
    type Error = PacketDecodeError;

    fn decode(buf: &mut dyn Buf, ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        Ok(match ctx.direction {
            PacketDirection::Client => Self::Client(ClientPacket::decode(buf, ctx)?),
            PacketDirection::Server => Self::Server(ServerPacket::decode(buf, ctx)?),
        })
    }
}

impl<'a> Packet for AnyPacket<'a> {
    fn get_id(&self) -> i32 {
        match self {
            AnyPacket::Client(packet) => packet.get_id(),
            AnyPacket::Server(packet) => packet.get_id(),
        }
    }
}

#[derive(Debug)]
pub struct PacketDecodeContext {
    pub connection_state: ConnectionState,
    pub packet_id: i32,
    pub direction: PacketDirection,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PacketDirection {
    Client,
    Server,
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
    (
        $enum_name:ident $(<$($enum_gen:lifetime),*>)?
        $($name:ident $(<$($gen:lifetime),*>)? { $($(#[$meta:meta])? $field:ident : $ftype:ty),* $(,)? } = $discrim:expr)*
    ) => {
        $(
            #[derive(Debug, Clone, Eq, PartialEq, protocol_derive::Protocol)]
            pub struct $name $(<$($gen),*>)? {
                $($(#[$meta])? pub $field: $ftype),*
            }

            impl $(<$($gen),*>)? packet::Packet for $name $(<$($gen),*>)? {
                fn get_id(&self) -> i32 {
                    $discrim
                }
            }
        )*

        #[derive(DelegateDebug, Clone, Eq, PartialEq, derive_more::From, packet_derive::Packet)]
        #[repr(i32)]
        pub enum $enum_name $(<$($enum_gen),*>)? {
            $(
                $name($name $(<$($gen),*>)? ) = $discrim,
            )*
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Protocol)]
pub struct KnownPack<'a> {
    #[protocol(ctx = IdentifierProtocolContext::DoubleString)]
    pub identifier: Identifier<'a>,
    pub version: Cow<'a, str>,
}
