use std::{borrow::Cow, convert::Infallible};

use bitflags::bitflags;
use enum_map::Enum;
use getset::Getters;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use ownable::{IntoOwned, ToBorrowed, ToOwned};
use protocol_derive::Protocol;

pub mod buf;
pub mod identifier;
pub mod text;

pub use buf::{Decodable, DecodeError, Encodable, EncodeError};
pub use identifier::*;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, BorrowCow};

use crate as protocol;

#[derive(Debug, Eq, PartialEq, Clone, Copy, TryFromPrimitive, IntoPrimitive, Protocol)]
#[repr(i32)]
#[protocol(varint)]
pub enum ConnectionState {
    Handshaking = 0,
    Status = 1,
    Login = 2,
    Configuration = 3,
    Play = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Protocol)]
#[repr(i32)]
#[protocol(varint)]
pub enum ChatMode {
    Full = 0,
    CommandsOnly = 1,
    Hidden = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, Protocol)]
#[repr(i32)]
#[protocol(varint)]
pub enum Hand {
    Left = 0,
    Right = 1,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DisplayedSkinParts: u8 {
        const CAPE = 0x01;
        const JACKET = 0x02;
        const LEFT_SLEEVE = 0x04;
        const RIGHT_SLEEVE = 0x08;
        const LEFT_PANTS_LEG = 0x10;
        const RIGHT_PANTS_LEG = 0x20;
        const HAT = 0x40;
        const UNUSED = 0x80;
    }
}

impl Encodable for DisplayedSkinParts {
    type Context = ();
    type Error = Infallible;

    fn encode(
        &self,
        buf: &mut dyn bytes::BufMut,
        _ctx: (),
    ) -> Result<(), EncodeError<Self::Error>> {
        buf.put_u8(self.bits());

        Ok(())
    }
}

impl Decodable for DisplayedSkinParts {
    type Context = ();
    type Error = Infallible;

    fn decode(buf: &mut dyn bytes::Buf, _ctx: ()) -> Result<Self, DecodeError<Self::Error>>
    where
        Self: Sized,
    {
        Ok(DisplayedSkinParts::from_bits(buf.get_u8()).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct ClientInformation {
    pub locale: String,
    pub view_distance: u8,
    pub chat_mode: ChatMode,
    pub chat_colors: bool,
    pub displayed_skin_parts: DisplayedSkinParts,
    pub main_hand: Hand,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArmorMaterial {
    Leather,
    Chainmail,
    Iron,
    Gold,
    Diamond,
    Turtle,
    Netherite,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, IntoOwned, ToBorrowed, ToOwned)]
pub struct Score<'a> {
    #[serde_as(as = "BorrowCow")]
    name: Cow<'a, str>,
    #[serde_as(as = "BorrowCow")]
    objective: Cow<'a, str>,
}
