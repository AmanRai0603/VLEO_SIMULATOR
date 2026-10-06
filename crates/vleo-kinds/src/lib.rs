//! Every kind of file the tools write, in one table.
//!
//! docs/PLAN_1_0.md, phase C: "Every reader follows the schema: `vleo-py`,
//! `vleo-cli`, `vleo-server` and `vleo-ffi`, and the cases and results saved
//! under `~/.vleo`." Each reader had its own copy of the same few checks — is
//! this a VLEO file, which kind does it say it is, is it from a newer tool —
//! and each said its refusal in its own words. Here they are one rule: a file
//! is taken only as a kind this table lists, in a format this table lists for
//! it, by the reader this table names.
//!
//! The table also keeps two things apart that must never be confused
//! (AGENTS.md: "today's design is never taken for a released one"):
//! `design.vleo` is **today's design**, the repository's tree as one file,
//! kind `design`, format 1; a **released design** is a design the system
//! engineer released, kind `released design`, format 2. Until phase D moves
//! the design into the one schema, the first is what the tool runs, and no
//! reader takes either for the other.
//!
//! Python's reader (`crates/vleo-py/python/vleo/files.py`) carries this
//! table as generated text, held to it by a test, and its rule is held to
//! this one, case by case.

/// SQLite's `application_id` in every database the tools write: 'VLEO'.
pub const APPLICATION_ID: i64 = 1_447_838_031;

/// The program that reads a kind of file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reader {
    /// The one library that reads, writes and checks every design file
    /// (`vleo-files`), installed and in the page.
    Files,
    /// The tool's own database files before 1.0 (`vleo-design`): today's
    /// design and saved results, read by the server, the command line and
    /// the tool.
    Design,
    /// The text files under `~/.vleo` (`vleo-modules`): a case's inputs and
    /// a saved result.
    Modules,
}

impl Reader {
    pub fn name(self) -> &'static str {
        match self {
            Reader::Files => "vleo-files",
            Reader::Design => "vleo-design",
            Reader::Modules => "vleo-modules",
        }
    }
}

/// How a kind of file is stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Store {
    /// A SQLite database: `application_id` is [`APPLICATION_ID`],
    /// `user_version` the format, and `meta.file_kind` the kind's name.
    Database { format: u32 },
    /// Text, whose first `#!` line names the kind and its version: `line` is
    /// what the line starts with, `version` what follows it — empty when the
    /// version is the template of inputs the file was written for.
    Text {
        line: &'static str,
        version: &'static str,
    },
}

/// One kind of file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kind {
    /// The name the file gives itself: `meta.file_kind`, or the word on its
    /// `#!` line.
    pub name: &'static str,
    pub store: Store,
    /// The file as people know it.
    pub file: &'static str,
    pub reader: Reader,
    /// The kind and format it becomes when it is opened, if it is upgraded.
    pub becomes: Option<(&'static str, u32)>,
    /// What it is, in a sentence.
    pub says: &'static str,
}

impl Kind {
    /// The format of a database kind; none for text.
    pub fn format(&self) -> Option<u32> {
        match self.store {
            Store::Database { format } => Some(format),
            Store::Text { .. } => None,
        }
    }
}

const fn db(
    name: &'static str,
    format: u32,
    file: &'static str,
    reader: Reader,
    becomes: Option<(&'static str, u32)>,
    says: &'static str,
) -> Kind {
    Kind {
        name,
        store: Store::Database { format },
        file,
        reader,
        becomes,
        says,
    }
}

/// The version a saved result's `#! result` line names.
pub const RESULT_VERSION: &str = "vleo-result/1";
/// The version a saved sweep's `#! sweep` line names.
pub const SWEEP_VERSION: &str = "vleo-sweep/1";
/// The line a case's inputs name the template they were written for on.
pub const CASE_LINE: &str = "#! template";

/// Every kind of file the tools write, and every kind they wrote before that
/// is still read.
pub const KINDS: &[Kind] = &[
    // A group's files before 1.0 (groups/schema.sql), upgraded when opened
    // with a copy kept (vleo-files, upgrade.rs).
    db(
        "structure",
        1,
        "a group's .vleo",
        Reader::Files,
        Some(("group", 2)),
        "a group's structure before 1.0, written by the group application",
    ),
    db(
        "node",
        1,
        "a node's .vleo",
        Reader::Files,
        Some(("node", 2)),
        "a node's file before 1.0, written by the node application",
    ),
    db(
        "release",
        1,
        "a group's sealed .vleo",
        Reader::Files,
        Some(("group release", 2)),
        "a group's sealed release before 1.0",
    ),
    // The tool's own files before 1.0 (crates/vleo-design).
    db(
        "design",
        1,
        "design.vleo",
        Reader::Design,
        None,
        "today's design: the repository's tree as one file, which the tool runs until phase D; never a released design",
    ),
    db(
        "results",
        1,
        "a .vleor",
        Reader::Design,
        None,
        "saved results, many in one file",
    ),
    // The one schema (crates/vleo-files/src/schema.sql), section 10.
    db("node", 2, "a node's .vleo", Reader::Files, None, "one node's file"),
    db("group", 2, "a group's .vleo", Reader::Files, None, "a group's file"),
    db(
        "group release",
        2,
        "a group's sealed .vleo",
        Reader::Files,
        None,
        "a group's sealed release",
    ),
    db("preview", 2, "a preview's .vleo", Reader::Files, None, "a preview sent to a group"),
    db(
        "preview answer",
        2,
        "a preview answer's .vleo",
        Reader::Files,
        None,
        "a group's answer to a preview",
    ),
    db("issue", 2, "an issue's .vleo", Reader::Files, None, "an issue raised"),
    db(
        "daily snapshot",
        2,
        "daily/<date>.vleo",
        Reader::Files,
        None,
        "how the whole design stood on one day",
    ),
    db(
        "released design",
        2,
        "design/<version>.vleo",
        Reader::Files,
        None,
        "a design the system engineer released; never today's design",
    ),
    db("case", 2, "a case's .vleo", Reader::Files, None, "a case: the inputs a run sets"),
    db("results", 2, "a results .vleo", Reader::Files, None, "saved results"),
    db("key", 2, "a key's .vleo", Reader::Files, None, "a person's key, locked by their passphrase"),
    // The text files under ~/.vleo (crates/vleo-modules).
    Kind {
        name: "case inputs",
        store: Store::Text {
            line: CASE_LINE,
            version: "",
        },
        file: "~/.vleo/case/inputs.csv",
        reader: Reader::Modules,
        becomes: None,
        says: "a case's inputs, written for one template of inputs and carried over to a newer one when opened, with a copy kept",
    },
    Kind {
        name: "result",
        store: Store::Text {
            line: "#! result",
            version: RESULT_VERSION,
        },
        file: "~/.vleo/results/<id>/result.csv",
        reader: Reader::Modules,
        becomes: None,
        says: "one saved run",
    },
    Kind {
        name: "sweep",
        store: Store::Text {
            line: "#! sweep",
            version: SWEEP_VERSION,
        },
        file: "~/.vleo/results/<id>/sweep.csv",
        reader: Reader::Modules,
        becomes: None,
        says: "one saved sweep",
    },
];

/// The text kind whose `#!` line starts with `line`.
pub fn text(line: &str) -> &'static Kind {
    KINDS
        .iter()
        .find(|k| matches!(k.store, Store::Text { line: l, .. } if l == line))
        .expect("a text kind of the table")
}

/// What a reader asks for: the reader it is, the kinds it takes by name, and
/// what to call them when a file is not one.
#[derive(Clone, Copy, Debug)]
pub struct Want<'a> {
    pub reader: Reader,
    pub names: &'a [&'a str],
    /// What the reader takes, as a refusal says it: "a results file".
    pub called: &'a str,
}

/// Why a database file is not taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Not a database the VLEO tools wrote.
    NotOurs,
    /// A VLEO file that does not say what kind it is.
    NoKind { format: i64 },
    /// It says a kind and format no VLEO tool writes.
    Unknown { name: String, format: i64 },
    /// A kind this reader takes, in a format newer than it reads.
    Newer {
        name: String,
        format: i64,
        newest: u32,
    },
    /// Another kind of file.
    OtherKind {
        found: &'static Kind,
        called: String,
    },
}

impl Refusal {
    /// The refusal in words, for the file at `place`.
    pub fn says(&self, place: &str) -> String {
        match self {
            Refusal::NotOurs => format!("{place}: not a database the VLEO tools wrote"),
            Refusal::NoKind { format } => {
                format!("{place}: a VLEO file in format {format} that does not say what kind it is")
            }
            Refusal::Unknown { name, format } => format!(
                "{place}: says it is a {name} file in format {format}, which no VLEO tool writes"
            ),
            Refusal::Newer {
                name,
                format,
                newest,
            } => format!(
                "{place}: a {name} file in format {format}, newer than this tool reads ({newest}): use the newer tool"
            ),
            Refusal::OtherKind { found, called } => format!(
                "{place}: a {} file, not {called} — {}",
                found.name, found.says
            ),
        }
    }
}

/// The kind a database file is, taken by the reader `want` names, or why it
/// is not taken. `application_id` and `format` are the file's `PRAGMA`s,
/// `file_kind` its `meta.file_kind`. In order:
///
/// 1. a database the tools did not write is refused;
/// 2. so is one that does not say what kind it is, and one whose kind no
///    tool writes — as newer, when its format is beyond any this reader
///    reads, since a newer tool may say its kinds anew;
/// 3. a kind this reader does not take, nor becomes one it takes, is another
///    kind of file, said by its name and what it is;
/// 4. a kind it takes, in a format this table lists for this reader, is taken;
/// 5. in any other format it is newer than this reader reads, or a format no
///    tool writes.
pub fn identify(
    application_id: i64,
    format: i64,
    file_kind: Option<&str>,
    want: Want,
) -> Result<&'static Kind, Refusal> {
    if application_id != APPLICATION_ID {
        return Err(Refusal::NotOurs);
    }
    // A newer tool may name its kinds anew, or say them elsewhere: a file in
    // a format beyond any this reader reads is newer, whatever it says.
    let newest_here = KINDS
        .iter()
        .filter(|k| k.reader == want.reader)
        .filter_map(Kind::format)
        .max()
        .unwrap_or(0);
    let newer = |name: &str| Refusal::Newer {
        name: name.to_string(),
        format,
        newest: newest_here,
    };
    let Some(name) = file_kind else {
        return Err(if format > i64::from(newest_here) {
            newer("VLEO")
        } else {
            Refusal::NoKind { format }
        });
    };
    let same_name: Vec<&'static Kind> = KINDS.iter().filter(|k| k.name == name).collect();
    if same_name.is_empty() {
        return Err(if format > i64::from(newest_here) {
            newer(name)
        } else {
            Refusal::Unknown {
                name: name.to_string(),
                format,
            }
        });
    }
    let wanted = |n: &str| want.names.contains(&n);
    let this_format = same_name
        .iter()
        .find(|k| k.format().map(i64::from) == Some(format));
    let becomes_wanted = this_format.is_some_and(|k| k.becomes.is_some_and(|(b, _)| wanted(b)));
    if !wanted(name) && !becomes_wanted {
        return Err(Refusal::OtherKind {
            found: this_format.unwrap_or(&same_name[0]),
            called: want.called.to_string(),
        });
    }
    if let Some(k) = this_format.filter(|k| k.reader == want.reader) {
        return Ok(k);
    }
    match same_name
        .iter()
        .filter(|k| k.reader == want.reader)
        .filter_map(|k| k.format())
        .max()
    {
        Some(newest) if format > i64::from(newest) => Err(Refusal::Newer {
            name: name.to_string(),
            format,
            newest,
        }),
        _ => Err(Refusal::Unknown {
            name: name.to_string(),
            format,
        }),
    }
}

/// The table, as the Python reader carries it: the block between the
/// `# --- kinds` markers in `crates/vleo-py/python/vleo/files.py`.
pub fn python_table() -> String {
    let q = |s: &str| format!("{s:?}");
    let mut o = String::new();
    o.push_str("# --- kinds: GENERATED from crates/vleo-kinds (python_table); do not edit ---\n");
    o.push_str(&format!(
        "APP_ID = {APPLICATION_ID}  # 'VLEO', in every database the tools write\n"
    ));
    o.push_str("# (name, format, reader, becomes, says): every database kind the tools write.\n");
    o.push_str("KINDS = [\n");
    for k in KINDS {
        let Some(format) = k.format() else { continue };
        let becomes = k
            .becomes
            .map_or("None".to_string(), |(b, f)| format!("({}, {f})", q(b)));
        o.push_str(&format!(
            "    ({}, {format}, {}, {becomes}, {}),\n",
            q(k.name),
            q(k.reader.name()),
            q(k.says)
        ));
    }
    o.push_str("]\n");
    o.push_str("# --- end kinds ---\n");
    o
}
