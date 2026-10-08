use std::fmt;

use amp_protocol::ErrorResponse;

/// Why a command failed.
///
/// A handler fails a command by returning [`Error::new`] with the code and
/// description the peer should see, or by returning any other variant, which
/// goes on the wire with the code `UNKNOWN`. A client receives the peer's
/// code and description back as [`Error::Command`].
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The command failed with an AMP error code and description.
    Command {
        /// The error code, by convention all-caps `SNAKE_CASE`.
        code: String,
        /// A human-readable description of the failure.
        description: String,
    },
    /// The connection failed.
    Io(std::io::Error),
    /// A box broke the protocol, or a peer sent a box that is neither a
    /// request nor a response.
    Protocol(amp_protocol::Error),
    /// Arguments or results could not be converted to or from a box.
    Serde(serde_amp::Error),
    /// The connection closed before the command was answered.
    ConnectionClosed,
}

impl Error {
    /// Fails a command with `code` and `description`.
    pub fn new(code: impl Into<String>, description: impl Into<String>) -> Self {
        Error::Command {
            code: code.into(),
            description: description.into(),
        }
    }

    /// The error for a request naming a command the router does not know.
    pub(crate) fn unhandled(command: &[u8]) -> Self {
        Error::new(
            ErrorResponse::UNHANDLED,
            format!("unhandled command: {}", String::from_utf8_lossy(command)),
        )
    }

    /// The code that goes on the wire.
    pub(crate) fn code(&self) -> &str {
        match self {
            Error::Command { code, .. } => code,
            _ => ErrorResponse::UNKNOWN,
        }
    }

    /// The description that goes on the wire.
    pub(crate) fn description(&self) -> String {
        match self {
            Error::Command { description, .. } => description.clone(),
            other => other.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Command { code, description } => write!(f, "{code}: {description}"),
            Error::Io(err) => write!(f, "connection failed: {err}"),
            Error::Protocol(err) => write!(f, "protocol error: {err}"),
            Error::Serde(err) => write!(f, "conversion failed: {err}"),
            Error::ConnectionClosed => f.write_str("connection closed"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Protocol(err) => Some(err),
            Error::Serde(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<amp_protocol::Error> for Error {
    fn from(err: amp_protocol::Error) -> Self {
        match err {
            amp_protocol::Error::Io(err) => Error::Io(err),
            other => Error::Protocol(other),
        }
    }
}

impl From<serde_amp::Error> for Error {
    fn from(err: serde_amp::Error) -> Self {
        Error::Serde(err)
    }
}

/// Lets a service wrapped in tower middleware, whose error type is a boxed
/// error, be served. An `Error` inside the box comes back out unchanged;
/// anything else, such as a timeout, becomes an `UNKNOWN` error carrying its
/// message.
impl From<tower::BoxError> for Error {
    fn from(err: tower::BoxError) -> Self {
        match err.downcast::<Error>() {
            Ok(err) => *err,
            Err(err) => Error::new(ErrorResponse::UNKNOWN, err.to_string()),
        }
    }
}
