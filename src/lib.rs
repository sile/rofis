extern crate bytecodec;
extern crate fibers_http_server;
extern crate futures;
extern crate httpcodec;
#[macro_use]
extern crate trackable;

pub use file_server::FileServer;

mod file_server;
