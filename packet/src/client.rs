//! Clientbound packets.

use std::convert::Infallible;

use crate::{Packet, PacketDecodeContext, PacketDecodeError};
use bytes::{Buf, BufMut};
use derive_more::derive::From;
use protocol::{
    buf::{OptionProtocolContext, VecProtocolContext},
    identifier::Identifier,
    Decodable, DecodeError, Encodable, EncodeError,
};
use protocol_derive::Protocol;
use serde::{Deserialize, Serialize};
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
    ClientLoginPacket

    LoginDisconnectPacket {} = 0x00
    EncryptionRequestPacket {
        server_id: String,
        #[protocol(ctx = VecProtocolContext::LengthPrefixed)]
        public_key: Vec<u8>,
        #[protocol(ctx = VecProtocolContext::LengthPrefixed)]
        verify_token: Vec<u8>,
        should_authenticate: bool,
    } = 0x01
    LoginSuccessPacket {
        player_uuid: Uuid,
        player_username: String,
        #[protocol(ctx = VecProtocolContext::LengthPrefixed)]
        properties: Vec<ClientLoginSuccessProperty>,
        strict_error_handling: bool,
    } = 0x02
    SetCompressionPacket {
        #[protocol(varint)]
        packet_size_threshold: i32,
    } = 0x03
    LoginPluginRequestPacket {
        #[protocol(varint)]
        message_id: i32,
        channel: Identifier,
        #[protocol(ctx = VecProtocolContext::Remaining)]
        data: Vec<u8>,
    } = 0x04
    LoginCookieRequestPacket {
        key: Identifier,
    } = 0x05
}

#[derive(Debug, Clone, Eq, PartialEq, Protocol)]
pub struct ClientLoginSuccessProperty {
    name: String,
    value: String,
    #[protocol(ctx = OptionProtocolContext::BoolPrefixed)]
    signature: Option<String>,
}

packets! {
    ClientConfigurationPacket

    None1 {} = 0x00
}

packets! {
    ClientPlayPacket

    None2 {} = 0x00
}

#[derive(Debug, Clone, Eq, PartialEq, From)]
pub enum ClientPacket {
    Status(ClientStatusPacket),
    Login(ClientLoginPacket),
    Configuration(ClientConfigurationPacket),
    Play(ClientPlayPacket),
}

impl Encodable for ClientPacket {
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

impl Decodable for ClientPacket {
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

impl Packet for ClientPacket {
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
