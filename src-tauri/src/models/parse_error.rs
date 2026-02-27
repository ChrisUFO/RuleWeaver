use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseEnumError;

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid enum variant")
    }
}

impl std::error::Error for ParseEnumError {}
