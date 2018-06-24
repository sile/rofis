extern crate bytecodec;
extern crate fibers_http_server;
extern crate futures;
extern crate httpcodec;
extern crate mime_guess;
#[macro_use]
extern crate trackable;
#[macro_use]
extern crate slog;

pub use file_server::FileServer;

mod file_server;
