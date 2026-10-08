#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod client;
mod command;
mod error;
mod handle;
mod router;
mod server;

/// The box type that carries a command's arguments and results, re-exported
/// from `amp-protocol` so that services use the version this crate was built
/// against.
pub use amp_protocol::AmpBox;
/// The request type that services accept, re-exported from `amp-protocol`
/// for the same reason as [`AmpBox`].
pub use amp_protocol::Request;
pub use client::Client;
pub use command::Command;
pub use error::Error;
pub use handle::{Handle, HandleWith};
pub use router::Router;
pub use server::{serve, serve_connection};
/// Declares a struct as a command's arguments. See [`Command`].
pub use tower_amp_macros::command;
