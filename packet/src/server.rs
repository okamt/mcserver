//! Serverbound packets.

use std::borrow::Cow;
use std::convert::Infallible;

use bytes::{Buf, BufMut};
use delegate_display::DelegateDebug;
use derive_more::derive::From;
use protocol::buf::ArrayProtocolContext;
use protocol::{
    identifier::Identifier, ChatMode, ConnectionState, Decodable, DisplayedSkinParts, Encodable,
    Hand,
};
use protocol::{ClientInformation, DecodeError, EncodeError};
use uuid::Uuid;

use crate::{packets, Packet, PacketDecodeContext, PacketDecodeError};

use crate as packet;

packets! {
    ServerHandshakingPacket<'a>

    HandshakePacket<'a> {
        #[protocol(varint)]
        protocol_version: i32,
        server_address: Cow<'a, str>,
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
    ServerLoginPacket<'a>

    LoginStartPacket<'a> {
        player_username: Cow<'a, str>,
        player_uuid: Uuid,
    } = 0x00
    EncryptionResponsePacket<'a> {
        #[protocol(ctx = ArrayProtocolContext::LengthPrefixed)]
        shared_secret: Cow<'a, [u8]>,
        #[protocol(ctx = ArrayProtocolContext::LengthPrefixed)]
        verify_token: Cow<'a, [u8]>,
    } = 0x01
    LoginPluginResponsePacket<'a> {
        #[protocol(varint)]
        message_id: i32,
        successful: bool,
        #[protocol(ctx = ArrayProtocolContext::Remaining)]
        data: Cow<'a, [u8]>,
    } = 0x02
    LoginAcknowledgedPacket {} = 0x03
}

packets! {
    ServerConfigurationPacket<'a>

    ClientInformationPacket<'a> {
        locale: Cow<'a, str>,
        view_distance: u8,
        chat_mode: ChatMode,
        chat_colors: bool,
        displayed_skin_parts: DisplayedSkinParts,
        main_hand: Hand,
        enable_text_filtering: bool,
        allow_server_listings: bool,
    } = 0x00
    ServerboundPluginMessagePacket<'a> {
        channel_identifier: Identifier,
        #[protocol(ctx = ArrayProtocolContext::Remaining)]
        data: Cow<'a, [u8]>,
    } = 0x02
}

impl<'a> Into<ClientInformation> for ClientInformationPacket<'a> {
    fn into(self) -> ClientInformation {
        ClientInformation {
            locale: self.locale.to_string(),
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

#[derive(DelegateDebug, Clone, Eq, PartialEq, From)]
pub enum ServerPacket<'a> {
    Handshaking(ServerHandshakingPacket<'a>),
    Status(ServerStatusPacket),
    Login(ServerLoginPacket<'a>),
    Configuration(ServerConfigurationPacket<'a>),
    Play(ServerPlayPacket),
}

impl<'a> Encodable for ServerPacket<'a> {
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

impl<'a> Decodable for ServerPacket<'a> {
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

impl<'a> Packet for ServerPacket<'a> {
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
