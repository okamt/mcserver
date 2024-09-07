use std::fmt::{Display, Write};

use bitflags::bitflags;
use getset::Getters;
use num_derive::{FromPrimitive, ToPrimitive};
use thiserror::Error;

use crate::packet::server::ServerConfigurationPacket;

const IDENTIFIER_MAX_LEN: usize = 32767;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    namespace: String,
    value: String,
}

impl Identifier {
    pub fn from_string(string: impl ToString) -> Result<Self, IdentifierError> {
        let string = string.to_string();

        if string.len() > IDENTIFIER_MAX_LEN {
            return Err(IdentifierError::TooLong(string));
        }

        let mut colon_i = 0;

        for (i, c) in string.char_indices() {
            match c {
                'a'..='z' | '0'..='9' | '.' | '-' | '_' => continue,
                ':' if colon_i == 0 => colon_i = i,
                '/' if colon_i != 0 => continue,
                _ => return Err(IdentifierError::IllegalCharacter(string, i)),
            }
        }

        Ok(Self {
            namespace: string[..colon_i].to_string(),
            value: string[colon_i + 1..].to_string(),
        })
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl TryFrom<String> for Identifier {
    type Error = IdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Identifier::from_string(value)
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.namespace.fmt(f)?;
        f.write_char(':')?;
        self.value.fmt(f)
    }
}

#[derive(Debug, Error)]
pub enum IdentifierError {
    #[error(
        "identifier {0} is too long, must be at most {} characters",
        IDENTIFIER_MAX_LEN
    )]
    TooLong(String),
    #[error("identifier {0} has illegal character at position {1}, must be one of [a-z0-9.-_] in namespace or [a-z0-9.-_/] in value")]
    IllegalCharacter(String, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum ChatMode {
    Full = 0,
    CommandsOnly = 1,
    Hidden = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
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

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct ClientInformation {
    locale: String,
    view_distance: u8,
    chat_mode: ChatMode,
    chat_colors: bool,
    displayed_skin_parts: DisplayedSkinParts,
    main_hand: Hand,
    enable_text_filtering: bool,
    allow_server_listings: bool,
}

impl ClientInformation {
    pub fn from_packet(packet: ServerConfigurationPacket) -> Option<Self> {
        match packet {
            ServerConfigurationPacket::ClientInformation {
                locale,
                view_distance,
                chat_mode,
                chat_colors,
                displayed_skin_parts,
                main_hand,
                enable_text_filtering,
                allow_server_listings,
            } => Some(Self {
                locale,
                view_distance,
                chat_mode,
                chat_colors,
                displayed_skin_parts,
                main_hand,
                enable_text_filtering,
                allow_server_listings,
            }),
            _ => None,
        }
    }
}
