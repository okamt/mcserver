use derive_more::derive::From;
use ownable::{IntoOwned, ToBorrowed, ToOwned};
use rgb::RGB8;
use serde::{de::Visitor, Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, BoolFromInt, BorrowCow, DisplayFromStr};
use std::borrow::Cow;
use uuid::Uuid;

use crate::{Identifier, Score};

#[derive(Serialize, Deserialize, Debug, PartialEq, IntoOwned, ToOwned)]
#[serde(remote = "Self")] // https://github.com/jonasbb/serde_with/issues/702
#[skip_serializing_none]
pub struct TextComponent<'a> {
    #[serde(borrow, flatten)]
    content: TextContent<'a>,
    #[serde(borrow, default, skip_serializing_if = "Vec::is_empty")]
    extra: Vec<Box<TextComponent<'a>>>,
    #[serde(borrow, flatten)]
    style: TextStyling<'a>,
}

impl<'a> From<&'a str> for TextComponent<'a> {
    fn from(value: &'a str) -> TextComponent<'a> {
        TextComponent {
            content: value.into(),
            extra: Vec::new(),
            style: Default::default(),
        }
    }
}

impl Serialize for TextComponent<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        TextComponent::serialize(self, serializer)
    }
}

impl<'a, 'de> Deserialize<'de> for TextComponent<'a>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = TextComponent<'de>;

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.into())
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                TextComponent::deserialize(serde::de::value::MapAccessDeserializer::new(map))
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a text component")
            }
        }

        deserializer.deserialize_any(V)
    }
}

// https://stackoverflow.com/questions/61216723/how-can-i-deserialize-an-enum-with-an-optional-internal-tag
#[serde_as]
#[derive(Serialize, Deserialize, Debug, From, PartialEq, IntoOwned, ToOwned)]
#[serde(untagged)]
pub enum TextContent<'a> {
    Text(#[serde(borrow)] TextContentText<'a>),
    Translatable(#[serde(borrow)] TextContentTranslatable<'a>),
    Keybind(#[serde(borrow)] TextContentKeybind<'a>),
    Score(#[serde(borrow)] TextContentScore<'a>),
    Selector(#[serde(borrow)] TextContentSelector<'a>),
    Nbt(#[serde(borrow)] TextContentNbt<'a>),
}

impl<'a> From<&'a str> for TextContent<'a> {
    fn from(value: &'a str) -> Self {
        TextContent::Text(value.into())
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, IntoOwned, ToBorrowed, ToOwned)]
#[serde(tag = "type", rename = "text")]
pub struct TextContentText<'a> {
    #[serde_as(as = "BorrowCow")]
    text: Cow<'a, str>,
}

impl<'a> From<&'a str> for TextContentText<'a> {
    fn from(value: &'a str) -> Self {
        TextContentText { text: value.into() }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, IntoOwned, ToOwned)]
#[serde(tag = "type", rename = "translatable")]
pub struct TextContentTranslatable<'a> {
    #[serde_as(as = "BorrowCow")]
    translate: Cow<'a, str>,
    #[serde(borrow)]
    with: Vec<Box<TextComponent<'a>>>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, IntoOwned, ToOwned)]
#[serde(tag = "type", rename = "keybind")]
pub struct TextContentKeybind<'a> {
    #[serde_as(as = "BorrowCow")]
    keybind: Cow<'a, str>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, IntoOwned, ToOwned)]
#[serde(tag = "type", rename = "score")]
pub struct TextContentScore<'a> {
    #[serde(borrow)]
    score: Score<'a>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, IntoOwned, ToOwned)]
#[serde(tag = "type", rename = "selector")]
pub struct TextContentSelector<'a> {
    #[serde_as(as = "BorrowCow")]
    selector: Cow<'a, str>,
    #[serde(borrow)]
    separator: Option<Box<TextComponent<'a>>>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, IntoOwned, ToOwned)]
#[serde(tag = "type", rename = "nbt")]
#[skip_serializing_none]
pub struct TextContentNbt<'a> {
    #[serde_as(as = "BorrowCow")]
    nbt: Cow<'a, str>,
    #[serde_as(as = "Option<BoolFromInt>")]
    interpret: Option<bool>,
    #[serde(borrow)]
    separator: Option<Box<TextComponent<'a>>>,
    #[serde(borrow, flatten)]
    source: TextContentNbtSource<'a>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, IntoOwned, ToBorrowed, ToOwned)]
#[serde(untagged)]
pub enum TextContentNbtSource<'a> {
    Block {
        #[serde_as(as = "BorrowCow")]
        block: Cow<'a, str>,
    },
    Entity {
        #[serde_as(as = "BorrowCow")]
        entity: Cow<'a, str>,
    },
    Storage {
        #[serde_as(as = "BorrowCow")]
        storage: Cow<'a, str>,
    },
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, IntoOwned, ToOwned)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct TextStyling<'a> {
    color: Option<TextColor>,
    #[serde_as(as = "Option<BoolFromInt>")]
    bold: Option<bool>,
    #[serde_as(as = "Option<BoolFromInt>")]
    italic: Option<bool>,
    #[serde_as(as = "Option<BoolFromInt>")]
    underlined: Option<bool>,
    #[serde_as(as = "Option<BoolFromInt>")]
    strikethrough: Option<bool>,
    #[serde_as(as = "Option<BoolFromInt>")]
    obfuscated: Option<bool>,
    #[serde(borrow)]
    font: Option<TextFont<'a>>,
    #[serde_as(as = "Option<BorrowCow>")]
    insertion: Option<Cow<'a, str>>,
    #[serde(borrow)]
    click_event: Option<TextClickEvent<'a>>,
    #[serde(borrow)]
    hover_event: Option<TextHoverEvent<'a>>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, IntoOwned, ToBorrowed, ToOwned)]
#[serde(rename_all = "snake_case", tag = "action", content = "value")]
pub enum TextClickEvent<'a> {
    OpenUrl(#[serde_as(as = "BorrowCow")] Cow<'a, str>),
    RunCommand(#[serde_as(as = "BorrowCow")] Cow<'a, str>),
    SuggestCommand(#[serde_as(as = "BorrowCow")] Cow<'a, str>),
    ChangePage(#[serde_as(as = "DisplayFromStr")] i32),
    CopyToClipboard(#[serde_as(as = "BorrowCow")] Cow<'a, str>),
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, IntoOwned, ToOwned)]
#[serde(rename_all = "snake_case", tag = "action", content = "value")]
#[skip_serializing_none]
pub enum TextHoverEvent<'a> {
    ShowText(#[serde(borrow)] Box<TextComponent<'a>>),
    ShowItem {
        #[serde_as(as = "BorrowCow")]
        id: Cow<'a, str>,
        count: i32,
        #[serde_as(as = "Option<BorrowCow>")]
        tag: Option<Cow<'a, str>>,
    },
    ShowEntity {
        #[serde_as(as = "BorrowCow")]
        r#type: Cow<'a, str>,
        #[ownable(clone)]
        id: Uuid,
        #[serde_as(as = "Option<BorrowCow>")]
        name: Option<Cow<'a, str>>,
    },
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntoOwned, ToOwned)]
pub enum TextColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkCyan,
    DarkRed,
    Purple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    BrightGreen,
    Cyan,
    Red,
    Pink,
    Yellow,
    White,
    #[ownable(clone)] Custom(RGB8)
}

impl TextColor {
    #[rustfmt::skip]
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "black"        => TextColor::Black,
            "dark_blue"    => TextColor::DarkBlue,
            "dark_green"   => TextColor::DarkGreen,
            "dark_aqua"    => TextColor::DarkCyan,
            "dark_red"     => TextColor::DarkRed,
            "dark_purple"  => TextColor::Purple,
            "gold"         => TextColor::Gold,
            "gray"         => TextColor::Gray,
            "dark_gray"    => TextColor::DarkGray,
            "blue"         => TextColor::Blue,
            "green"        => TextColor::BrightGreen,
            "aqua"         => TextColor::Cyan,
            "red"          => TextColor::Red,
            "light_purple" => TextColor::Pink,
            "yellow"       => TextColor::Yellow,
            "white"        => TextColor::White,
            _              => return None,
        })
    }

    #[rustfmt::skip]
    pub const fn name(&self) -> Option<&'static str> {
        Some(match self {
            TextColor::Black       => "black",
            TextColor::DarkBlue    => "dark_blue",
            TextColor::DarkGreen   => "dark_green",
            TextColor::DarkCyan    => "dark_aqua",
            TextColor::DarkRed     => "dark_red",
            TextColor::Purple      => "dark_purple",
            TextColor::Gold        => "gold",
            TextColor::Gray        => "gray",
            TextColor::DarkGray    => "dark_gray",
            TextColor::Blue        => "blue",
            TextColor::BrightGreen => "green",
            TextColor::Cyan        => "aqua",
            TextColor::Red         => "red",
            TextColor::Pink        => "light_purple",
            TextColor::Yellow      => "yellow",
            TextColor::White       => "white",
            _                      => return None,
        })
    }

    #[rustfmt::skip]
    pub const fn foreground(&self) -> RGB8 {
        match self {
            TextColor::Black       => RGB8 { r: 0,   g: 0,   b: 0   }, // #000000
            TextColor::DarkBlue    => RGB8 { r: 0,   g: 0,   b: 170 }, // #0000aa
            TextColor::DarkGreen   => RGB8 { r: 0,   g: 170, b: 0   }, // #00aa00
            TextColor::DarkCyan    => RGB8 { r: 0,   g: 170, b: 170 }, // #00aaaa
            TextColor::DarkRed     => RGB8 { r: 170, g: 0,   b: 0   }, // #aa0000
            TextColor::Purple      => RGB8 { r: 170, g: 0,   b: 170 }, // #aa00aa
            TextColor::Gold        => RGB8 { r: 255, g: 170, b: 0   }, // #ffaa00
            TextColor::Gray        => RGB8 { r: 170, g: 170, b: 170 }, // #aaaaaa
            TextColor::DarkGray    => RGB8 { r: 85,  g: 85,  b: 85  }, // #555555
            TextColor::Blue        => RGB8 { r: 85,  g: 85,  b: 255 }, // #5555ff
            TextColor::BrightGreen => RGB8 { r: 85,  g: 255, b: 85  }, // #55ff55
            TextColor::Cyan        => RGB8 { r: 85,  g: 255, b: 255 }, // #55ffff
            TextColor::Red         => RGB8 { r: 255, g: 85,  b: 85  }, // #ff5555
            TextColor::Pink        => RGB8 { r: 255, g: 85,  b: 255 }, // #ff55ff
            TextColor::Yellow      => RGB8 { r: 255, g: 255, b: 85  }, // #ffff55
            TextColor::White       => RGB8 { r: 255, g: 255, b: 255 }, // #ffffff
            TextColor::Custom(c)   => *c,
        }
    }

    #[rustfmt::skip]
    pub const fn background(&self) -> RGB8 {
        match self {
            TextColor::Black       => RGB8 { r: 0,   g: 0,   b: 0   }, // #000000
            TextColor::DarkBlue    => RGB8 { r: 0,   g: 0,   b: 42  }, // #00002a
            TextColor::DarkGreen   => RGB8 { r: 0,   g: 42,  b: 0   }, // #002a00
            TextColor::DarkCyan    => RGB8 { r: 0,   g: 42,  b: 42  }, // #002a2a
            TextColor::DarkRed     => RGB8 { r: 42,  g: 0,   b: 0   }, // #2a0000
            TextColor::Purple      => RGB8 { r: 42,  g: 0,   b: 42  }, // #2a002a
            TextColor::Gold        => RGB8 { r: 42,  g: 42,  b: 0   }, // #2a2a00
            TextColor::Gray        => RGB8 { r: 42,  g: 42,  b: 42  }, // #2a2a2a
            TextColor::DarkGray    => RGB8 { r: 21,  g: 21,  b: 21  }, // #151515
            TextColor::Blue        => RGB8 { r: 21,  g: 21,  b: 63  }, // #15153f
            TextColor::BrightGreen => RGB8 { r: 21,  g: 63,  b: 21  }, // #153f15
            TextColor::Cyan        => RGB8 { r: 21,  g: 63,  b: 63  }, // #153f3f
            TextColor::Red         => RGB8 { r: 63,  g: 21,  b: 21  }, // #3f1515
            TextColor::Pink        => RGB8 { r: 63,  g: 21,  b: 63  }, // #3f153f
            TextColor::Yellow      => RGB8 { r: 63,  g: 63,  b: 21  }, // #3f3f15
            TextColor::White       => RGB8 { r: 63,  g: 63,  b: 63  }, // #3f3f3f
            TextColor::Custom(c)   => *c,
        }
    }
}

impl Into<TextColor> for RGB8 {
    fn into(self) -> TextColor {
        TextColor::Custom(self)
    }
}

impl Serialize for TextColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(name) = self.name() {
            serializer.serialize_str(name)
        } else {
            let rgb = self.foreground();
            serializer.serialize_str(&format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b))
        }
    }
}

impl<'de> Deserialize<'de> for TextColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = TextColor;

            fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Some(color) = TextColor::from_name(&string) {
                    return Ok(color);
                }

                let string = match string.strip_prefix("#") {
                    Some(v) => v,
                    None => return Err(E::custom("expected #")),
                };
                let [r, g, b] = [0, 2, 4].map(|i| {
                    let hex_code = string
                        .get(i..i + 2)
                        .ok_or(E::custom("expected valid hex code"))?;
                    u8::from_str_radix(hex_code, 16).map_err(E::custom)
                });
                Ok(RGB8 {
                    r: r?,
                    g: g?,
                    b: b?,
                }
                .into())
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a color name or a #-prefixed hexadecimal RGB specification")
            }
        }

        deserializer.deserialize_str(V)
    }
}

pub enum TextStyle {
    Random,
    Bold,
    Strikethrough,
    Underlined,
    Italic,
}

impl TextStyle {
    #[rustfmt::skip]
    pub const fn from_code(code: char) -> Option<Self> {
        Some(match code {
            'k' => TextStyle::Random,
            'l' => TextStyle::Bold,
            'm' => TextStyle::Strikethrough,
            'n' => TextStyle::Underlined,
            'o' => TextStyle::Italic,
            _   => return None,
        })
    }

    #[rustfmt::skip]
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "obfuscated"    => TextStyle::Random,
            "bold"          => TextStyle::Bold,
            "strikethrough" => TextStyle::Strikethrough,
            "underline"     => TextStyle::Underlined,
            "italic"        => TextStyle::Italic,
            _               => return None,
        })
    }

    #[rustfmt::skip]
    pub const fn code(&self) -> char {
        match self {
            TextStyle::Random =>        'k',
            TextStyle::Bold =>          'l',
            TextStyle::Strikethrough => 'm',
            TextStyle::Underlined =>    'n',
            TextStyle::Italic =>        'o',
        }
    }

    #[rustfmt::skip]
    pub const fn name(&self) -> &'static str {
        match self {
            TextStyle::Random =>        "obfuscated",
            TextStyle::Bold =>          "bold",
            TextStyle::Strikethrough => "strikethrough",
            TextStyle::Underlined =>    "underline",
            TextStyle::Italic =>        "italic",
        }
    }
}

#[derive(Debug, PartialEq, IntoOwned, ToBorrowed, ToOwned)]
pub enum TextFont<'a> {
    Default,
    Uniform,
    Alt,
    Illageralt,
    Custom(Identifier<'a>),
}

impl<'a> TextFont<'a> {
    pub fn identifier<'b>(&'b self) -> Identifier<'a>
    where
        'b: 'a,
    {
        match self {
            TextFont::Default => Identifier {
                namespace: None,
                value: Cow::Borrowed("default"),
            },
            TextFont::Uniform => Identifier {
                namespace: None,
                value: Cow::Borrowed("uniform"),
            },
            TextFont::Alt => Identifier {
                namespace: None,
                value: Cow::Borrowed("alt"),
            },
            TextFont::Illageralt => Identifier {
                namespace: None,
                value: Cow::Borrowed("illageralt"),
            },
            TextFont::Custom(identifier) => identifier.as_borrowed::<'b>(),
        }
    }
}

impl<'a> Into<Identifier<'a>> for TextFont<'a> {
    fn into(self) -> Identifier<'a> {
        match self {
            TextFont::Default => Identifier {
                namespace: None,
                value: Cow::Borrowed("default"),
            },
            TextFont::Uniform => Identifier {
                namespace: None,
                value: Cow::Borrowed("uniform"),
            },
            TextFont::Alt => Identifier {
                namespace: None,
                value: Cow::Borrowed("alt"),
            },
            TextFont::Illageralt => Identifier {
                namespace: None,
                value: Cow::Borrowed("illageralt"),
            },
            TextFont::Custom(identifier) => identifier,
        }
    }
}

impl Serialize for TextFont<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.identifier().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TextFont<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = TextFont<'static>;

            fn visit_string<E>(self, string: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let identifier: Identifier<'static> = string.try_into().map_err(E::custom)?;

                Ok(match identifier.value.as_ref() {
                    "default" => TextFont::Default,
                    "uniform" => TextFont::Uniform,
                    "alt" => TextFont::Alt,
                    "illageralt" => TextFont::Illageralt,
                    _ => TextFont::Custom(identifier),
                })
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an identifier")
            }
        }

        deserializer.deserialize_str(V)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Random text component extracted from a Hypixel packet.
    #[rustfmt::skip]
    const TEXT_COMPONENT_1_NBT: &[u8] = &[0x0a, 0x09, 0x00, 0x05, 0x65, 0x78, 0x74, 0x72, 0x61, 0x0a, 0x00, 0x00, 0x00, 0x03, 0x01, 0x00, 0x0d, 0x73, 0x74, 0x72, 0x69, 0x6b, 0x65, 0x74, 0x68, 0x72, 0x6f, 0x75, 0x67, 0x68, 0x00, 0x08, 0x00, 0x04, 0x74, 0x65, 0x78, 0x74, 0x00, 0x11, 0x20, 0xc2, 0xa7, 0x62, 0x3e, 0xc2, 0xa7, 0x63, 0x3e, 0xc2, 0xa7, 0x61, 0x3e, 0xc2, 0xa7, 0x72, 0x20, 0x00, 0x0a, 0x00, 0x0a, 0x63, 0x6c, 0x69, 0x63, 0x6b, 0x45, 0x76, 0x65, 0x6e, 0x74, 0x08, 0x00, 0x06, 0x61, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x00, 0x0b, 0x72, 0x75, 0x6e, 0x5f, 0x63, 0x6f, 0x6d, 0x6d, 0x61, 0x6e, 0x64, 0x08, 0x00, 0x05, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x00, 0x31, 0x2f, 0x76, 0x69, 0x65, 0x77, 0x70, 0x72, 0x6f, 0x66, 0x69, 0x6c, 0x65, 0x20, 0x64, 0x37, 0x66, 0x33, 0x63, 0x37, 0x35, 0x30, 0x2d, 0x37, 0x39, 0x31, 0x63, 0x2d, 0x34, 0x66, 0x37, 0x33, 0x2d, 0x61, 0x30, 0x34, 0x38, 0x2d, 0x62, 0x30, 0x36, 0x32, 0x34, 0x66, 0x64, 0x31, 0x62, 0x61, 0x36, 0x36, 0x00, 0x0a, 0x00, 0x0a, 0x68, 0x6f, 0x76, 0x65, 0x72, 0x45, 0x76, 0x65, 0x6e, 0x74, 0x08, 0x00, 0x06, 0x61, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x00, 0x09, 0x73, 0x68, 0x6f, 0x77, 0x5f, 0x74, 0x65, 0x78, 0x74, 0x0a, 0x00, 0x05, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x01, 0x00, 0x0d, 0x73, 0x74, 0x72, 0x69, 0x6b, 0x65, 0x74, 0x68, 0x72, 0x6f, 0x75, 0x67, 0x68, 0x00, 0x08, 0x00, 0x04, 0x74, 0x65, 0x78, 0x74, 0x00, 0x95, 0xc2, 0xa7, 0x36, 0x5b, 0x4d, 0x56, 0x50, 0xc2, 0xa7, 0x30, 0x2b, 0x2b, 0xc2, 0xa7, 0x36, 0x5d, 0x20, 0x41, 0x76, 0x65, 0x72, 0x73, 0x61, 0xc2, 0xa7, 0x66, 0x0a, 0xc2, 0xa7, 0x37, 0x48, 0x79, 0x70, 0x69, 0x78, 0x65, 0x6c, 0x20, 0x4c, 0x65, 0x76, 0x65, 0x6c, 0x3a, 0x20, 0xc2, 0xa7, 0x36, 0x32, 0x35, 0x30, 0x0a, 0xc2, 0xa7, 0x37, 0x41, 0x63, 0x68, 0x69, 0x65, 0x76, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x20, 0x50, 0x6f, 0x69, 0x6e, 0x74, 0x73, 0x3a, 0x20, 0xc2, 0xa7, 0x65, 0x36, 0x2c, 0x32, 0x39, 0x30, 0x0a, 0xc2, 0xa7, 0x37, 0x47, 0x75, 0x69, 0x6c, 0x64, 0x3a, 0x20, 0xc2, 0xa7, 0x62, 0x45, 0x73, 0x74, 0x61, 0x6e, 0x64, 0x61, 0x72, 0x74, 0x65, 0x0a, 0x0a, 0xc2, 0xa7, 0x65, 0x43, 0x6c, 0x69, 0x63, 0x6b, 0x20, 0x74, 0x6f, 0x20, 0x76, 0x69, 0x65, 0x77, 0x20, 0xc2, 0xa7, 0x36, 0x41, 0x76, 0x65, 0x72, 0x73, 0x61, 0xc2, 0xa7, 0x65, 0x27, 0x73, 0x20, 0x70, 0x72, 0x6f, 0x66, 0x69, 0x6c, 0x65, 0x21, 0x00, 0x00, 0x01, 0x00, 0x0d, 0x73, 0x74, 0x72, 0x69, 0x6b, 0x65, 0x74, 0x68, 0x72, 0x6f, 0x75, 0x67, 0x68, 0x00, 0x08, 0x00, 0x04, 0x74, 0x65, 0x78, 0x74, 0x00, 0x2f, 0xc2, 0xa7, 0x36, 0x5b, 0x4d, 0x56, 0x50, 0xc2, 0xa7, 0x30, 0x2b, 0x2b, 0xc2, 0xa7, 0x36, 0x5d, 0x20, 0x41, 0x76, 0x65, 0x72, 0x73, 0x61, 0xc2, 0xa7, 0x66, 0x20, 0xc2, 0xa7, 0x36, 0x6a, 0x6f, 0x69, 0x6e, 0x65, 0x64, 0x20, 0x74, 0x68, 0x65, 0x20, 0x6c, 0x6f, 0x62, 0x62, 0x79, 0x21, 0x00, 0x01, 0x00, 0x0d, 0x73, 0x74, 0x72, 0x69, 0x6b, 0x65, 0x74, 0x68, 0x72, 0x6f, 0x75, 0x67, 0x68, 0x00, 0x08, 0x00, 0x04, 0x74, 0x65, 0x78, 0x74, 0x00, 0x0d, 0x20, 0xc2, 0xa7, 0x61, 0x3c, 0xc2, 0xa7, 0x63, 0x3c, 0xc2, 0xa7, 0x62, 0x3c, 0x00, 0x01, 0x00, 0x0d, 0x73, 0x74, 0x72, 0x69, 0x6b, 0x65, 0x74, 0x68, 0x72, 0x6f, 0x75, 0x67, 0x68, 0x00, 0x08, 0x00, 0x04, 0x74, 0x65, 0x78, 0x74, 0x00, 0x00, 0x00];
    const TEXT_COMPONENT_1_JSON: &str = r###"
        {
            "extra": [
                {
                    "strikethrough": 0,
                    "text": " §b>§c>§a>§r "
                },
                {
                    "clickEvent": {
                        "action": "run_command",
                        "value": "/viewprofile d7f3c750-791c-4f73-a048-b0624fd1ba66"
                    },
                    "hoverEvent": {
                        "action": "show_text",
                        "value": {
                            "strikethrough": 0,
                            "text": "§6[MVP§0++§6] Aversa§f\n§7Hypixel Level: §6250\n§7Achievement Points: §e6,290\n§7Guild: §bEstandarte\n\n§eClick to view §6Aversa§e's profile!"
                        }
                    },
                    "strikethrough": 0,
                    "text": "§6[MVP§0++§6] Aversa§f §6joined the lobby!"
                },
                {
                    "strikethrough": 0,
                    "text": " §a<§c<§b<"
                }
            ],
            "strikethrough": 0,
            "text": ""
        }
    "###;

    fn nbt_text_component() -> TextComponent<'static> {
        use nbt::*;

        let parser = NbtParser::parse(TEXT_COMPONENT_1_NBT, true).expect("NBT should be valid");

        let deserializer = &mut serde::Deserializer::from_parser(&parser);
        let text_component: TextComponent<'_> = serde_path_to_error::deserialize(deserializer)
            .expect("NBT encoded text component should be deserializable");
        text_component.into_owned()
    }

    fn json_text_component() -> TextComponent<'static> {
        serde_json::from_str::<TextComponent<'_>>(TEXT_COMPONENT_1_JSON)
            .expect("JSON text component should be deserializable")
    }

    #[test]
    fn nbt_same_as_json() {
        assert_eq!(nbt_text_component(), json_text_component());
    }
}
