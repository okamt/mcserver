//! Clientbound packets.

use crate::packet::ambassador_impl_Packet;
use ambassador::Delegate;
use bytes::{BufMut, BytesMut};
use derive_more::derive::From;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::connection::ConnectionState;

use super::{BufExt, BufMutExt, Packet, PacketDecodeResult, PacketDecoder, PacketEncodeResult};

#[derive(Debug, Clone, Eq, PartialEq, ToPrimitive)]
#[repr(i32)]
pub enum ClientStatusPacket {
    StatusResponse { response: StatusResponse } = 0x00,
    PongResponse { payload: i64 } = 0x01,
}

impl Packet for ClientStatusPacket {
    fn get_id(&self) -> i32 {
        self.to_i32().unwrap()
    }

    fn encode(&self) -> PacketEncodeResult<Vec<u8>> {
        let mut buf = BytesMut::with_capacity(4096);

        match self {
            Self::StatusResponse { response } => {
                buf.put_string(serde_json::to_string(response).unwrap());
            }
            Self::PongResponse { payload } => {
                buf.put_i64(*payload);
            }
        }

        Ok(buf.into())
    }
}

impl PacketDecoder for ClientStatusPacket {
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
#[repr(i32)]
pub enum ClientLoginPacket {
    LoginSuccess {
        player_uuid: Uuid,
        player_username: String,
        properties: Vec<ClientLoginSuccessProperty>,
        strict_error_handling: bool,
    } = 0x02,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClientLoginSuccessProperty {
    name: String,
    value: String,
    is_signed: bool,
    signature: Option<String>,
}

impl Packet for ClientLoginPacket {
    fn get_id(&self) -> i32 {
        self.to_i32().unwrap()
    }

    fn encode(&self) -> PacketEncodeResult<Vec<u8>> {
        let mut buf = BytesMut::with_capacity(4096);

        match self {
            ClientLoginPacket::LoginSuccess {
                player_uuid,
                player_username,
                properties,
                strict_error_handling,
            } => {
                buf.put_uuid(player_uuid);
                buf.put_string(player_username);
                buf.put_varint(properties.len().try_into().unwrap());
                for property in properties {
                    buf.put_string(&property.name);
                    buf.put_string(&property.value);
                    buf.put_bool(property.is_signed);
                    if property.is_signed {
                        buf.put_string(
                            property
                                .signature
                                .as_ref()
                                .expect("signature must be present if is_signed is true"),
                        );
                    }
                }
                buf.put_bool(*strict_error_handling);
            }
        }

        Ok(buf.into())
    }
}

impl PacketDecoder for ClientLoginPacket {
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
#[repr(i32)]
pub enum ClientConfigurationPacket {
    None,
}

impl Packet for ClientConfigurationPacket {
    fn get_id(&self) -> i32 {
        self.to_i32().unwrap()
    }

    fn encode(&self) -> PacketEncodeResult<Vec<u8>> {
        todo!()
    }
}

impl PacketDecoder for ClientConfigurationPacket {
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
#[repr(i32)]
pub enum ClientPlayPacket {
    None,
}

impl Packet for ClientPlayPacket {
    fn get_id(&self) -> i32 {
        self.to_i32().unwrap()
    }

    fn encode(&self) -> PacketEncodeResult<Vec<u8>> {
        todo!()
    }
}

impl PacketDecoder for ClientPlayPacket {
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

#[derive(Debug, Clone, Eq, PartialEq, From, Delegate)]
#[delegate(Packet)]
pub enum ClientPacket {
    Status(ClientStatusPacket),
    Login(ClientLoginPacket),
    Configuration(ClientConfigurationPacket),
    Play(ClientPlayPacket),
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
