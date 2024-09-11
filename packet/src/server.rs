//! Serverbound packets.

use std::convert::Infallible;

use bytes::{Buf, BufMut};
use derive_more::derive::From;
use protocol::buf::VecProtocolContext;
use protocol::{
    identifier::Identifier, ChatMode, ConnectionState, Decodable, DisplayedSkinParts, Encodable,
    Hand,
};
use protocol::{ClientInformation, DecodeError, EncodeError};
use uuid::Uuid;

use crate::{packets, Packet, PacketDecodeContext, PacketDecodeError};

use crate as packet;

packets! {
    ServerHandshakingPacket

    HandshakePacket {
        #[protocol(varint)]
        protocol_version: i32,
        server_address: String,
        server_port: u16,
        next_state: ConnectionState,
    } = 0x00
}

packets! {
    ServerStatusPacket

    StatusRequestPacket {} = 0x00
    PingRequestPacket { payload: i64 } = 0x01
}

packets! {
    ServerLoginPacket

    LoginStartPacket {
        player_username: String,
        player_uuid: Uuid,
    } = 0x00
    LoginAcknowledgedPacket {} = 0x03
}

packets! {
    ServerConfigurationPacket

    ClientInformationPacket {
        locale: String,
        view_distance: u8,
        chat_mode: ChatMode,
        chat_colors: bool,
        displayed_skin_parts: DisplayedSkinParts,
        main_hand: Hand,
        enable_text_filtering: bool,
        allow_server_listings: bool,
    } = 0x00
    ServerboundPluginMessagePacket {
        channel_identifier: Identifier,
        #[protocol(ctx = VecProtocolContext::Remaining)]
        data: Vec<u8>,
    } = 0x02
}

impl Into<ClientInformation> for ClientInformationPacket {
    fn into(self) -> ClientInformation {
        ClientInformation {
            locale: self.locale,
            view_distance: self.view_distance,
            chat_mode: self.chat_mode,
            chat_colors: self.chat_colors,
            displayed_skin_parts: self.displayed_skin_parts,
            main_hand: self.main_hand,
            enable_text_filtering: self.enable_text_filtering,
            allow_server_listings: self.allow_server_listings,
        }
    }
}

packets! {
    ServerPlayPacket

    None1 {} = 0x00
}

#[derive(Debug, Clone, Eq, PartialEq, From)]
pub enum ServerPacket {
    Handshaking(ServerHandshakingPacket),
    Status(ServerStatusPacket),
    Login(ServerLoginPacket),
    Configuration(ServerConfigurationPacket),
    Play(ServerPlayPacket),
}

impl Encodable for ServerPacket {
    type Context = ();
    type Error = Infallible;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>> {
        match self {
            Self::Handshaking(packet) => packet.encode(buf, ctx),
            Self::Status(packet) => packet.encode(buf, ctx),
            Self::Login(packet) => packet.encode(buf, ctx),
            Self::Configuration(packet) => packet.encode(buf, ctx),
            Self::Play(packet) => packet.encode(buf, ctx),
        }
    }
}

impl Decodable for ServerPacket {
    type Context = PacketDecodeContext;
    type Error = PacketDecodeError;

    fn decode(buf: &mut dyn Buf, ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        Ok(match ctx.connection_state {
            ConnectionState::Handshaking => {
                Self::Handshaking(ServerHandshakingPacket::decode(buf, ctx)?)
            }
            ConnectionState::Status => Self::Status(ServerStatusPacket::decode(buf, ctx)?),
            ConnectionState::Login => Self::Login(ServerLoginPacket::decode(buf, ctx)?),
            ConnectionState::Configuration => {
                Self::Configuration(ServerConfigurationPacket::decode(buf, ctx)?)
            }
            ConnectionState::Play => Self::Play(ServerPlayPacket::decode(buf, ctx)?),
        })
    }
}

impl Packet for ServerPacket {
    fn get_id(&self) -> i32 {
        match self {
            Self::Handshaking(packet) => packet.get_id(),
            Self::Status(packet) => packet.get_id(),
            Self::Login(packet) => packet.get_id(),
            Self::Configuration(packet) => packet.get_id(),
            Self::Play(packet) => packet.get_id(),
        }
    }
}
