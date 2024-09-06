//! Serverbound packets.

use crate::packet::ambassador_impl_Packet;
use ambassador::Delegate;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

use crate::connection::ConnectionState;

use super::{
    BufExt, Packet, PacketDecodeError, PacketDecodeResult, PacketDecoder, PacketEncodeResult,
};

#[derive(Debug, Clone, Eq, PartialEq, ToPrimitive)]
#[repr(i32)]
pub enum ServerHandshakingPacket {
    Handshake {
        protocol_version: i32,
        server_address: String,
        server_port: u16,
        next_state: ConnectionState,
    } = 0x00,
}

impl Packet for ServerHandshakingPacket {
    fn get_id(&self) -> i32 {
        self.to_i32().unwrap()
    }

    fn encode(&self) -> PacketEncodeResult<Vec<u8>> {
        todo!()
    }
}

impl PacketDecoder for ServerHandshakingPacket {
    fn decode<B>(
        connection_state: ConnectionState,
        len: u16,
        packet_id: i32,
        buf: &mut B,
    ) -> PacketDecodeResult<Self>
    where
        B: BufExt,
    {
        Ok(match packet_id {
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

                ServerHandshakingPacket::Handshake {
                    protocol_version,
                    server_address,
                    server_port,
                    next_state,
                }
            }
            _ => unimplemented!("handshaking state packet id {:#04X}", packet_id),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ToPrimitive)]
#[repr(i32)]
pub enum ServerStatusPacket {
    StatusRequest = 0x00,
    PingRequest { payload: i64 } = 0x01,
}

impl Packet for ServerStatusPacket {
    fn get_id(&self) -> i32 {
        self.to_i32().unwrap()
    }

    fn encode(&self) -> PacketEncodeResult<Vec<u8>> {
        todo!()
    }
}

impl PacketDecoder for ServerStatusPacket {
    fn decode<B>(
        connection_state: ConnectionState,
        len: u16,
        packet_id: i32,
        buf: &mut B,
    ) -> PacketDecodeResult<Self>
    where
        B: BufExt,
    {
        Ok(match packet_id {
            0x00 => ServerStatusPacket::StatusRequest,
            0x01 => {
                let payload = buf.get_i64();

                ServerStatusPacket::PingRequest { payload }
            }
            _ => unimplemented!("status state packet id {:#04X}", packet_id),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ToPrimitive)]
//#[repr(i32)]
pub enum ServerLoginPacket {}

impl Packet for ServerLoginPacket {
    fn get_id(&self) -> i32 {
        self.to_i32().unwrap()
    }

    fn encode(&self) -> PacketEncodeResult<Vec<u8>> {
        todo!()
    }
}

impl PacketDecoder for ServerLoginPacket {
    fn decode<B>(
        connection_state: ConnectionState,
        len: u16,
        packet_id: i32,
        buf: &mut B,
    ) -> PacketDecodeResult<Self>
    where
        B: BufExt,
    {
        todo!()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ToPrimitive)]
//#[repr(i32)]
pub enum ServerConfigurationPacket {}

impl Packet for ServerConfigurationPacket {
    fn get_id(&self) -> i32 {
        self.to_i32().unwrap()
    }

    fn encode(&self) -> PacketEncodeResult<Vec<u8>> {
        todo!()
    }
}

impl PacketDecoder for ServerConfigurationPacket {
    fn decode<B>(
        connection_state: ConnectionState,
        len: u16,
        packet_id: i32,
        buf: &mut B,
    ) -> PacketDecodeResult<Self>
    where
        B: BufExt,
    {
        todo!()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ToPrimitive)]
//#[repr(i32)]
pub enum ServerPlayPacket {}

impl Packet for ServerPlayPacket {
    fn get_id(&self) -> i32 {
        self.to_i32().unwrap()
    }

    fn encode(&self) -> PacketEncodeResult<Vec<u8>> {
        todo!()
    }
}

impl PacketDecoder for ServerPlayPacket {
    fn decode<B>(
        connection_state: ConnectionState,
        len: u16,
        packet_id: i32,
        buf: &mut B,
    ) -> PacketDecodeResult<Self>
    where
        B: BufExt,
    {
        todo!()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Delegate)]
#[delegate(Packet)]
pub enum ServerPacket {
    Handshaking(ServerHandshakingPacket),
    Status(ServerStatusPacket),
    Login(ServerLoginPacket),
    Configuration(ServerConfigurationPacket),
    Play(ServerPlayPacket),
}

impl PacketDecoder for ServerPacket {
    fn decode<B>(
        connection_state: ConnectionState,
        len: u16,
        packet_id: i32,
        buf: &mut B,
    ) -> PacketDecodeResult<Self>
    where
        B: BufExt,
    {
        Ok(match connection_state {
            ConnectionState::Handshaking => Self::Handshaking(ServerHandshakingPacket::decode(
                connection_state,
                len,
                packet_id,
                buf,
            )?),
            ConnectionState::Status => Self::Status(ServerStatusPacket::decode(
                connection_state,
                len,
                packet_id,
                buf,
            )?),
            ConnectionState::Login => Self::Login(ServerLoginPacket::decode(
                connection_state,
                len,
                packet_id,
                buf,
            )?),
            ConnectionState::Configuration => Self::Configuration(
                ServerConfigurationPacket::decode(connection_state, len, packet_id, buf)?,
            ),
            ConnectionState::Play => Self::Play(ServerPlayPacket::decode(
                connection_state,
                len,
                packet_id,
                buf,
            )?),
        })
    }
}
