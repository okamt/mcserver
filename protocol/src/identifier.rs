use std::{
    borrow::Cow,
    fmt::{Display, Write},
};

use cowext::CowStrExt;
use thiserror::Error;

const IDENTIFIER_MAX_LEN: usize = 32767;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier<'a> {
    namespace: Option<Cow<'a, str>>,
    value: Cow<'a, str>,
}

impl<'a> Identifier<'a> {
    pub fn from_string(string: impl Into<Cow<'a, str>>) -> Result<Self, IdentifierParseError> {
        let string: Cow<'a, str> = string.into();

        if string.len() > IDENTIFIER_MAX_LEN {
            return Err(IdentifierParseError::TooLong(string.to_string()));
        }

        let mut colon_i = 0;

        for (i, c) in string.char_indices() {
            match c {
                'a'..='z' | '0'..='9' | '.' | '-' | '_' => continue,
                ':' if colon_i == 0 => colon_i = i,
                '/' if colon_i != 0 => continue,
                _ => {
                    return Err(IdentifierParseError::IllegalCharacter(
                        string.to_string(),
                        i,
                    ))
                }
            }
        }

        Ok(if colon_i == 0 {
            Self {
                namespace: None,
                value: string,
            }
        } else {
            let (mut left, right) = string.split_at(colon_i + 1);
            left.pop();
            Self {
                namespace: Some(left),
                value: right,
            }
        })
    }

    pub fn from_parts(
        namespace: impl Into<Cow<'a, str>>,
        value: impl Into<Cow<'a, str>>,
    ) -> Result<Self, IdentifierParseError> {
        let (namespace, value): (Cow<'a, str>, Cow<'a, str>) = (namespace.into(), value.into());

        for (i, c) in namespace.char_indices() {
            match c {
                'a'..='z' | '0'..='9' | '.' | '-' | '_' => continue,
                _ => {
                    return Err(IdentifierParseError::IllegalCharacter(
                        namespace.to_string(),
                        i,
                    ))
                }
            }
        }

        for (i, c) in value.char_indices() {
            match c {
                'a'..='z' | '0'..='9' | '.' | '-' | '_' | '/' => continue,
                _ => {
                    return Err(IdentifierParseError::IllegalCharacter(
                        namespace.to_string(),
                        i,
                    ))
                }
            }
        }

        Ok(Identifier {
            namespace: Some(namespace),
            value,
        })
    }

    pub fn namespace(&self) -> &str {
        self.namespace
            .as_ref()
            .unwrap_or(&Cow::Borrowed("minecraft"))
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl TryFrom<String> for Identifier<'static> {
    type Error = IdentifierParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Identifier::from_string(value)
    }
}

impl Display for Identifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.namespace().fmt(f)?;
        f.write_char(':')?;
        self.value.fmt(f)
    }
}

#[derive(Debug, Error)]
pub enum IdentifierParseError {
    #[error(
        "identifier {0} is too long, must be at most {} characters",
        IDENTIFIER_MAX_LEN
    )]
    TooLong(String),
    #[error("identifier {0} has illegal character at position {1}, must be one of [a-z0-9.-_] in namespace or [a-z0-9.-_/] in value")]
    IllegalCharacter(String, usize),
}
