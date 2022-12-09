use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Message(String),
    Eof,
    TrailingCharacters,
    BadData,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(message) => formatter.write_str(&format!("Error: {}", message)),
            Error::BadData => formatter.write_str("Error: Bad data"),
            Error::Eof => formatter.write_str("Error: Unexpected EOF"),
            Error::TrailingCharacters => {
                formatter.write_str("Error: Unexpected trailing characters")
            }
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::Eof => "unexpected end of file",
            Error::TrailingCharacters => "characters after the end",
            Error::BadData => "bad or malformed data",
        }
    }
}
