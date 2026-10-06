use std::fmt;

use serde::{de, ser};

/// Everything that can go wrong converting between a Rust value and a box.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A `Serialize` or `Deserialize` implementation reported a problem, such
    /// as a missing or unknown field.
    Message(String),
    /// A key or value broke the protocol's length limits.
    Protocol(amp_protocol::Error),
    /// The top-level value is not a struct or map, so it cannot be a box.
    NotABox,
    /// The value has no representation in AMP. The payload says which kind of
    /// value, for example a nested struct or an enum variant carrying data.
    Unsupported(&'static str),
    /// A value on the wire could not be read as the requested type. The
    /// payload says what was expected.
    InvalidValue(&'static str),
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(message) => f.write_str(message),
            Error::Protocol(err) => write!(f, "{err}"),
            Error::NotABox => f.write_str("a box must be a struct or map at the top level"),
            Error::Unsupported(what) => write!(f, "{what} cannot be represented in AMP"),
            Error::InvalidValue(expected) => write!(f, "expected {expected}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Protocol(err) => Some(err),
            _ => None,
        }
    }
}
