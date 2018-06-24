extern crate bytecodec;
extern crate fibers_http_server;
extern crate futures;
extern crate httpcodec;
extern crate mime_guess;
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
