//! Two files compared, block by block.
//!
//! docs/PLAN_1_0.md, phase C: "Comparison: any two revisions, releases or
//! designs, node by node." Every file is the one schema (`schema.sql`), so one
//! comparison serves them all: two revisions of a node's file, two releases of
//! a group, two released designs, or today's design against a released one.
//! A file from before 1.0 is upgraded first, as the application opens it.
//!
//! What is compared is the content: every row of every table that says what
//! the design is, each matched across the two files by its key, and each
//! given to the block it belongs to — a port, a wire into it, a case, a text,
//! a table, a closure, a loop. A block is matched by its `uid`, which never
//! changes, so a block renamed is one block with a new `id`, not one removed
//! and another added. What belongs to no block (the file's meta, its people,
//! the group's own texts and tables) is the file's.
//!
//! The record of how the file came to be — its signatures, changes,
//! requests, issues and comments — is history, not content. It is counted,
//! not compared row by row: what a later file holds that the earlier does not.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Cell, File, Table};

/// How each table of the content is compared: the columns that are its key,
/// and the column that names the block a row belongs to (`None` for a table
/// that is the file's alone).
const CONTENT: &[(&str, &[&str], Option<&str>)] = &[
    ("meta", &["key"], None),
    ("person", &["name"], None),
    ("person_key", &["person", "public_key"], None),
    ("assignment", &["block_uid", "person"], Some("block_uid")),
    ("block", &["uid"], Some("uid")),
    (
        "port",
        &["block_uid", "direction", "name"],
        Some("block_uid"),
    ),
    ("wire", &["to_block", "to_port"], Some("to_block")),
    ("mount", &["block_uid"], Some("block_uid")),
    ("closure", &["uid"], Some("block_uid")),
    ("loop", &["uid"], Some("block_uid")),
    ("test_case", &["block_uid", "name"], Some("block_uid")),
    ("text", &["scope", "kind"], Some("scope")),
    ("tbl", &["scope", "path"], Some("scope")),
    ("media", &["scope", "path"], Some("scope")),
];

/// The tables of the content, compared row by row.
pub const CONTENT_TABLES: [&str; CONTENT.len()] = {
    let mut out = [""; CONTENT.len()];
    let mut i = 0;
    while i < CONTENT.len() {
        out[i] = CONTENT[i].0;
        i += 1;
    }
    out
};

/// The history: counted, not compared row by row.
const HISTORY: &[&str] = &[
    "signature",
    "change",
    "request",
    "issue",
    "comment",
    "locked_key",
];

/// One difference in one row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Difference {
    /// A row the second file has and the first does not.
    Added { table: String, key: String },
    /// A row the first file has and the second does not.
    Removed { table: String, key: String },
    /// A field of a row both have, before and after.
    Changed {
        table: String,
        key: String,
        column: String,
        before: String,
        after: String,
    },
}

impl Difference {
    /// The difference in words, as a person reads it.
    pub fn says(&self) -> String {
        match self {
            Difference::Added { table, key } => format!("{}: added", place(table, key)),
            Difference::Removed { table, key } => format!("{}: removed", place(table, key)),
            Difference::Changed {
                table,
                key,
                column,
                before,
                after,
            } if before.contains('\n') || after.contains('\n') => {
                let (gone, came) = lines_changed(before, after);
                let first = came
                    .first()
                    .or(gone.first())
                    .map_or(String::new(), |l| format!(", first {}", short(l)));
                format!(
                    "{}: {column}, {} line(s) added and {} removed{first}",
                    place(table, key),
                    came.len(),
                    gone.len()
                )
            }
            Difference::Changed {
                table,
                key,
                column,
                before,
                after,
            } => format!(
                "{}: {column} {} → {}",
                place(table, key),
                short(before),
                short(after)
            ),
        }
    }
}

/// What became of one block between the two files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Added,
    Removed,
    Changed,
}

impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Status::Added => "added",
            Status::Removed => "removed",
            Status::Changed => "changed",
        }
    }
}

/// One block that differs: its uid, its id in each file it is in, and every
/// difference in its rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub uid: String,
    /// Its `id` in the first file, if it is there.
    pub before: Option<String>,
    /// Its `id` in the second file, if it is there.
    pub after: Option<String>,
    pub status: Status,
    pub differences: Vec<Difference>,
}

impl Block {
    /// The name a person knows it by: its id now, or its id before if it is
    /// gone, or its uid if neither file gives it one.
    pub fn id(&self) -> &str {
        self.after
            .as_deref()
            .or(self.before.as_deref())
            .unwrap_or(&self.uid)
    }
}

/// Two files compared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    /// What differs in what is the file's own: its meta, its people, its texts
    /// and tables.
    pub file: Vec<Difference>,
    /// Every block that differs, in the order of its id.
    pub blocks: Vec<Block>,
    /// How many blocks both files hold the same.
    pub same: usize,
    /// For each table of the history: rows only the first holds, rows only
    /// the second holds.
    pub history: Vec<(String, usize, usize)>,
}

impl Comparison {
    /// Nothing in the content differs.
    pub fn is_same(&self) -> bool {
        self.file.is_empty() && self.blocks.is_empty()
    }

    /// The comparison in a line: how many blocks changed, were added, were
    /// removed and are the same.
    pub fn summary(&self) -> String {
        let n = |s: Status| self.blocks.iter().filter(|b| b.status == s).count();
        format!(
            "{} block(s) changed, {} added, {} removed, {} the same; {} difference(s) in the file itself",
            n(Status::Changed),
            n(Status::Added),
            n(Status::Removed),
            self.same,
            self.file.len()
        )
    }
}

/// The lines of a text the other does not keep, each way: the lines only the
/// first has, and the lines only the second has, by their longest common
/// subsequence, so a line moved by an insertion above it is not counted.
pub fn lines_changed<'a>(before: &'a str, after: &'a str) -> (Vec<&'a str>, Vec<&'a str>) {
    let (a, b): (Vec<&str>, Vec<&str>) = (before.lines().collect(), after.lines().collect());
    let (n, m) = (a.len(), b.len());
    let mut l = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            l[i][j] = if a[i] == b[j] {
                l[i + 1][j + 1] + 1
            } else {
                l[i + 1][j].max(l[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let (mut gone, mut came) = (Vec::new(), Vec::new());
    while i < n && j < m {
        if a[i] == b[j] {
            i += 1;
            j += 1;
        } else if l[i + 1][j] >= l[i][j + 1] {
            gone.push(a[i]);
            i += 1;
        } else {
            came.push(b[j]);
            j += 1;
        }
    }
    gone.extend(&a[i..]);
    came.extend(&b[j..]);
    (gone, came)
}

fn place(table: &str, key: &str) -> String {
    if key.is_empty() {
        table.to_string()
    } else {
        format!("{table} {key}")
    }
}

fn short(s: &str) -> String {
    let one: String = s.replace('\n', "⏎");
    if one.chars().count() > 60 {
        format!("“{}…”", one.chars().take(60).collect::<String>())
    } else {
        format!("“{one}”")
    }
}

fn cell(c: &Cell) -> String {
    match c {
        Cell::Null => String::new(),
        Cell::Int(i) => i.to_string(),
        Cell::Text(t) => t.clone(),
        Cell::Blob(b) => format!(
            "{} bytes, SHA-256 {}",
            b.len(),
            &crate::keys::hex(&crate::keys::sha256(b))[..16]
        ),
    }
}

/// A table's rows by their key: each row as column → value.
type Rows = BTreeMap<Vec<String>, BTreeMap<String, String>>;

fn rows(t: Option<&Table>, key: &[&str]) -> Rows {
    let mut out = Rows::new();
    let Some(t) = t else { return out };
    for r in &t.rows {
        let fields: BTreeMap<String, String> = t
            .columns
            .iter()
            .zip(r)
            .map(|(c, v)| (c.clone(), cell(v)))
            .collect();
        let k = key
            .iter()
            .map(|k| fields.get(*k).cloned().unwrap_or_default())
            .collect();
        out.insert(k, fields);
    }
    out
}

/// The key a person reads: its parts, but for the one that names the block.
fn key_says(key: &[&str], k: &[String], block: Option<&str>) -> String {
    key.iter()
        .zip(k)
        .filter(|(c, _)| Some(**c) != block)
        .map(|(_, v)| v.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Compare two files: what the second holds that differs from the first.
pub fn compare(a: &File, b: &File) -> Comparison {
    let (ta, tb) = (a.to_tables(), b.to_tables());
    let table = |ts: &[Table], name: &str| ts.iter().find(|t| t.name == name).cloned();
    let ids = |f: &File| -> BTreeMap<String, String> {
        f.blocks
            .iter()
            .map(|b| (b.uid.clone(), b.id.clone()))
            .collect()
    };
    let (ids_a, ids_b) = (ids(a), ids(b));
    let mut file = Vec::new();
    let mut by_block: BTreeMap<String, Vec<Difference>> = BTreeMap::new();
    for (name, key, block) in CONTENT {
        let (ra, rb) = (
            rows(table(&ta, name).as_ref(), key),
            rows(table(&tb, name).as_ref(), key),
        );
        let keys: BTreeSet<&Vec<String>> = ra.keys().chain(rb.keys()).collect();
        for k in keys {
            let (x, y) = (ra.get(k), rb.get(k));
            let owner = block.and_then(|c| {
                x.or(y)
                    .and_then(|r| r.get(c))
                    .filter(|s| ids_a.contains_key(*s) || ids_b.contains_key(*s))
                    .cloned()
            });
            let says = key_says(key, k, *block);
            let mut found = Vec::new();
            match (x, y) {
                (None, Some(_)) => found.push(Difference::Added {
                    table: name.to_string(),
                    key: says,
                }),
                (Some(_), None) => found.push(Difference::Removed {
                    table: name.to_string(),
                    key: says,
                }),
                (Some(x), Some(y)) => {
                    for (column, before) in x {
                        let after = y.get(column).cloned().unwrap_or_default();
                        if *before != after {
                            found.push(Difference::Changed {
                                table: name.to_string(),
                                key: says.clone(),
                                column: column.clone(),
                                before: before.clone(),
                                after,
                            });
                        }
                    }
                }
                (None, None) => {}
            }
            match owner {
                Some(uid) => by_block.entry(uid).or_default().extend(found),
                None => file.extend(found),
            }
        }
    }
    let all: BTreeSet<&String> = ids_a.keys().chain(ids_b.keys()).collect();
    let mut blocks = Vec::new();
    let mut same = 0;
    for uid in all {
        let (before, after) = (ids_a.get(uid).cloned(), ids_b.get(uid).cloned());
        let differences = by_block.remove(uid).unwrap_or_default();
        let status = match (&before, &after) {
            (None, Some(_)) => Status::Added,
            (Some(_), None) => Status::Removed,
            _ if differences.is_empty() => {
                same += 1;
                continue;
            }
            _ => Status::Changed,
        };
        blocks.push(Block {
            uid: uid.clone(),
            before,
            after,
            status,
            differences,
        });
    }
    blocks.sort_by(|x, y| x.id().cmp(y.id()));
    let history = HISTORY
        .iter()
        .map(|name| {
            let all = |ts: &[Table]| -> BTreeMap<Vec<String>, usize> {
                let mut m = BTreeMap::new();
                if let Some(t) = ts.iter().find(|t| t.name == *name) {
                    for r in &t.rows {
                        *m.entry(r.iter().map(cell).collect()).or_default() += 1;
                    }
                }
                m
            };
            let (ha, hb) = (all(&ta), all(&tb));
            let only = |x: &BTreeMap<Vec<String>, usize>, y: &BTreeMap<Vec<String>, usize>| {
                x.iter()
                    .map(|(k, n)| n.saturating_sub(*y.get(k).unwrap_or(&0)))
                    .sum::<usize>()
            };
            (name.to_string(), only(&ha, &hb), only(&hb, &ha))
        })
        .collect();
    Comparison {
        file,
        blocks,
        same,
        history,
    }
}
