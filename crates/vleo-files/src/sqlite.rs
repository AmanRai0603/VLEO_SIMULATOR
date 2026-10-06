//! A design file on disk, installed: SQLite, through rusqlite.
//!
//! The page has its own SQLite and hands the library the rows it read
//! ([`File::from_tables`]); installed, the library reads and writes the file
//! itself, here. Both go through the same tables, so what one writes the
//! other reads.

use std::path::{Path, PathBuf};

use rusqlite::types::{Value, ValueRef};
use rusqlite::{params_from_iter, Connection, OpenFlags};

use crate::error::{Error, ErrorKind};
use crate::format_1;
use crate::meta::FORMAT;
use crate::model::{Cell, File, Table};
use crate::upgrade::{self, Upgrade};

/// The schema every file is made with.
pub const SCHEMA: &str = include_str!("schema.sql");

/// SQLite's `application_id` for every VLEO file — the number the group files
/// have carried since format 1 (groups/schema.sql), kept so one test tells a
/// VLEO file from any other database whatever its format.
pub const APPLICATION_ID: i64 = vleo_kinds::APPLICATION_ID;

fn io(path: &Path, what: &str, e: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Io, format!("{}: {what}: {e}", path.display()))
}

/// Write `file` to `path`, whole: to a new file beside it first, then moved
/// over the old one, so a failure part-way leaves the old file as it was.
///
/// A file in an older format at `path` is kept beside it first
/// ([`kept_copy`]), so saving an upgraded file never loses the one it was
/// upgraded from. An older copy already there is never written over.
pub fn write(file: &File, path: &Path) -> Result<(), Error> {
    file.check_values()?;
    keep_older(path)?;
    let tmp = path.with_extension("partial");
    let _ = std::fs::remove_file(&tmp);
    {
        let db = Connection::open(&tmp).map_err(|e| io(&tmp, "could not be made", e))?;
        db.execute_batch(SCHEMA)
            .map_err(|e| io(&tmp, "the schema could not be written", e))?;
        let tx = db
            .unchecked_transaction()
            .map_err(|e| io(&tmp, "could not be written", e))?;
        for t in file.to_tables() {
            insert(&tx, &t).map_err(|e| io(&tmp, &format!("the table {}", t.name), e))?;
        }
        tx.commit()
            .map_err(|e| io(&tmp, "could not be written", e))?;
    }
    std::fs::rename(&tmp, path).map_err(|e| io(path, "could not be replaced", e))
}

fn keep_older(path: &Path) -> Result<(), Error> {
    if !path.exists() {
        return Ok(());
    }
    let format = {
        let Ok(db) = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
            return Ok(());
        };
        match format_of(&db, path) {
            Ok(f) => f,
            // Not a VLEO file, or a newer one: not this library's to keep.
            Err(_) => return Ok(()),
        }
    };
    if format >= FORMAT {
        return Ok(());
    }
    let copy = kept_copy(path, format);
    let bytes = std::fs::read(path).map_err(|e| io(path, "could not be read", e))?;
    if copy.exists() {
        let there = std::fs::read(&copy).map_err(|e| io(&copy, "could not be read", e))?;
        if there != bytes {
            return Err(Error::new(
                ErrorKind::Io,
                format!(
                    "{} is kept from before, and is not the file being saved over: neither is touched",
                    copy.display()
                ),
            ));
        }
        return Ok(());
    }
    std::fs::write(&copy, &bytes).map_err(|e| io(&copy, "could not be kept", e))
}

fn insert(db: &Connection, t: &Table) -> rusqlite::Result<()> {
    if t.rows.is_empty() {
        return Ok(());
    }
    let marks = vec!["?"; t.columns.len()].join(", ");
    let sql = format!(
        "INSERT INTO \"{}\" ({}) VALUES ({marks})",
        t.name,
        t.columns
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut st = db.prepare(&sql)?;
    for row in &t.rows {
        st.execute(params_from_iter(row.iter().map(|c| match c {
            Cell::Null => Value::Null,
            Cell::Int(i) => Value::Integer(*i),
            Cell::Text(s) => Value::Text(s.clone()),
            Cell::Blob(b) => Value::Blob(b.clone()),
        })))?;
    }
    Ok(())
}

/// The file at `path`, read whole. Refused, by name, if it is not a VLEO file,
/// if it is in a newer format than this library knows, or if it is in an
/// older one ([`open`] upgrades that).
pub fn read(path: &Path) -> Result<File, Error> {
    let (format, tables) = raw(path)?;
    if format < FORMAT {
        return Err(Error::new(
            ErrorKind::Format,
            format!(
                "{} is format {format}, from before this one ({FORMAT}): it is upgraded when opened, keeping a copy",
                path.display()
            ),
        ));
    }
    File::from_tables(tables)
}

/// The format-1 file at `path`, as it is: a group's file from before 1.0.
pub fn read_format_1(path: &Path) -> Result<format_1::Old, Error> {
    let (format, tables) = raw(path)?;
    if format != format_1::FORMAT {
        return Err(Error::new(
            ErrorKind::Format,
            format!("{} is format {format}, not format 1", path.display()),
        ));
    }
    format_1::Old::from_tables(tables)
}

/// A file opened: as it is, or upgraded from an older format.
#[derive(Debug)]
pub struct Opened {
    pub file: File,
    /// The format it was in on disk.
    pub format: i64,
    /// What the upgrade said, when there was one.
    pub said: Vec<String>,
}

/// The file at `path`, opened by the application: a file in this format as it
/// is, one in an older format upgraded with nothing dropped
/// (`crate::upgrade`). The file on disk is not touched; it becomes this format
/// when it is saved, and [`write`] keeps the old one beside it.
pub fn open(path: &Path, how: &Upgrade) -> Result<Opened, Error> {
    let (format, tables) = raw(path)?;
    if format == FORMAT {
        return Ok(Opened {
            file: File::from_tables(tables)?,
            format,
            said: Vec::new(),
        });
    }
    if format != format_1::FORMAT {
        return Err(Error::new(
            ErrorKind::Format,
            format!(
                "{} is format {format}, which this application has no upgrade from",
                path.display()
            ),
        ));
    }
    let up = upgrade::from_format_1(tables, how)?;
    Ok(Opened {
        file: up.file,
        format,
        said: up.said,
    })
}

/// Where the old file is kept when a file in an older format is saved over:
/// beside it, its format in its name — `solar-1.1.vleo` keeps
/// `solar-1.1.format-1.vleo`.
pub fn kept_copy(path: &Path, format: i64) -> PathBuf {
    let stem = path
        .file_stem()
        .map_or(String::new(), |s| s.to_string_lossy().into_owned());
    let name = match path.extension() {
        Some(ext) => format!("{stem}.format-{format}.{}", ext.to_string_lossy()),
        None => format!("{stem}.format-{format}"),
    };
    path.with_file_name(name)
}

/// The format of the VLEO file at `path`, and its tables as they are.
fn raw(path: &Path) -> Result<(i64, Vec<Table>), Error> {
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| io(path, "could not be opened", e))?;
    let format = format_of(&db, path)?;
    let mut names: Vec<String> = Vec::new();
    {
        let mut st = db
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .map_err(|e| io(path, "could not be read", e))?;
        let rows = st
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| io(path, "could not be read", e))?;
        for n in rows {
            names.push(n.map_err(|e| io(path, "could not be read", e))?);
        }
    }
    let mut tables = Vec::new();
    for name in names {
        tables.push(table(&db, &name).map_err(|e| io(path, &format!("the table {name}"), e))?);
    }
    Ok((format, tables))
}

/// Every kind this library reads, by name: the one schema's, and a group's
/// files from before 1.0, which it upgrades.
fn read_here() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();
    for k in vleo_kinds::KINDS {
        if k.reader == vleo_kinds::Reader::Files && !names.contains(&k.name) {
            names.push(k.name);
        }
    }
    names
}

/// The format of an open VLEO file, refused by the one rule every reader
/// follows (`vleo_kinds::identify`) if it is not one, is a kind this library
/// does not read — today's design, `design.vleo`, is the tool's own until
/// phase D — or is newer than this library reads.
fn format_of(db: &Connection, path: &Path) -> Result<i64, Error> {
    let pragma = |name: &str| -> Result<i64, Error> {
        db.query_row(&format!("PRAGMA {name}"), [], |r| r.get(0))
            .map_err(|e| io(path, "is not a SQLite database", e))
    };
    let (app, format) = (pragma("application_id")?, pragma("user_version")?);
    let kind: Option<String> = if app == APPLICATION_ID {
        db.query_row("SELECT value FROM meta WHERE key = 'file_kind'", [], |r| {
            r.get(0)
        })
        .ok()
    } else {
        None
    };
    let names = read_here();
    let want = vleo_kinds::Want {
        reader: vleo_kinds::Reader::Files,
        names: &names,
        called: "a file in the one schema",
    };
    match vleo_kinds::identify(app, format, kind.as_deref(), want) {
        Ok(_) => Ok(format),
        Err(refused) => Err(Error::new(
            match refused {
                vleo_kinds::Refusal::Newer { .. } => ErrorKind::Format,
                _ => ErrorKind::WrongKind,
            },
            refused.says(&path.display().to_string()),
        )),
    }
}

fn table(db: &Connection, name: &str) -> rusqlite::Result<Table> {
    let mut st = db.prepare(&format!("SELECT * FROM \"{name}\" ORDER BY rowid"))?;
    let columns: Vec<String> = st.column_names().iter().map(|c| c.to_string()).collect();
    let n = columns.len();
    let rows = st
        .query_map([], |r| {
            (0..n)
                .map(|i| {
                    Ok(match r.get_ref(i)? {
                        ValueRef::Null => Cell::Null,
                        ValueRef::Integer(v) => Cell::Int(v),
                        ValueRef::Real(v) => Cell::Text(v.to_string()),
                        ValueRef::Text(t) => Cell::Text(String::from_utf8_lossy(t).into_owned()),
                        ValueRef::Blob(b) => Cell::Blob(b.to_vec()),
                    })
                })
                .collect::<rusqlite::Result<Vec<Cell>>>()
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Table {
        name: name.to_string(),
        columns,
        rows,
    })
}
