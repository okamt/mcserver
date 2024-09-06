//! Clientbound packets.

use crate::packet::ambassador_impl_Packet;
use ambassador::Delegate;
use bytes::{BufMut, BytesMut};
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
    None,
}

impl Packet for ClientLoginPacket {
    fn get_id(&self) -> i32 {
        self.to_i32().unwrap()
    }

    fn encode(&self) -> PacketEncodeResult<Vec<u8>> {
        todo!()
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

#[derive(Debug, Clone, Eq, PartialEq, Delegate)]
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
