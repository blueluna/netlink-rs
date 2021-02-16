use std::error;
use std::fmt;
use std::io;
use std::result;
use std::str;
use std::string;

#[derive(Debug)]
pub enum NetlinkErrorKind {
    NotEnoughData,
    NotFound,
    InvalidValue,
    InvalidLength,
}

#[derive(Debug)]
pub struct NetlinkError {
    pub kind: NetlinkErrorKind,
}

impl NetlinkError {
    pub fn new(kind: NetlinkErrorKind) -> NetlinkError {
        NetlinkError { kind: kind }
    }
}

impl fmt::Display for NetlinkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NetlinkError {:?}", self.kind)
    }
}

impl error::Error for NetlinkError {
    fn description(&self) -> &str {
        "NetlinkError"
    }
}

/// Errors signaling issues with the Netlink communication
#[derive(Debug)]
pub enum Error {
    /// An std::io error has occured
    Io(io::Error),
    /// A str UTF-8 error has occured
    Utf8(str::Utf8Error),
    /// An UTF-8 string conversion error has occured
    FromUtf8(string::FromUtf8Error),
    /// A Netlink transport error has occured
    Netlink(NetlinkError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Utf8(ref err) => write!(f, "UTF8 error: {}", err),
            Error::FromUtf8(ref err) => write!(f, "From UTF8 error: {}", err),
            Error::Netlink(ref err) => write!(f, "Pack error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
            Error::FromUtf8(ref err) => Some(err),
            Error::Netlink(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<NetlinkError> for Error {
    fn from(err: NetlinkError) -> Error {
        Error::Netlink(err)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error::Utf8(err)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error::FromUtf8(err)
    }
}

/// Result alias for crate errors
pub type Result<T> = result::Result<T, Error>;
