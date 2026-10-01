//! Why a sweep, a figure or a saved result could not be made, read or kept —
//! by kind.
//!
//! A caller decides by the kind and shows the message; it never reads the
//! sentence to find out what went wrong. The message is the one this crate has
//! always given, word for word, so everything a person or the wire sees is
//! unchanged.

use alloc::format;
use alloc::string::String;
use core::fmt;

/// What went wrong, as a caller decides by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The file system refused: a read, a write, a rename or a removal failed.
    Io,
    /// What was asked for is not there — a result by a name nothing was kept
    /// under.
    Missing,
    /// The request cannot be met as asked: a name that is not a result's, a
    /// sweep over a row that cannot be moved or with too few points.
    Invalid,
    /// What was read does not say what it must: a saved result, a saved sweep
    /// or a figure description that does not hold together.
    Malformed,
}

/// An error from this crate: its kind, and the sentence that says it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Error {
        Error {
            kind,
            message: message.into(),
        }
    }

    /// The same error, said of `what`: `what: message`, the kind kept.
    pub fn within(self, what: impl fmt::Display) -> Error {
        Error {
            kind: self.kind,
            message: format!("{what}: {}", self.message),
        }
    }

    /// The file system refused. A file that is not there is [`ErrorKind::Missing`].
    #[cfg(feature = "std")]
    pub(crate) fn from_io(e: std::io::Error) -> Error {
        let kind = if e.kind() == std::io::ErrorKind::NotFound {
            ErrorKind::Missing
        } else {
            ErrorKind::Io
        };
        Error::new(kind, alloc::string::ToString::to_string(&e))
    }

    /// The file system refused at `what`, said as `what: why`.
    #[cfg(feature = "std")]
    pub(crate) fn io(what: impl fmt::Display, e: std::io::Error) -> Error {
        Error::from_io(e).within(what)
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// For a caller that still passes its errors on as text: the sentence.
impl From<Error> for String {
    fn from(e: Error) -> String {
        e.message
    }
}
