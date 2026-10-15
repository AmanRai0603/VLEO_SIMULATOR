//! The copy's record: where a person's working copy of the database came
//! from, and every file as it was copied (docs/PLAN_F.md, decision 3).
//!
//! The database lives on the shared drive. Each person copies the latest into
//! the folder beside the program and works on that copy; the program puts back
//! only the files that person signed, sealed or answered, and never over one
//! that changed on the drive since the copy. Both halves of that rule read
//! this record: a file in the copy whose bytes are not what was copied is the
//! person's own work, and a file on the drive whose bytes are not what was
//! copied is someone else's.
//!
//! It is written beside the copy as `copied.toml`, by the program, never by
//! hand. Each file is named by its path inside the database, with `/` between
//! folders whatever the system, and the SHA-256 of its bytes.

use crate::error::{Error, ErrorKind};
use crate::keys::{hex, sha256};
use std::collections::BTreeMap;

/// The file name the record is kept under, beside the copy's own files.
pub const FILE_NAME: &str = "copied.toml";

/// Where a copy came from, when, and each file's digest as it was copied.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Copied {
    /// The drive folder the copy was taken from, as this computer names it.
    pub drive: String,
    /// When it was taken: YYYY-MM-DDTHH:MM:SSZ.
    pub copied_at: String,
    /// Each file's path inside the database, and the SHA-256 of its bytes.
    pub files: BTreeMap<String, String>,
}

/// The SHA-256 of `bytes`, as the record writes it.
pub fn digest(bytes: &[u8]) -> String {
    hex(&sha256(bytes))
}

fn malformed(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

/// A path the record may hold: relative, with `/` between its parts, and no
/// part that is empty, `.` or `..`. A record that names anything else would
/// let a copy or a put-back reach outside the database.
pub fn path_ok(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.starts_with('/')
        && !path.contains(':')
        && path
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
}

impl Copied {
    /// The record as `copied.toml` holds it.
    pub fn to_toml(&self) -> String {
        let q = |s: &str| toml::Value::String(s.to_string()).to_string();
        let mut out = String::from(
            "# Written by the program when this copy was taken from the drive.\n\
             # It is how the program knows what changed, here and on the drive,\n\
             # since. Never edit it.\n",
        );
        out.push_str(&format!("drive = {}\n", q(&self.drive)));
        out.push_str(&format!("copied_at = {}\n", q(&self.copied_at)));
        for (path, sha) in &self.files {
            out.push_str(&format!(
                "\n[[file]]\npath = {}\nsha256 = {}\n",
                q(path),
                q(sha)
            ));
        }
        out
    }

    /// The record `copied.toml` holds, refused by name when it is not one.
    pub fn from_toml(text: &str) -> Result<Copied, Error> {
        let v: toml::Value = text
            .parse()
            .map_err(|e| malformed(format!("{FILE_NAME} is not TOML: {e}")))?;
        let s = |k: &str| -> Result<String, Error> {
            v.get(k)
                .and_then(|x| x.as_str())
                .map(str::to_string)
                .ok_or_else(|| malformed(format!("{FILE_NAME} does not say its {k}")))
        };
        let mut out = Copied {
            drive: s("drive")?,
            copied_at: s("copied_at")?,
            files: BTreeMap::new(),
        };
        crate::chain::instant(&out.copied_at, &format!("{FILE_NAME}'s copied_at"))?;
        for f in v
            .get("file")
            .and_then(|x| x.as_array())
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            let path = f.get("path").and_then(|x| x.as_str()).unwrap_or("");
            let sha = f.get("sha256").and_then(|x| x.as_str()).unwrap_or("");
            if !path_ok(path) {
                return Err(malformed(format!(
                    "{FILE_NAME} names {path:?}, which is not a path inside the database"
                )));
            }
            if sha.len() != 64 || !sha.bytes().all(|c| c.is_ascii_hexdigit()) {
                return Err(malformed(format!(
                    "{FILE_NAME} gives {path} the digest {sha:?}, which is not a SHA-256"
                )));
            }
            if out
                .files
                .insert(path.to_string(), sha.to_ascii_lowercase())
                .is_some()
            {
                return Err(malformed(format!("{FILE_NAME} names {path} twice")));
            }
        }
        Ok(out)
    }

    /// The paths whose bytes in `now` are not what was copied: changed, added
    /// since, or gone. `now` maps each file's path to its digest as it is.
    pub fn changed(&self, now: &BTreeMap<String, String>) -> Vec<String> {
        let mut out: Vec<String> = now
            .iter()
            .filter(|(p, sha)| self.files.get(*p) != Some(*sha))
            .map(|(p, _)| p.clone())
            .collect();
        out.extend(self.files.keys().filter(|p| !now.contains_key(*p)).cloned());
        out.sort();
        out
    }
}
