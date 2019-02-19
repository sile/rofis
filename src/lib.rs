//! A read-only, puny HTTP file server.
#![warn(missing_docs)]
#[macro_use]
extern crate trackable;
#[macro_use]
extern crate slog;

pub use error::{Error, ErrorKind};
pub use file_server::FileServer;

mod error;
mod file_server;

/// This crate specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;
