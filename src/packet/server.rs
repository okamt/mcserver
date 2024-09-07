//! Serverbound packets.

use crate::{
    packet::ambassador_impl_Packet,
    types::{ChatMode, DisplayedSkinParts, Hand, Identifier},
};
use ambassador::Delegate;
use derive_more::derive::From;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use uuid::Uuid;

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

                Self::Handshake {
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
            _ => unimplemented!("server status state packet id {:#04X}", packet_id),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ToPrimitive)]
#[repr(i32)]
pub enum ServerLoginPacket {
    LoginStart {
        player_username: String,
        player_uuid: Uuid,
    } = 0x00,
    LoginAcknowledged = 0x03,
}

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
        Ok(match packet_id {
            0x00 => {
                let player_username = buf.get_string()?;
                let player_uuid = buf.get_uuid();

                Self::LoginStart {
                    player_username,
                    player_uuid,
                }
            }
            0x03 => Self::LoginAcknowledged,
            _ => unimplemented!("server login state packet id {:#04X}", packet_id),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ToPrimitive)]
#[repr(i32)]
pub enum ServerConfigurationPacket {
    ClientInformation {
        locale: String,
        view_distance: u8,
        chat_mode: ChatMode,
        chat_colors: bool,
        displayed_skin_parts: DisplayedSkinParts,
        main_hand: Hand,
        enable_text_filtering: bool,
        allow_server_listings: bool,
    } = 0x00,
    ServerboundPluginMessage {
        channel_identifier: Identifier,
        data: Vec<u8>,
    } = 0x02,
}

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
        Ok(match packet_id {
            0x00 => {
                let locale = buf.get_string()?;
                let view_distance = buf.get_u8();
                let chat_mode = buf.get_enum()?;
                let chat_colors = buf.get_bool();
                let displayed_skin_parts = DisplayedSkinParts::from_bits(buf.get_u8()).unwrap();
                let main_hand = buf.get_enum()?;
                let enable_text_filtering = buf.get_bool();
                let allow_server_listings = buf.get_bool();

                Self::ClientInformation {
                    locale,
                    view_distance,
                    chat_mode,
                    chat_colors,
                    displayed_skin_parts,
                    main_hand,
                    enable_text_filtering,
                    allow_server_listings,
                }
            }
            0x02 => {
                let channel_identifier = buf.get_identifier()?;
                let data = buf.get_byte_array();

                Self::ServerboundPluginMessage {
                    channel_identifier,
                    data,
                }
            }
            _ => unimplemented!("server configuration state packet id {:#04X}", packet_id),
        })
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
        Ok(match packet_id {
            _ => unimplemented!("server play state packet id {:#04X}", packet_id),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, From, Delegate)]
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
