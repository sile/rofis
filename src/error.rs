use fibers_http_server::{self, Status};
use std::io;
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt, TrackableError};

/// This crate specific `Error` type.
#[derive(Debug, Clone)]
pub struct Error(TrackableError<ErrorKind>);
derive_traits_for_trackable_error_newtype!(Error, ErrorKind);
impl Error {
    pub(crate) fn to_http_status(&self) -> Status {
        match self.kind() {
            ErrorKind::NotFound => Status::NotFound,
            ErrorKind::InternalServerError => Status::InternalServerError,
        }
    }
}
impl From<io::Error> for Error {
    fn from(f: io::Error) -> Self {
        let kind = match f.kind() {
            io::ErrorKind::NotFound => ErrorKind::NotFound,
            _ => ErrorKind::InternalServerError,
        };
        kind.cause(f).into()
    }
}
impl From<fibers_http_server::Error> for Error {
    fn from(f: fibers_http_server::Error) -> Self {
        ErrorKind::InternalServerError.cause(f).into()
    }
}

/// Possible error kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ErrorKind {
    NotFound,
    InternalServerError,
}
impl TrackableErrorKind for ErrorKind {}
