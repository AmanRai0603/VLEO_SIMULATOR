//! Why a design file, a key or a signature was refused — by kind.
//!
//! A caller decides by the kind and shows the message; it never reads the
//! sentence to find out what went wrong.

use std::fmt;

/// What went wrong, as a caller decides by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Text that should hold a key, a signature or a fingerprint does not.
    Malformed,
    /// The passphrase does not unlock the key.
    Passphrase,
    /// The key was changed after it was locked: it is not the key that was made.
    Tampered,
    /// A signature does not check against the key it names.
    Signature,
    /// The operating system gave no randomness to make a key with.
    Randomness,
    /// The file is not the kind it was opened as, or not a VLEO file at all.
    WrongKind,
    /// The file is in a format this application does not read as it is.
    Format,
    /// The file could not be read or written.
    Io,
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

    /// What went wrong.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The sentence a person reads.
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
