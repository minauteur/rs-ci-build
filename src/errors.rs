//! Errors module.

use iron::IronError;
use iron::status;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::string::FromUtf8Error;
use urlencoded;

/// `CIError` combines errors from the `urlencoded` driver and our custom
/// errors for the CI/build system.
#[derive(Debug)]
pub enum CIError {
    UrlDecoding(urlencoded::UrlDecodingError),
    CmdIo(io::Error),
    Utf8(FromUtf8Error),
    Unknown(String),
}
impl Error for CIError {
    fn description(&self) -> &str {
        match *self {
            CIError::UrlDecoding(ref e) => e.description(),
            CIError::CmdIo(ref e) => e.description(),
            CIError::Utf8(ref e) => e.description(),
            CIError::Unknown(ref e) => e,
        }
    }
}
impl Display for CIError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CIError::UrlDecoding(ref e) => write!(f, "{}", e.description()),
            CIError::CmdIo(ref e) => write!(f, "{}", e.description()),
            CIError::Utf8(ref e) => write!(f, "{}", e.description()),
            CIError::Unknown(ref e) => write!(f, "{}", e),
        }
    }
}

impl From<FromUtf8Error> for CIError {
    fn from(err: FromUtf8Error) -> CIError {
        CIError::Utf8(err)
    }
}
impl From<io::Error> for CIError {
    fn from(err: io::Error) -> CIError {
        CIError::CmdIo(err)
    }
}
impl From<urlencoded::UrlDecodingError> for CIError {
    fn from(err: urlencoded::UrlDecodingError) -> CIError {
        CIError::UrlDecoding(err)
    }
}
impl From<CIError> for IronError {
    fn from(err: CIError) -> IronError {
        IronError::new(err, status::InternalServerError)
    }
}
/// Temporary `Error` type for reporting `try_update_logger` errors.
/// This should be replaced with a more generic error type in the future.
// FIXME: Replace with more generic error-ific type
pub enum LoggerError {
    NoLogger,
}
