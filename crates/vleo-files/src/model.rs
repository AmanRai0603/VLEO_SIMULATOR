//! A design file in memory: its meta, and one record per row of every table.
//!
//! The tables are `schema.sql`, and nothing here adds to them: each record is
//! one table, its fields that table's columns in the same order, declared once
//! by [`record!`]. A test holds the two together, so a column added to the
//! schema and forgotten here — or the other way round — fails the build's
//! tests rather than a person's file.
//!
//! A file reaches this model as rows: from SQLite directly when installed
//! (`crate::sqlite`), from the page's own SQLite when in the browser. Either
//! way the same [`File::from_tables`] reads them and the same checks run.

use std::collections::BTreeMap;

use crate::error::{Error, ErrorKind};

/// One cell of a row, as SQLite holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Null,
    Int(i64),
    Text(String),
    Blob(Vec<u8>),
}

/// What a field becomes as a cell.
pub trait IntoCell {
    fn into_cell(self) -> Cell;
}

/// What a cell becomes as a field — or why it cannot.
pub trait FromCell: Sized {
    fn from_cell(cell: Cell, table: &str, column: &str) -> Result<Self, Error>;
}

impl IntoCell for String {
    fn into_cell(self) -> Cell {
        Cell::Text(self)
    }
}
impl IntoCell for i64 {
    fn into_cell(self) -> Cell {
        Cell::Int(self)
    }
}
impl IntoCell for Vec<u8> {
    fn into_cell(self) -> Cell {
        Cell::Blob(self)
    }
}
impl IntoCell for Option<String> {
    fn into_cell(self) -> Cell {
        self.map_or(Cell::Null, Cell::Text)
    }
}

fn wrong(table: &str, column: &str, cell: &Cell, want: &str) -> Error {
    Error::new(
        ErrorKind::Malformed,
        format!("{table}.{column} holds {cell:?} where it holds {want}"),
    )
}

impl FromCell for String {
    fn from_cell(cell: Cell, table: &str, column: &str) -> Result<Self, Error> {
        match cell {
            Cell::Text(s) => Ok(s),
            c => Err(wrong(table, column, &c, "text")),
        }
    }
}
impl FromCell for i64 {
    fn from_cell(cell: Cell, table: &str, column: &str) -> Result<Self, Error> {
        match cell {
            Cell::Int(i) => Ok(i),
            c => Err(wrong(table, column, &c, "a whole number")),
        }
    }
}
impl FromCell for Vec<u8> {
    fn from_cell(cell: Cell, table: &str, column: &str) -> Result<Self, Error> {
        match cell {
            Cell::Blob(b) => Ok(b),
            c => Err(wrong(table, column, &c, "bytes")),
        }
    }
}
impl FromCell for Option<String> {
    fn from_cell(cell: Cell, table: &str, column: &str) -> Result<Self, Error> {
        match cell {
            Cell::Null => Ok(None),
            Cell::Text(s) => Ok(Some(s)),
            c => Err(wrong(table, column, &c, "text or nothing")),
        }
    }
}

/// A table's rows as cells, in its columns' order: how a file travels between
/// SQLite (installed or in the page) and this model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
}

/// One table's record.
pub trait Record: Sized {
    const TABLE: &'static str;
    const COLUMNS: &'static [&'static str];
    fn to_row(&self) -> Vec<Cell>;
    fn from_row(row: Vec<Cell>) -> Result<Self, Error>;
}

/// A record per table, its fields the table's columns, in order.
macro_rules! record {
    ($(#[$m:meta])* $name:ident = $table:literal { $($(#[$fm:meta])* $field:ident : $ty:ty),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Debug, Default, PartialEq, Eq)]
        pub struct $name {
            $($(#[$fm])* pub $field: $ty,)*
        }
        impl Record for $name {
            const TABLE: &'static str = $table;
            const COLUMNS: &'static [&'static str] = &[$(stringify!($field)),*];
            fn to_row(&self) -> Vec<Cell> {
                vec![$(IntoCell::into_cell(self.$field.clone())),*]
            }
            fn from_row(row: Vec<Cell>) -> Result<Self, Error> {
                if row.len() != Self::COLUMNS.len() {
                    return Err(Error::new(
                        ErrorKind::Malformed,
                        format!("a row of {} has {} cells, not {}", $table, row.len(), Self::COLUMNS.len()),
                    ));
                }
                let mut cells = row.into_iter();
                Ok($name {
                    $($field: FromCell::from_cell(cells.next().unwrap_or(Cell::Null), $table, stringify!($field))?,)*
                })
            }
        }
    };
}
pub(crate) use record;

record!(
    /// A person a file registers, by their role (OPERATING_1_0, section 2).
    Person = "person" { name: String, role: String, deputy_for: String }
);
record!(
    /// A public key registered for a person, from a date, and from when it
    /// stopped counting if it was revoked.
    PersonKey = "person_key" {
        person: String, public_key: String, registered_at: String,
        registered_by: String, revoked_from: String,
    }
);
record!(
    /// Who writes which block, from which contract version.
    Assignment = "assignment" { block_uid: String, person: String, contract_version: i64 }
);
record!(
    /// A block, to any depth.
    Block = "block" {
        uid: String, id: String, parent_uid: String, question: String,
        behaviour: String, perspective: String, ord: i64, archived: i64,
        contract_version: i64, revision: i64,
    }
);
record!(
    /// An input or output of a block.
    Port = "port" {
        block_uid: String, direction: String, name: String, symbol: String, port_type: String,
        unit: String, lower: String, upper: String, range_reason: String,
        state: String, maturity: String, value: String, choices: String,
        bundle: String, open_owner: String, open_due: String, says: String, ord: i64,
    }
);
record!(
    /// A wire into an input.
    Wire = "wire" { to_block: String, to_port: String, from_ref: String }
);
record!(
    /// A group mounted on a block.
    Mount = "mount" { block_uid: String, group_id: String, release: String }
);
record!(
    /// A requirement closed against an achieved value, and which way it binds.
    Closure = "closure" {
        uid: String, block_uid: String, required_ref: String,
        achieved_ref: String, sense: String, says: String,
    }
);
record!(
    /// A loop declared on the block that holds it.
    Loop = "loop" {
        uid: String, block_uid: String, members: String, settles: String,
        tolerance: String, max_iterations: i64,
    }
);
record!(
    /// A case: evidence that a block gives the answer it should.
    TestCase = "test_case" {
        block_uid: String, name: String, inputs: String, expected: String,
        tolerance: String, provenance: String, source: String,
    }
);
record!(
    /// Markdown, for the file or a block.
    Text = "text" { scope: String, kind: String, body: String }
);
record!(
    /// A table, as CSV.
    Tbl = "tbl" { scope: String, path: String, csv: String }
);
record!(
    /// A file that is not a table.
    Media = "media" {
        scope: String, path: String, media_type: String, sha256: String, bytes: Vec<u8>,
    }
);
record!(
    /// A signature, against the digest of exactly what it signed.
    SignatureRow = "signature" {
        scope: String, revision: String, signer: String, public_key: String,
        digest: String, signature: String, signed_at: String, verdict: String, note: String,
    }
);
record!(
    /// A change, as it was made.
    Change = "change" {
        at: String, who: String, scope: String, what: String,
        before: Option<String>, after: Option<String>,
    }
);
record!(
    /// A request: of a contract, or of the code.
    Request = "request" {
        uid: String, kind: String, scope: String, raised_by: String, at: String,
        body: String, status: String, answer: String,
    }
);
record!(
    /// An issue raised against the design.
    Issue = "issue" {
        number: i64, group_id: String, raised_by: String, at: String, place: String,
        what: String, evidence: String, addressed_to: String, closed_by: String,
    }
);
record!(
    /// A comment on any part.
    Comment = "comment" {
        scope: String, section: String, author: String, at: String, body: String, resolved: i64,
    }
);
record!(
    /// A key file's sealed key.
    LockedKeyRow = "locked_key" {
        person: String, public_key: String, salt: String, nonce: String,
        iterations: i64, sealed: String,
    }
);

/// The values a constrained column may hold, as `schema.sql` says. The page's
/// SQLite holds the same CHECKs; this list is what the library itself refuses
/// by, so a file is refused the same whichever reads it. A test holds it to
/// the schema.
pub const ALLOWED: &[(&str, &str, &[&str])] = &[
    (
        "person",
        "role",
        &[
            "programme manager",
            "system engineer",
            "subsystem engineer",
            "node engineer",
            "developer",
        ],
    ),
    (
        "block",
        "behaviour",
        &["method", "children", "stated", "lookup", "open"],
    ),
    (
        "block",
        "perspective",
        &["", "management", "system", "subsystem"],
    ),
    ("port", "direction", &["in", "out"]),
    (
        "port",
        "port_type",
        &[
            "number",
            "integer",
            "choice",
            "boolean",
            "list",
            "table",
            "series",
            "uncertain",
            "text",
            "file",
        ],
    ),
    (
        "port",
        "state",
        &["decided", "allocated", "open", "achieved"],
    ),
    (
        "port",
        "maturity",
        &["", "estimated", "calculated", "measured"],
    ),
    ("closure", "sense", &["<=", ">="]),
    (
        "test_case",
        "provenance",
        &[
            "independent-derivation",
            "published-source",
            "independent-tool",
            "physical-bound",
            "authors-code",
            "self-snapshot",
            "agent-generated",
        ],
    ),
    ("signature", "verdict", &["ok", "changes"]),
    ("request", "kind", &["contract", "code"]),
    ("request", "status", &["open", "accepted", "declined"]),
];

/// A whole file: its meta, and every row of every table.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct File {
    pub meta: BTreeMap<String, String>,
    pub people: Vec<Person>,
    pub keys: Vec<PersonKey>,
    pub assignments: Vec<Assignment>,
    pub blocks: Vec<Block>,
    pub ports: Vec<Port>,
    pub wires: Vec<Wire>,
    pub mounts: Vec<Mount>,
    pub closures: Vec<Closure>,
    pub loops: Vec<Loop>,
    pub cases: Vec<TestCase>,
    pub texts: Vec<Text>,
    pub tables: Vec<Tbl>,
    pub media: Vec<Media>,
    pub signatures: Vec<SignatureRow>,
    pub changes: Vec<Change>,
    pub requests: Vec<Request>,
    pub issues: Vec<Issue>,
    pub comments: Vec<Comment>,
    pub locked_keys: Vec<LockedKeyRow>,
}

fn table_of<R: Record>(rows: &[R]) -> Table {
    Table {
        name: R::TABLE.to_string(),
        columns: R::COLUMNS.iter().map(|c| c.to_string()).collect(),
        rows: rows.iter().map(Record::to_row).collect(),
    }
}

fn rows_of<R: Record>(tables: &mut BTreeMap<String, Table>) -> Result<Vec<R>, Error> {
    let Some(t) = tables.remove(R::TABLE) else {
        return Ok(Vec::new());
    };
    if t.columns
        .iter()
        .map(String::as_str)
        .ne(R::COLUMNS.iter().copied())
    {
        return Err(Error::new(
            ErrorKind::Malformed,
            format!(
                "the table {} has the columns {:?}, not {:?}",
                R::TABLE,
                t.columns,
                R::COLUMNS
            ),
        ));
    }
    t.rows.into_iter().map(R::from_row).collect()
}

impl File {
    /// The tables a file is made of, in the schema's order, `meta` first.
    pub fn to_tables(&self) -> Vec<Table> {
        vec![
            Table {
                name: "meta".into(),
                columns: vec!["key".into(), "value".into()],
                rows: self
                    .meta
                    .iter()
                    .map(|(k, v)| vec![Cell::Text(k.clone()), Cell::Text(v.clone())])
                    .collect(),
            },
            table_of(&self.people),
            table_of(&self.keys),
            table_of(&self.assignments),
            table_of(&self.blocks),
            table_of(&self.ports),
            table_of(&self.wires),
            table_of(&self.mounts),
            table_of(&self.closures),
            table_of(&self.loops),
            table_of(&self.cases),
            table_of(&self.texts),
            table_of(&self.tables),
            table_of(&self.media),
            table_of(&self.signatures),
            table_of(&self.changes),
            table_of(&self.requests),
            table_of(&self.issues),
            table_of(&self.comments),
            table_of(&self.locked_keys),
        ]
    }

    /// A file from its tables, refused if a table is not one the schema has,
    /// a column is not where the schema puts it, a cell is not the kind its
    /// column holds, or a constrained column holds a value it may not.
    pub fn from_tables(tables: Vec<Table>) -> Result<File, Error> {
        let mut by_name: BTreeMap<String, Table> = BTreeMap::new();
        for t in tables {
            if by_name.contains_key(&t.name) {
                return Err(Error::new(
                    ErrorKind::Malformed,
                    format!("the table {} is given twice", t.name),
                ));
            }
            by_name.insert(t.name.clone(), t);
        }
        let mut meta = BTreeMap::new();
        if let Some(t) = by_name.remove("meta") {
            for row in t.rows {
                match <[Cell; 2]>::try_from(row) {
                    Ok([Cell::Text(k), Cell::Text(v)]) => {
                        meta.insert(k, v);
                    }
                    _ => {
                        return Err(Error::new(
                            ErrorKind::Malformed,
                            "a row of meta is not a key and a value",
                        ))
                    }
                }
            }
        }
        let f = File {
            meta,
            people: rows_of(&mut by_name)?,
            keys: rows_of(&mut by_name)?,
            assignments: rows_of(&mut by_name)?,
            blocks: rows_of(&mut by_name)?,
            ports: rows_of(&mut by_name)?,
            wires: rows_of(&mut by_name)?,
            mounts: rows_of(&mut by_name)?,
            closures: rows_of(&mut by_name)?,
            loops: rows_of(&mut by_name)?,
            cases: rows_of(&mut by_name)?,
            texts: rows_of(&mut by_name)?,
            tables: rows_of(&mut by_name)?,
            media: rows_of(&mut by_name)?,
            signatures: rows_of(&mut by_name)?,
            changes: rows_of(&mut by_name)?,
            requests: rows_of(&mut by_name)?,
            issues: rows_of(&mut by_name)?,
            comments: rows_of(&mut by_name)?,
            locked_keys: rows_of(&mut by_name)?,
        };
        if let Some(name) = by_name.keys().next() {
            return Err(Error::new(
                ErrorKind::Malformed,
                format!("the table {name} is not in the design file's schema"),
            ));
        }
        f.check_values()?;
        Ok(f)
    }

    /// Every constrained column holds a value it may (`ALLOWED`).
    pub fn check_values(&self) -> Result<(), Error> {
        for t in self.to_tables() {
            for (table, column, allowed) in ALLOWED {
                if t.name != *table {
                    continue;
                }
                let i = t.columns.iter().position(|c| c == column).ok_or_else(|| {
                    Error::new(
                        ErrorKind::Malformed,
                        format!("{table} has no column {column}"),
                    )
                })?;
                for row in &t.rows {
                    if let Cell::Text(v) = &row[i] {
                        if !allowed.contains(&v.as_str()) {
                            return Err(Error::new(
                                ErrorKind::Malformed,
                                format!(
                                    "{table}.{column} is {v:?}, which is not one of {}",
                                    allowed.join(", ")
                                ),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
