use chrono;
use reqwest;
use std::{fmt, io};
use zohohorrorshow;

use failure::{Backtrace, Context, Fail};

/// A type alias for handling errors throughout ZohoHorrorshow.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur while interacting with the Zoho API.
#[derive(Debug)]
pub struct Error {
    ctx: Context<ErrorKind>,
}

impl Error {
    /// Return the kind of this error.
    // pub fn kind(&self) -> &ErrorKind {
    //     self.ctx.get_context()
    // }

    pub(crate) fn chrono_parse(err: chrono::ParseError) -> Error {
        Error::from(ErrorKind::Chrono(err.to_string()))
    }

    pub(crate) fn io(err: io::Error) -> Error {
        Error::from(ErrorKind::IOError(err.to_string()))
    }

    pub(crate) fn zohohorrorshow(err: zohohorrorshow::errors::Error) -> Error {
        Error::from(ErrorKind::ZohoHorrorshow(err.to_string()))
    }

    pub(crate) fn reqwest(err: reqwest::Error) -> Error {
        Error::from(ErrorKind::Reqwest(err.to_string()))
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.ctx.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.ctx.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ctx.fmt(f)
    }
}

pub enum ErrorKind {
    IOError(String),
    ZohoHorrorshow(String),
    Reqwest(String),
    Chrono(String),
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ErrorKind::IOError(ref msg) => write!(f, "{}", msg),
            ErrorKind::ZohoHorrorshow(ref msg) => write!(f, "{}", msg),
            ErrorKind::Reqwest(ref msg) => write!(f, "{}", msg),
            ErrorKind::Chrono(ref msg) => write!(f, "{}", msg),
            _ => panic!("Invalid error kind encountered"),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error::from(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(ctx: Context<ErrorKind>) -> Error {
        Error { ctx }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::reqwest(err)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
        Error::chrono_parse(err)
    }
}

impl From<zohohorrorshow::errors::Error> for Error {
    fn from(err: zohohorrorshow::errors::Error) -> Error {
        Error::zohohorrorshow(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::io(err)
    }
}
