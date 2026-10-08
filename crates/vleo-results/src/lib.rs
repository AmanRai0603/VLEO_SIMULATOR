//! `results.vleor` — saved results, many in one file.
//!
//! A results folder keeps one folder per result, which is right for a laptop
//! and awkward to send. This file holds any number of them: each whole, as the
//! CSV its folder keeps, so `import` puts back exactly what `export` took; and
//! every value again as a row of `value`, so Python or any SQL reader can ask
//! for numbers without parsing CSV. Its schema is `results.sql`, beside this
//! crate.
//!
//! The crate held the design as one file, `design.vleo`, too, until the
//! design's own files (`design/`) became what every face reads.
//!
//! This crate knows the file, not what a result means: the command line turns
//! each saved result into a [`Kept`] and back, with the engine's own reader.

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use std::fmt;
use std::path::Path;

/// What went wrong, as a caller decides by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The file system refused.
    Io,
    /// Not the file asked for: not SQLite, not one the tools wrote, or another
    /// kind (a group's release opened as a results file).
    WrongFile,
    /// The file is a newer format than this code reads.
    Newer,
    /// SQLite refused.
    Database,
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
    fn db(path: &Path, e: rusqlite::Error) -> Error {
        Error::new(ErrorKind::Database, format!("{}: {e}", path.display()))
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

impl From<Error> for String {
    fn from(e: Error) -> String {
        e.message
    }
}

/// Open one of the tools' databases read-only, as the kind `kind`, by the one
/// rule every reader follows (`vleo_kinds::identify`): refused if it is not
/// SQLite, not one the tools wrote, another kind (a group's release opened
/// as results), or from a newer tool — each by name, before anything is read
/// from it.
fn open_ours(path: &Path, kind: &str, called: &str) -> Result<Connection, Error> {
    let mut head = [0u8; 16];
    {
        use std::io::Read;
        let mut f = std::fs::File::open(path)
            .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", path.display())))?;
        if f.read_exact(&mut head).is_err() || &head[..15] != b"SQLite format 3" {
            return Err(Error::new(
                ErrorKind::WrongFile,
                format!("{}: not a database file", path.display()),
            ));
        }
    }
    let db = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| Error::db(path, e))?;
    let pragma = |name: &str| -> Result<i64, Error> {
        db.query_row(&format!("PRAGMA {name}"), [], |r| r.get(0))
            .map_err(|e| Error::db(path, e))
    };
    let (app, format) = (pragma("application_id")?, pragma("user_version")?);
    let found: Option<String> = if app == vleo_kinds::APPLICATION_ID {
        db.query_row("SELECT value FROM meta WHERE key = 'file_kind'", [], |r| {
            r.get(0)
        })
        .optional()
        .unwrap_or(None)
    } else {
        None
    };
    let want = vleo_kinds::Want {
        reader: vleo_kinds::Reader::Results,
        names: &[kind],
        called,
    };
    match vleo_kinds::identify(app, format, found.as_deref(), want) {
        Ok(_) => Ok(db),
        Err(refused) => Err(Error::new(
            match refused {
                vleo_kinds::Refusal::Newer { .. } => ErrorKind::Newer,
                _ => ErrorKind::WrongFile,
            },
            refused.says(&path.display().to_string()),
        )),
    }
}

/// The format this code writes and the newest it reads.
pub const FORMAT: u32 = 1;
/// What the file calls itself in `meta`.
pub const KIND: &str = "results";
const SCHEMA: &str = include_str!("../results.sql");

/// One row of a result, as the `value` table holds it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Value {
    /// `input`, `output` or `blocked`.
    pub section: String,
    pub id: String,
    pub name: String,
    pub value: String,
    pub unit: String,
    pub si: Option<f64>,
    pub credibility: String,
    pub governing: String,
    pub note: String,
}

/// One saved result, as the file holds it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Kept {
    /// Its folder's name in a results folder.
    pub name: String,
    pub question: String,
    pub target: String,
    pub mode: String,
    pub saved: String,
    /// The name somebody gave it, or nothing.
    pub label: String,
    pub chain: String,
    pub kernel: String,
    pub graph: String,
    pub case: String,
    pub ran: usize,
    pub blocked: usize,
    pub thinned: String,
    pub pinned: bool,
    /// The result whole, as `result.csv`.
    pub csv: String,
    /// Its sweep, as `sweep.csv`, or nothing.
    pub sweep_csv: String,
    pub values: Vec<Value>,
}

/// Write `kept` to `out`, replacing whatever was there, whole: the file is
/// written beside `out` and renamed into place.
pub fn write(out: &Path, kept: &[Kept], tool: &str, written: &str) -> Result<(), Error> {
    let db_err =
        |e: rusqlite::Error| Error::new(ErrorKind::Database, format!("{}: {e}", out.display()));
    let part = out.with_extension("vleor.part");
    let _ = std::fs::remove_file(&part);
    if let Some(dir) = out.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)
                .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", dir.display())))?;
        }
    }
    let mut db = Connection::open(&part).map_err(db_err)?;
    db.execute_batch(SCHEMA).map_err(db_err)?;
    {
        let tx = db.transaction().map_err(db_err)?;
        {
            let mut put = tx
                .prepare(
                    "INSERT INTO result (name, question, target, mode, saved, label, chain, kernel, \
                     graph, case_id, ran, blocked, thinned, pinned, csv, sweep_csv) VALUES \
                     (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                )
                .map_err(db_err)?;
            let mut val = tx
                .prepare(
                    "INSERT INTO value (result, section, ord, id, name, value, unit, si, \
                     credibility, governing, note) VALUES \
                     (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                )
                .map_err(db_err)?;
            for k in kept {
                put.execute(params![
                    k.name,
                    k.question,
                    k.target,
                    k.mode,
                    k.saved,
                    k.label,
                    k.chain,
                    k.kernel,
                    k.graph,
                    k.case,
                    k.ran as i64,
                    k.blocked as i64,
                    k.thinned,
                    k.pinned,
                    k.csv,
                    k.sweep_csv
                ])
                .map_err(db_err)?;
                for (i, v) in k.values.iter().enumerate() {
                    val.execute(params![
                        k.name,
                        v.section,
                        i as i64,
                        v.id,
                        v.name,
                        v.value,
                        v.unit,
                        v.si,
                        v.credibility,
                        v.governing,
                        v.note
                    ])
                    .map_err(db_err)?;
                }
            }
        }
        for (k, v) in [
            ("file_kind", KIND.to_string()),
            ("format", FORMAT.to_string()),
            ("tool", tool.to_string()),
            ("written", written.to_string()),
            ("results", kept.len().to_string()),
        ] {
            tx.execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)",
                params![k, v],
            )
            .map_err(db_err)?;
        }
        tx.commit().map_err(db_err)?;
    }
    db.close().map_err(|(_, e)| db_err(e))?;
    std::fs::rename(&part, out)
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", out.display())))
}

/// Every result in `file`, in the order they were written. Refuses a file that
/// is not SQLite, not ours, not a results file, or newer than this code.
pub fn read(file: &Path) -> Result<Vec<Kept>, Error> {
    let db = open_ours(file, KIND, "a results file")?;
    let db_err =
        |e: rusqlite::Error| Error::new(ErrorKind::Database, format!("{}: {e}", file.display()));
    let mut out = Vec::new();
    {
        let mut q = db
            .prepare(
                "SELECT name, question, target, mode, saved, label, chain, kernel, graph, case_id, \
                 ran, blocked, thinned, pinned, csv, sweep_csv FROM result ORDER BY rowid",
            )
            .map_err(db_err)?;
        let rows = q
            .query_map([], |r| {
                Ok(Kept {
                    name: r.get(0)?,
                    question: r.get(1)?,
                    target: r.get(2)?,
                    mode: r.get(3)?,
                    saved: r.get(4)?,
                    label: r.get(5)?,
                    chain: r.get(6)?,
                    kernel: r.get(7)?,
                    graph: r.get(8)?,
                    case: r.get(9)?,
                    ran: r.get::<_, i64>(10)? as usize,
                    blocked: r.get::<_, i64>(11)? as usize,
                    thinned: r.get(12)?,
                    pinned: r.get(13)?,
                    csv: r.get(14)?,
                    sweep_csv: r.get(15)?,
                    values: Vec::new(),
                })
            })
            .map_err(db_err)?;
        for k in rows {
            out.push(k.map_err(db_err)?);
        }
    }
    let mut q = db
        .prepare(
            "SELECT section, id, name, value, unit, si, credibility, governing, note FROM value \
             WHERE result = ?1 ORDER BY ord",
        )
        .map_err(db_err)?;
    for k in &mut out {
        let rows = q
            .query_map(params![k.name], |r| {
                Ok(Value {
                    section: r.get(0)?,
                    id: r.get(1)?,
                    name: r.get(2)?,
                    value: r.get(3)?,
                    unit: r.get(4)?,
                    si: r.get(5)?,
                    credibility: r.get(6)?,
                    governing: r.get(7)?,
                    note: r.get(8)?,
                })
            })
            .map_err(db_err)?;
        for v in rows {
            k.values.push(v.map_err(db_err)?);
        }
    }
    Ok(out)
}

/// What a results file says about itself, without reading its results.
pub fn meta(file: &Path, key: &str) -> Result<String, Error> {
    let db = open_ours(file, KIND, "a results file")?;
    db.query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| {
        r.get(0)
    })
    .optional()
    .map(Option::unwrap_or_default)
    .map_err(|e| Error::new(ErrorKind::Database, format!("{}: {e}", file.display())))
}

/// Whether a file starts as a SQLite database does — how a face tells a
/// results file from a result's CSV before reading either.
pub fn is_database(file: &Path) -> bool {
    use std::io::Read;
    let mut head = [0u8; 16];
    std::fs::File::open(file)
        .and_then(|mut f| f.read_exact(&mut head))
        .is_ok()
        && &head[..15] == b"SQLite format 3"
}
