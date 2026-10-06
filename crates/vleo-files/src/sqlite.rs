//! A design file on disk, installed: SQLite, through rusqlite.
//!
//! The page has its own SQLite and hands the library the rows it read
//! ([`File::from_tables`]); installed, the library reads and writes the file
//! itself, here. Both go through the same tables, so what one writes the
//! other reads.

use std::path::Path;

use rusqlite::types::{Value, ValueRef};
use rusqlite::{params_from_iter, Connection, OpenFlags};

use crate::error::{Error, ErrorKind};
use crate::meta::FORMAT;
use crate::model::{Cell, File, Table};

/// The schema every file is made with.
pub const SCHEMA: &str = include_str!("schema.sql");

/// SQLite's `application_id` for every VLEO file — the number the group files
/// have carried since format 1 (groups/schema.sql), kept so one test tells a
/// VLEO file from any other database whatever its format.
pub const APPLICATION_ID: i64 = 1_447_838_031;

fn io(path: &Path, what: &str, e: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Io, format!("{}: {what}: {e}", path.display()))
}

/// Write `file` to `path`, whole: to a new file beside it first, then moved
/// over the old one, so a failure part-way leaves the old file as it was.
pub fn write(file: &File, path: &Path) -> Result<(), Error> {
    file.check_values()?;
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
/// older one (it is upgraded first, keeping a copy, by `crate::upgrade`).
pub fn read(path: &Path) -> Result<File, Error> {
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| io(path, "could not be opened", e))?;
    let pragma = |name: &str| -> Result<i64, Error> {
        db.query_row(&format!("PRAGMA {name}"), [], |r| r.get(0))
            .map_err(|e| io(path, "is not a SQLite database", e))
    };
    if pragma("application_id")? != APPLICATION_ID {
        return Err(Error::new(
            ErrorKind::WrongKind,
            format!("{} is not a VLEO design file", path.display()),
        ));
    }
    let format = pragma("user_version")?;
    if format > FORMAT {
        return Err(Error::new(
            ErrorKind::Format,
            format!(
                "{} is format {format}, newer than this application reads ({FORMAT}): install the newer application",
                path.display()
            ),
        ));
    }
    if format < FORMAT {
        return Err(Error::new(
            ErrorKind::Format,
            format!(
                "{} is format {format}, from before this one ({FORMAT}): it is upgraded when opened, keeping a copy",
                path.display()
            ),
        ));
    }
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
    File::from_tables(tables)
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
