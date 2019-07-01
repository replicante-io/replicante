use std::fmt;

use failure::Backtrace;
use failure::Context;
use failure::Fail;

/// Error information returned by functions in case of errors.
#[derive(Debug)]
pub struct Error(Context<ErrorKind>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.0.get_context()
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error(inner)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(Context::new(kind))
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }

    fn name(&self) -> Option<&str> {
        self.kind().kind_name()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Exhaustive list of possible errors emitted by this crate.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "I/O Error for '{}'", _0)]
    Io(String),

    #[fail(display = "invalid YAML read from '{}'", _0)]
    YamlFile(String),
}

impl ErrorKind {
    fn kind_name(&self) -> Option<&str> {
        let name = match self {
            ErrorKind::Io(_) => "Io",
            ErrorKind::YamlFile(_) => "YamlFile",
        };
        Some(name)
    }
}

/// Short form alias for functions returning `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;