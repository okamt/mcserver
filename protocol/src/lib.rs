use std::convert::Infallible;

use bitflags::bitflags;
use getset::Getters;
use num_derive::{FromPrimitive, ToPrimitive};
use protocol_derive::Protocol;

pub mod buf;
pub mod identifier;

pub use buf::{Decodable, DecodeError, Encodable, EncodeError};

use crate as protocol;

#[derive(Debug, Eq, PartialEq, Clone, Copy, FromPrimitive, ToPrimitive, Protocol)]
#[repr(i32)]
#[protocol(varint)]
pub enum ConnectionState {
    Handshaking = 0,
    Status = 1,
    Login = 2,
    Configuration = 3,
    Play = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, Protocol)]
#[repr(i32)]
#[protocol(varint)]
pub enum ChatMode {
    Full = 0,
    CommandsOnly = 1,
    Hidden = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, Protocol)]
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
