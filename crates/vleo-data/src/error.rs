//! Why reference data could not be read, installed or used — by kind.
//!
//! A caller decides by the kind and shows the message; it never reads the
//! sentence to find out what went wrong. The message is the one this crate has
//! always given, word for word, so everything a person or the wire sees is
//! unchanged.

use std::fmt;
use std::path::Path;

/// What went wrong, as a caller decides by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The file system refused: a read, a write or a listing failed.
    Io,
    /// Where the data was asked for, there is none.
    Missing,
    /// A bundle is present and does not verify, so nothing in it is read.
    Unverified,
    /// A manifest or a table does not say what it must, or not as it must.
    Malformed,
    /// A setting in the environment cannot be read as what it names.
    Setting,
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

    /// The file system refused at `path`, said as `path: why`.
    pub(crate) fn io(path: &Path, e: impl fmt::Display) -> Error {
        Error::new(ErrorKind::Io, format!("{}: {e}", path.display()))
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

impl std::error::Error for Error {}

/// For a caller that still passes its errors on as text: the sentence.
impl From<Error> for String {
    fn from(e: Error) -> String {
        e.message
    }
}
