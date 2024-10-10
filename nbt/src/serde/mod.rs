//! Serialization/deserialization for the NBT format.
//!
//! # Notes
//!
//! Some restrictions apply when serializing/deserializing to/from NBT:
//!
//! - Unsigned integers are not representable in NBT, and as such are casted into/from their signed counterparts.
//! - `bool`s rely on type hints from the deserializer to deserialize correctly, if your enum/struct is too complex consider using `serde_with::BoolFromInt`.

pub mod de;
pub mod error;
pub mod ser;

pub use de::{from_parser, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_bytes, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StackItem {
    Compound,
    List,
}
