use std::fmt::{Display, Write};

use thiserror::Error;

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
