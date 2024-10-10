//! Clientbound packets.

use std::convert::Infallible;

use crate::{KnownPack, Packet, PacketDecodeContext, PacketDecodeError};
use bytes::{Buf, BufMut};
use delegate_display::DelegateDebug;
use derive_more::derive::From;
use protocol::{
    buf::{ArrayProtocolContext, IdentifierProtocolContext, OptionProtocolContext},
    identifier::Identifier,
    Decodable, DecodeError, Encodable, EncodeError,
};
use protocol_derive::Protocol;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use uuid::Uuid;

use protocol::ConnectionState;

use crate::packets;

use super::buf;

use crate as packet;

packets! {
    ClientStatusPacket

    StatusResponsePacket { response: StatusResponse } = 0x00
    PongResponsePacket { payload: i64 } = 0x01
}

packets! {
    ClientLoginPacket<'a>

    LoginDisconnectPacket {} = 0x00
    EncryptionRequestPacket<'a> {
        server_id: Cow<'a, str>,
        #[protocol(ctx = ArrayProtocolContext::LengthPrefixed)]
        public_key: Cow<'a, [u8]>,
        #[protocol(ctx = ArrayProtocolContext::LengthPrefixed)]
        verify_token: Cow<'a, [u8]>,
        should_authenticate: bool,
    } = 0x01
    LoginSuccessPacket<'a> {
        player_uuid: Uuid,
        player_username: Cow<'a, str>,
        #[protocol(ctx = ArrayProtocolContext::LengthPrefixed)]
        properties: Cow<'a, [ClientLoginSuccessProperty<'a>]>,
        strict_error_handling: bool,
    } = 0x02
    SetCompressionPacket {
        #[protocol(varint)]
        packet_size_threshold: i32,
    } = 0x03
    LoginPluginRequestPacket<'a> {
        #[protocol(varint)]
        message_id: i32,
        #[protocol(ctx = IdentifierProtocolContext::SingleString)]
        channel: Identifier<'a>,
        #[protocol(ctx = ArrayProtocolContext::Remaining)]
        data: Cow<'a, [u8]>,
    } = 0x04
    LoginCookieRequestPacket<'a> {
        #[protocol(ctx = IdentifierProtocolContext::SingleString)]
        key: Identifier<'a>,
    } = 0x05
}

#[derive(Debug, Clone, Eq, PartialEq, Protocol)]
pub struct ClientLoginSuccessProperty<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
    #[protocol(ctx = (OptionProtocolContext::BoolPrefixed, ()))]
    pub signature: Option<Cow<'a, str>>,
}

packets! {
    ClientConfigurationPacket<'a>

    ConfigurationCookieRequestPacket<'a> {
        #[protocol(ctx = IdentifierProtocolContext::SingleString)]
        key: Identifier<'a>,
    } = 0x00
    ConfigurationClientboundPluginMessagePacket<'a> {
        #[protocol(ctx = IdentifierProtocolContext::SingleString)]
        channel: Identifier<'a>,
        #[protocol(ctx = ArrayProtocolContext::Remaining)]
        data: Cow<'a, [u8]>,
    } = 0x01
    ConfigurationDisconnectPacket {
        // TODO
    } = 0x02
    RegistryDataPacket<'a> {
        #[protocol(ctx = IdentifierProtocolContext::SingleString)]
        registry_id: Identifier<'a>,
        #[protocol(ctx = ArrayProtocolContext::LengthPrefixed)]
        registry_entries: Cow<'a, [RegistryEntry<'a>]>,
    } = 0x07
    ClientboundKnownPacksPacket<'a> {
        #[protocol(ctx = ArrayProtocolContext::LengthPrefixed)]
        known_packs: Cow<'a, [KnownPack<'a>]>,
    } = 0x0E
}

#[derive(Debug, Clone, Eq, PartialEq, Protocol)]
pub struct RegistryEntry<'a> {
    #[protocol(ctx = IdentifierProtocolContext::SingleString)]
    id: Identifier<'a>,
    #[protocol(ctx = (OptionProtocolContext::BoolPrefixed, ()))]
    data: Option<Nbt>, // TODO
}

packets! {
    ClientPlayPacket

    None2 {} = 0x00
}

#[derive(DelegateDebug, Clone, Eq, PartialEq, From)]
pub enum ClientPacket<'a> {
    Status(ClientStatusPacket),
    Login(ClientLoginPacket<'a>),
    Configuration(ClientConfigurationPacket<'a>),
    Play(ClientPlayPacket),
}

impl<'a> Encodable for ClientPacket<'a> {
    type Context = ();
    type Error = Infallible;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>> {
        match self {
            Self::Status(packet) => packet.encode(buf, ctx),
            Self::Login(packet) => packet.encode(buf, ctx),
            Self::Configuration(packet) => packet.encode(buf, ctx),
            Self::Play(packet) => packet.encode(buf, ctx),
        }
    }
}

impl<'a> Decodable for ClientPacket<'a> {
    type Context = PacketDecodeContext;
    type Error = PacketDecodeError;

    fn decode(buf: &mut dyn Buf, ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        Ok(match ctx.connection_state {
            ConnectionState::Handshaking => {
                return Err(DecodeError::Specific(
                    "there are no clientbound packets in handshaking state",
                ))
            }
            ConnectionState::Status => Self::Status(ClientStatusPacket::decode(buf, ctx)?),
            ConnectionState::Login => Self::Login(ClientLoginPacket::decode(buf, ctx)?),
            ConnectionState::Configuration => {
                Self::Configuration(ClientConfigurationPacket::decode(buf, ctx)?)
            }
            ConnectionState::Play => Self::Play(ClientPlayPacket::decode(buf, ctx)?),
        })
    }
}

impl<'a> Packet for ClientPacket<'a> {
    fn get_id(&self) -> i32 {
        match self {
            Self::Status(packet) => packet.get_id(),
            Self::Login(packet) => packet.get_id(),
            Self::Configuration(packet) => packet.get_id(),
            Self::Play(packet) => packet.get_id(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub version: StatusResponseVersion,
    pub players: StatusResponsePlayers,
    pub description: StatusResponseDescription,
    pub favicon: String,
    pub enforces_secure_chat: bool,
}

impl Encodable for StatusResponse {
    type Context = ();
    type Error = Infallible;

    fn encode(
        &self,
        buf: &mut dyn BufMut,
        _ctx: Self::Context,
    ) -> Result<(), EncodeError<Self::Error>> {
        buf::put_string(buf, &serde_json::to_string(self).unwrap());

        Ok(())
    }
}

impl Decodable for StatusResponse {
    type Context = ();
    type Error = Infallible;

    fn decode(buf: &mut dyn Buf, _ctx: Self::Context) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        Ok(serde_json::from_str(&buf::get_string(buf)?)?)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponsePlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<StatusResponsePlayersSample>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponsePlayersSample {
    pub name: String,
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseDescription {
    pub text: String,
}
