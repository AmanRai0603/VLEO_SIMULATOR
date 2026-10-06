//! The file a group keeps today: format 1, read as it is.
//!
//! Format 1 is `groups/schema.sql`: a structure, a node file or a release, as
//! the group application writes them. This module reads one as its rows, and
//! gives the folder it holds and the fingerprint over that folder, exactly as
//! the page gives them (`web/js/gdb.js`, `folderFromDb`; `web/js/gseal.js`,
//! `fingerprint`). The fingerprint is what every sign-off before 1.0 was given
//! against and what a release was sealed with, so it is written once here, and
//! the upgrade (`crate::upgrade`) can show that what it upgraded is what the
//! people signed.

use std::collections::BTreeMap;

use crate::csv;
use crate::error::{Error, ErrorKind};
use crate::keys::{hex, sha256};
use crate::model::{record, Cell, FromCell, IntoCell, Record, Table};

/// Format 1's number: `user_version` in a group file before 1.0.
pub const FORMAT: i64 = 1;

record!(
    /// A member of the group, as format 1 named them.
    Member = "member" { name: String, role: String }
);
record!(
    /// A node: its contract and where it sits.
    Node = "node" {
        uid: String, id: String, question: String, kind: String, output: String,
        unit: String, lower: String, upper: String, value: String, stage: String,
        author: String, ord: i64, archived: i64, contract_version: i64, revision: i64,
    }
);
record!(
    /// An input of a node, and where it comes from.
    Input = "input" {
        node_uid: String, ord: i64, name: String, symbol: String, source: String,
        unit: String, dflt: String, min: String, max: String, says: String,
    }
);
record!(
    /// A text.
    Doc = "doc" { scope: String, kind: String, body: String }
);
record!(
    /// A table, as CSV.
    Tbl = "tbl" { scope: String, path: String, csv: String }
);
record!(
    /// A file that is not a table.
    Media = "media" {
        scope: String, path: String, r#type: String, sha256: String, bytes: Vec<u8>,
    }
);
record!(
    /// A sign-off: a name, against a fingerprint.
    Review = "review" {
        name: String, scope: String, version: String, fingerprint: String,
        date: String, verdict: String, note: String,
    }
);
record!(
    /// A comment, or an issue raised in the group application.
    Comment = "comment" {
        scope: String, section: String, author: String, at: String, body: String, resolved: i64,
    }
);
record!(
    /// A node engineer's request to change their contract.
    Request = "request" {
        node_uid: String, author: String, at: String, body: String, status: String, answer: String,
    }
);
record!(
    /// A change, as it was made.
    Change = "change" {
        at: String, who: String, scope: String, what: String,
        before: Option<String>, after: Option<String>,
    }
);

/// A whole format-1 file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Old {
    pub meta: BTreeMap<String, String>,
    pub members: Vec<Member>,
    pub nodes: Vec<Node>,
    pub inputs: Vec<Input>,
    pub docs: Vec<Doc>,
    pub tbls: Vec<Tbl>,
    pub media: Vec<Media>,
    pub reviews: Vec<Review>,
    pub comments: Vec<Comment>,
    pub requests: Vec<Request>,
    pub changes: Vec<Change>,
}

/// A folder, as the page holds one in memory: every path, and its bytes, in
/// the order the page lists them.
pub type Folder = BTreeMap<String, Vec<u8>>;

fn table_of<R: Record>(rows: &[R]) -> Table {
    Table {
        name: R::TABLE.to_string(),
        columns: R::COLUMNS
            .iter()
            .map(|c| c.trim_start_matches("r#").to_string())
            .collect(),
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
        .ne(R::COLUMNS.iter().map(|c| c.trim_start_matches("r#")))
    {
        return Err(Error::new(
            ErrorKind::Malformed,
            format!(
                "the format 1 table {} has the columns {:?}, not the ones format 1 gives it",
                R::TABLE,
                t.columns
            ),
        ));
    }
    t.rows.into_iter().map(R::from_row).collect()
}

/// The two scopes format 1 has: `group`, or a node's uid.
pub const GROUP: &str = "group";

const GROUP_DOCS: &[(&str, &str)] = &[
    ("explanation.md", "explanation"),
    ("theory.md", "theory"),
    ("flow.txt", "flow"),
];
const NODE_DOCS: &[(&str, &str)] = &[
    ("explanation.md", "explanation"),
    ("theory.md", "theory"),
    ("pseudocode.txt", "pseudocode"),
    ("results/how-run.md", "how_run"),
];

impl Old {
    /// A format-1 file from its tables, refused if a table is not one format 1
    /// has or its columns are not the ones format 1 gives it.
    pub fn from_tables(tables: Vec<Table>) -> Result<Old, Error> {
        let mut by_name: BTreeMap<String, Table> =
            tables.into_iter().map(|t| (t.name.clone(), t)).collect();
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
        let old = Old {
            meta,
            members: rows_of(&mut by_name)?,
            nodes: rows_of(&mut by_name)?,
            inputs: rows_of(&mut by_name)?,
            docs: rows_of(&mut by_name)?,
            tbls: rows_of(&mut by_name)?,
            media: rows_of(&mut by_name)?,
            reviews: rows_of(&mut by_name)?,
            comments: rows_of(&mut by_name)?,
            requests: rows_of(&mut by_name)?,
            changes: rows_of(&mut by_name)?,
        };
        if let Some(name) = by_name.keys().next() {
            return Err(Error::new(
                ErrorKind::Malformed,
                format!("the table {name} is not one format 1 has"),
            ));
        }
        Ok(old)
    }

    /// The tables the file is made of, `meta` first.
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
            table_of(&self.members),
            table_of(&self.nodes),
            table_of(&self.inputs),
            table_of(&self.docs),
            table_of(&self.tbls),
            table_of(&self.media),
            table_of(&self.reviews),
            table_of(&self.comments),
            table_of(&self.requests),
            table_of(&self.changes),
        ]
    }

    fn meta(&self, key: &str) -> String {
        self.meta.get(key).cloned().unwrap_or_default()
    }

    /// The folder this file holds, byte for byte the one the page shows and
    /// signs (`folderFromDb`): the group's lists written as CSV from their
    /// columns, then its texts, tables and media where the pattern puts them.
    pub fn folder(&self) -> Folder {
        let mut files = Folder::new();
        let mut put = |p: String, body: Vec<u8>| {
            files.insert(p, body);
        };
        let mut live: Vec<&Node> = self.nodes.iter().filter(|n| n.archived == 0).collect();
        live.sort_by(|a, b| (a.ord, &a.id).cmp(&(b.ord, &b.id)));
        let id_of: BTreeMap<&str, &str> = self
            .nodes
            .iter()
            .map(|n| (n.uid.as_str(), n.id.as_str()))
            .collect();
        let dir_of = |scope: &str| -> String {
            if scope == GROUP {
                String::new()
            } else {
                format!("nodes/{}/", id_of.get(scope).copied().unwrap_or(scope))
            }
        };
        let live_uids: Vec<&str> = live.iter().map(|n| n.uid.as_str()).collect();
        let in_scope = |scope: &str| scope == GROUP || live_uids.contains(&scope);
        let authors = |n: &Node| -> Vec<String> { split_authors(&n.author) };

        put(
            "group.csv".into(),
            csv::write(
                &["id", "name", "owner", "version", "summary"],
                &[["group_id", "group_name", "owner", "version", "summary"]
                    .iter()
                    .map(|k| self.meta(k))
                    .collect()],
            )
            .into_bytes(),
        );
        let mut members: Vec<&Member> = self.members.iter().collect();
        members.sort_by(|a, b| a.name.cmp(&b.name));
        let member_rows: Vec<Vec<String>> = members
            .iter()
            .map(|m| {
                let mine: Vec<&str> = live
                    .iter()
                    .filter(|n| authors(n).contains(&m.name))
                    .map(|n| n.id.as_str())
                    .collect();
                let nodes = if m.role == "owner" && mine.is_empty() {
                    "*".to_string()
                } else {
                    mine.join(" ")
                };
                vec![m.name.clone(), m.role.clone(), nodes]
            })
            .collect();
        put(
            "members.csv".into(),
            csv::write(&["name", "role", "nodes"], &member_rows).into_bytes(),
        );
        put(
            "nodes.csv".into(),
            csv::write(
                &[
                    "id", "question", "kind", "output", "unit", "lower", "upper", "value",
                ],
                &live
                    .iter()
                    .map(|n| {
                        vec![
                            n.id.clone(),
                            n.question.clone(),
                            n.kind.clone(),
                            n.output.clone(),
                            n.unit.clone(),
                            n.lower.clone(),
                            n.upper.clone(),
                            n.value.clone(),
                        ]
                    })
                    .collect::<Vec<_>>(),
            )
            .into_bytes(),
        );
        for n in &live {
            let mut ins: Vec<&Input> = self.inputs.iter().filter(|i| i.node_uid == n.uid).collect();
            ins.sort_by_key(|i| i.ord);
            if ins.is_empty() && n.kind != "computed" {
                continue;
            }
            // `symbol` is optional in the pattern: written only when some input has one.
            let sym = ins.iter().any(|i| !i.symbol.is_empty());
            let mut head = vec!["name"];
            if sym {
                head.push("symbol");
            }
            head.extend(["from", "unit", "default", "min", "max", "says"]);
            let rows: Vec<Vec<String>> = ins
                .iter()
                .map(|i| {
                    let mut r = vec![i.name.clone()];
                    if sym {
                        r.push(i.symbol.clone());
                    }
                    let from = id_of.get(i.source.as_str()).copied().unwrap_or(&i.source);
                    r.extend([
                        from.to_string(),
                        i.unit.clone(),
                        i.dflt.clone(),
                        i.min.clone(),
                        i.max.clone(),
                        i.says.clone(),
                    ]);
                    r
                })
                .collect();
            put(
                format!("nodes/{}/inputs.csv", n.id),
                csv::write(&head, &rows).into_bytes(),
            );
        }
        if !self.reviews.is_empty() {
            let rows: Vec<Vec<String>> = self
                .reviews
                .iter()
                .map(|r| {
                    let scope = if r.scope == GROUP {
                        GROUP
                    } else {
                        id_of.get(r.scope.as_str()).copied().unwrap_or(&r.scope)
                    };
                    vec![
                        r.name.clone(),
                        scope.to_string(),
                        r.version.clone(),
                        r.fingerprint.clone(),
                        r.date.clone(),
                        r.verdict.clone(),
                        r.note.clone(),
                    ]
                })
                .collect();
            put(
                "reviews.csv".into(),
                csv::write(
                    &[
                        "name",
                        "scope",
                        "version",
                        "fingerprint",
                        "date",
                        "verdict",
                        "note",
                    ],
                    &rows,
                )
                .into_bytes(),
            );
        }
        for d in &self.docs {
            if !in_scope(&d.scope) {
                continue;
            }
            let table = if d.scope == GROUP {
                GROUP_DOCS
            } else {
                NODE_DOCS
            };
            if let Some((rel, _)) = table.iter().find(|(_, k)| *k == d.kind) {
                put(
                    format!("{}{rel}", dir_of(&d.scope)),
                    d.body.clone().into_bytes(),
                );
            }
        }
        for t in &self.tbls {
            if in_scope(&t.scope) {
                put(
                    format!("{}{}", dir_of(&t.scope), t.path),
                    t.csv.clone().into_bytes(),
                );
            }
        }
        for m in &self.media {
            if in_scope(&m.scope) {
                put(format!("{}{}", dir_of(&m.scope), m.path), m.bytes.clone());
            }
        }
        files
    }
}

/// The node engineers a format-1 node names, as the page splits them: at each
/// comma, the spaces after it dropped.
pub fn split_authors(author: &str) -> Vec<String> {
    author
        .split(',')
        .enumerate()
        .map(|(i, a)| {
            if i == 0 {
                a.to_string()
            } else {
                a.trim_start().to_string()
            }
        })
        .collect()
}

/// Files that are the folder's record of itself, never part of what is signed.
fn unsigned(path: &str) -> bool {
    path == "reviews.csv" || path.starts_with("packages/") || path.starts_with("issues/")
}

/// The fingerprint over a scope of a folder: the whole group (`None`) or one
/// node's folder (its id). A SHA-256 over every file's path and SHA-256, in
/// path order, so the same files give the same fingerprint on every machine
/// and any change at all gives another. As `web/js/gseal.js` gives it.
pub fn fingerprint(folder: &Folder, node: Option<&str>) -> String {
    let prefix = node.map_or(String::new(), |id| format!("nodes/{id}/"));
    let lines: Vec<String> = folder
        .iter()
        .filter(|(p, _)| p.starts_with(&prefix) && !unsigned(p))
        .map(|(p, bytes)| format!("{p}\u{0}{}", hex(&sha256(bytes))))
        .collect();
    hex(&sha256(lines.join("\n").as_bytes()))
}
