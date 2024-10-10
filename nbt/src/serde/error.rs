use std::fmt::Display;

use serde::{de, ser};
use thiserror::Error;

use crate::NbtParseError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Serde(String),
    #[error(transparent)]
    Parse(#[from] NbtParseError),
    #[error("reached end of file")]
    Eof,
    #[error("no more values to deserialize")]
    NoMoreValues,
    #[error("value to deserialize has no name")]
    NoName,
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Serde(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Serde(msg.to_string())
    }
}
