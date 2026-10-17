//! A node written by plain edits, in the person's working copy
//! (docs/briefs/F3.md, F3-g2a; W3).
//!
//! A page cannot read a node file (it is SQLite, and nothing is compiled for
//! the browser for the application), so the Node workspace sends what the
//! person typed and the library writes it. Four edits, the template form's
//! own fields:
//!
//! - **the question** (`question`): the block's own;
//! - **the explanation's three texts** (`explain_simply`, `explain_breaks`,
//!   `explain_wrong`): in a release-shaped node file, the *Said simply*,
//!   *Where it breaks* and *Common misreading* sections of its one
//!   `explanation` text. One section is replaced alone; every other section,
//!   and everything before the first, is kept byte for byte;
//! - **the method** (`method_text`): its `pseudocode` text;
//! - **the cases**: its `results/isolation.csv`, whole.
//!
//! Each is refused, with nothing written, by name:
//!
//! - **a node of another shape**: one of `design/`'s shape (a `sheet` text)
//!   or one that holds neither shape is written here once one node shape is
//!   picked (F3-g0); until then it is read and asked of;
//! - **a case whose expected value is not a reference** (rule 4): every row's
//!   `origin` is one of [`checks::ORIGINS`];
//! - **a method or cases with no declaration, or one that does not hold**
//!   (rule 6): an edit of either restates who made the method and its
//!   numbers, and how an assistant helped, and is held to the declaration
//!   check every node is held to ([`checks::declared`]);
//! - **a decision with no record of which belief broke**: the question and
//!   the method are decisions (`vleo_sheet::derisk::about_of`), the wording
//!   and the cases are not. A decision changed writes a row of the node's own
//!   `versions.csv`, numbered with the revision it is saved as, or is refused
//!   naming what the record is missing (`template::Derisk::missing`);
//! - **an edit that changes nothing**: a revision rises for work;
//! - and everything a save refuses (`save::save`): a second writer, anyone
//!   but the node's writer or their deputy, a change outside the node's own
//!   rows. Ports, wires, behaviour, the sheet's words and the requests are
//!   never edited here: there is no edit for any of them.
//!
//! [`try_edit`] is the dry run, *checked as it is typed*: nothing written,
//! what signing would find (`checks::check_node`) and the method's own report
//! on each case, by the code those checks run.

use crate::checks::{self, Findings};
use crate::csv;
use crate::error::{Error, ErrorKind};
use crate::model::{File, Tbl, Text};
use crate::save::{self, Saved};
use std::path::Path;
use vleo_sheet::template::Derisk;

fn refused(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, message)
}

/// Which shape a node file is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A group's release from before 1.0: a `pseudocode` text, a
    /// `declaration.csv`, a `results/isolation.csv`.
    Release,
    /// `design/`'s, converted from the sheets: a `sheet` text.
    Design,
    /// Neither, or both.
    Neither,
}

/// The shape of the node `uid` in `f`.
pub fn shape(f: &File, uid: &str) -> Shape {
    let text = |k: &str| f.texts.iter().any(|t| t.scope == uid && t.kind == k);
    let table = |p: &str| f.tables.iter().any(|t| t.scope == uid && t.path == p);
    let release = text("pseudocode") || table("declaration.csv") || table("results/isolation.csv");
    match (release, text("sheet")) {
        (true, false) => Shape::Release,
        (false, true) => Shape::Design,
        _ => Shape::Neither,
    }
}

/// One of the explanation's three texts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Simply,
    Breaks,
    Wrong,
}

impl Part {
    /// Its section in a release's `explanation`.
    pub fn heading(self) -> &'static str {
        match self {
            Part::Simply => "Said simply",
            Part::Breaks => "Where it breaks",
            Part::Wrong => "Common misreading",
        }
    }
    /// Its field in the template form.
    pub fn field(self) -> &'static str {
        match self {
            Part::Simply => "explain_simply",
            Part::Breaks => "explain_breaks",
            Part::Wrong => "explain_wrong",
        }
    }
}

/// What the node `uid` in `f` says in `part`: its section of the
/// explanation, as written, or nothing.
pub fn section(f: &File, uid: &str, part: Part) -> String {
    f.texts
        .iter()
        .find(|t| t.scope == uid && t.kind == "explanation")
        .and_then(|t| {
            crate::folder::sections(&t.body)
                .into_iter()
                .find(|s| s.0.eq_ignore_ascii_case(part.heading()))
        })
        .map_or(String::new(), |s| s.1.trim_matches('\n').to_string())
}

/// The node's declaration as its `declaration.csv` holds it, or nothing.
pub fn declared_of(f: &File, uid: &str) -> Declared {
    let Some(t) = f
        .tables
        .iter()
        .find(|t| t.scope == uid && t.path == "declaration.csv")
    else {
        return Declared::default();
    };
    let (head, rows) = csv::parse(&t.csv);
    let get = |k: &str| {
        head.iter()
            .position(|h| csv::name_of(h) == k)
            .and_then(|i| rows.first().and_then(|r| r.get(i)))
            .map_or(String::new(), |v| v.trim().to_string())
    };
    Declared {
        author: get("author"),
        ai: get("ai"),
        source: get("source"),
        checked_by: get("checked_by"),
    }
}

/// One plain edit.
#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    Question(String),
    Explain(Part, String),
    /// The method, in the method language.
    Method(String),
    /// The cases, as `results/isolation.csv`'s text.
    Cases(String),
}

impl Edit {
    /// The template form's field it is.
    pub fn field(&self) -> &'static str {
        match self {
            Edit::Question(_) => "question",
            Edit::Explain(p, _) => p.field(),
            Edit::Method(_) => "method_text",
            Edit::Cases(_) => "case",
        }
    }
}

/// Who made the method and its numbers, and how an assistant helped:
/// `declaration.csv`'s row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Declared {
    pub author: String,
    pub ai: String,
    pub source: String,
    pub checked_by: String,
}

/// What one save or dry run asks: the edits, the declaration a method or
/// cases edit restates, and the record a decision changed carries.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Edits {
    pub edits: Vec<Edit>,
    pub declared: Option<Declared>,
    pub why: Option<Derisk>,
}

impl Edits {
    /// Which kinds of decision these edits change, each once: what needs a
    /// record of which belief broke (`vleo_sheet::derisk::about_of`).
    pub fn about(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        for e in &self.edits {
            if let Some(a) = vleo_sheet::derisk::about_of(e.field()) {
                if !out.contains(&a) {
                    out.push(a);
                }
            }
        }
        out
    }
}

/// `versions.csv`'s columns, as every group's is written.
const VERSION_COLUMNS: [&str; 11] = [
    "version",
    "date",
    "by",
    "believed",
    "tested",
    "learned",
    "changed",
    "risks",
    "cost",
    "rests_on",
    "breaks_if",
];

/// `explanation`'s `title` section set to `body`, everything else kept byte
/// for byte; a section it does not hold is placed where `order` puts it.
fn with_section(text: &str, title: &str, body: &str, order: &[String]) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let secs = crate::folder::sections(text);
    let body = body.trim_matches('\n').replace('\r', "");
    let at = secs.iter().position(|s| s.0.eq_ignore_ascii_case(title));
    let (from, to, heading) = match at {
        // The lines between its heading and the next one's.
        Some(i) => (
            secs[i].2,
            secs.get(i + 1).map_or(lines.len(), |n| n.2 - 1),
            false,
        ),
        None => {
            let rank = |t: &str| order.iter().position(|o| o.eq_ignore_ascii_case(t));
            let mine = rank(title).unwrap_or(usize::MAX);
            // Before the first section that comes after it, or at the end.
            let before = secs
                .iter()
                .find(|s| rank(&s.0).is_some_and(|r| r > mine))
                .map_or(lines.len(), |s| s.2 - 1);
            (before, before, true)
        }
    };
    let mut out: Vec<String> = lines[..from].iter().map(|l| l.to_string()).collect();
    if heading {
        if out.last().is_some_and(|l| !l.trim().is_empty()) {
            out.push(String::new());
        }
        out.push(format!("## {title}"));
    }
    out.push(String::new());
    out.extend(body.split('\n').map(str::to_string));
    if to < lines.len() {
        out.push(String::new());
    }
    out.extend(lines[to..].iter().map(|l| l.to_string()));
    let mut joined = out.join("\n");
    if to >= lines.len() && !joined.ends_with('\n') {
        joined.push('\n');
    }
    joined
}

/// The rows of `csv_text` whose `origin` is not a reference, as lines.
fn origins_refused(csv_text: &str) -> Result<(), String> {
    let (head, rows) = csv::parse(csv_text);
    let Some(i) = head.iter().position(|h| csv::name_of(h) == "origin") else {
        return Err(
            "the cases say no origin: each case says where its expected value came from".into(),
        );
    };
    let bad: Vec<String> = rows
        .iter()
        .enumerate()
        .filter_map(|(k, r)| {
            let o = r.get(i).map_or("", |v| v.trim());
            (!checks::ORIGINS.contains(&o)).then(|| format!("line {}: {o:?}", k + 2))
        })
        .collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "an expected value never comes from the code under test: each case's origin is {}, and {}",
            checks::ORIGINS.join(", "),
            bad.join("; ")
        ))
    }
}

/// The edits' content laid on `now`, before the declaration and the record
/// are asked for: what a dry run shows while either is still missing.
fn content(now: &File, uid: &str, id: &str, e: &Edits) -> Result<File, Error> {
    match shape(now, uid) {
        Shape::Release => {}
        Shape::Design => return Err(refused(format!(
            "{id} is in design/'s shape (its sheet): it is written here once one node shape is picked (F3-g0, question 9); until then it is read, asked of, and its page is today's design's"
        ))),
        Shape::Neither => return Err(refused(format!(
            "{id} holds neither a release's method and cases nor a sheet: it is written here once one node shape is picked (F3-g0, question 9)"
        ))),
    }
    if e.edits.is_empty() {
        return Err(refused(format!(
            "nothing is written to {id}: no part of it is given"
        )));
    }
    let spec = crate::folder::Spec::carried()?;
    let mut out = now.clone();
    for edit in &e.edits {
        match edit {
            Edit::Question(q) => {
                if q.trim().is_empty() {
                    return Err(refused(format!(
                        "{id}'s question is a sentence, and none is given"
                    )));
                }
                for b in out.blocks.iter_mut().filter(|b| b.uid == uid) {
                    b.question = q.trim().to_string();
                }
            }
            Edit::Explain(part, body) => {
                if body.trim().is_empty() {
                    return Err(refused(format!(
                        "{id}'s \"{}\" would say nothing: say it, or leave it as it is",
                        part.heading()
                    )));
                }
                if let Some(why) = spec.forbidden("explanation", body) {
                    return Err(refused(format!(
                        "{id}'s \"{}\" is not written: {why}",
                        part.heading()
                    )));
                }
                let order = spec.headings("explanation");
                match out
                    .texts
                    .iter_mut()
                    .find(|t| t.scope == uid && t.kind == "explanation")
                {
                    Some(t) => t.body = with_section(&t.body, part.heading(), body, &order),
                    None => out.texts.push(Text {
                        scope: uid.to_string(),
                        kind: "explanation".into(),
                        body: with_section("", part.heading(), body, &order),
                    }),
                }
            }
            Edit::Method(src) => {
                if src.trim().is_empty() {
                    return Err(refused(format!(
                        "{id}'s method would be empty: a computed node is its method"
                    )));
                }
                let body = format!("{}\n", src.replace('\r', "").trim_end_matches('\n'));
                match out
                    .texts
                    .iter_mut()
                    .find(|t| t.scope == uid && t.kind == "pseudocode")
                {
                    Some(t) => t.body = body,
                    None => out.texts.push(Text {
                        scope: uid.to_string(),
                        kind: "pseudocode".into(),
                        body,
                    }),
                }
            }
            Edit::Cases(text) => {
                origins_refused(text)
                    .map_err(|why| refused(format!("{id}'s cases are not written: {why}")))?;
                let body = format!("{}\n", text.replace('\r', "").trim_end_matches('\n'));
                set_table(&mut out, uid, "results/isolation.csv", body);
            }
        }
    }
    Ok(out)
}

fn set_table(f: &mut File, uid: &str, path: &str, csv: String) {
    match f
        .tables
        .iter_mut()
        .find(|t| t.scope == uid && t.path == path)
    {
        Some(t) => t.csv = csv,
        None => f.tables.push(Tbl {
            scope: uid.to_string(),
            path: path.to_string(),
            csv,
        }),
    }
}

/// An edit applied: the file, and which kinds of decision it changed.
#[derive(Clone, Debug, PartialEq)]
pub struct Applied {
    pub file: File,
    pub about: Vec<&'static str>,
}

/// `e` applied to `now`, the node file at `path`, as `by` at `at` (an ISO
/// 8601 time): every refusal above but a save's own, with nothing changed.
/// Pure: the copy is not read or written.
pub fn apply(now: &File, path: &str, e: &Edits, by: &str, at: &str) -> Result<Applied, Error> {
    let id = save::the_node(now, path)?;
    let b = now.blocks[0].clone();
    let mut out = content(now, &b.uid, &id, e)?;
    let restates = e
        .edits
        .iter()
        .any(|x| matches!(x, Edit::Method(_) | Edit::Cases(_)));
    if restates {
        let Some(d) = &e.declared else {
            return Err(refused(format!(
                "{id}'s method and its numbers are somebody's: say who made them, and how an assistant helped, if at all. Silence is not \"none\""
            )));
        };
        let date = at.get(..10).unwrap_or(at).to_string();
        let row = vec![
            d.author.trim().to_string(),
            d.ai.trim().to_string(),
            date,
            d.source.trim().to_string(),
            d.checked_by.trim().to_string(),
        ];
        set_table(
            &mut out,
            &b.uid,
            "declaration.csv",
            csv::write(&["author", "ai", "date", "source", "checked_by"], &[row]),
        );
        let said: Vec<String> = checks::declared(&out, &b)
            .errors()
            .map(|f| f.what.clone())
            .collect();
        if !said.is_empty() {
            return Err(refused(format!(
                "{id}'s declaration does not hold, so nothing is written: {}",
                said.join("; ")
            )));
        }
    }
    let about = e.about();
    if !about.is_empty() {
        let why = e.why.clone().unwrap_or_default();
        let missing = why.missing();
        if !missing.is_empty() {
            return Err(refused(format!(
                "this changes {}'s {} and does not say which belief broke: the record is missing {}",
                id,
                about.join(" and "),
                missing.join(", ")
            )));
        }
        let version = save::next_revision(now, path)?.to_string();
        let mut row = vec![version, at.to_string(), by.to_string()];
        for c in &VERSION_COLUMNS[3..] {
            row.push(why.get(c).trim().to_string());
        }
        let (head, mut rows) = out
            .tables
            .iter()
            .find(|t| t.scope == b.uid && t.path == "versions.csv")
            .map(|t| csv::parse(&t.csv))
            .unwrap_or_else(|| {
                (
                    VERSION_COLUMNS.iter().map(|c| c.to_string()).collect(),
                    Vec::new(),
                )
            });
        if head
            .iter()
            .map(String::as_str)
            .ne(VERSION_COLUMNS.iter().copied())
        {
            return Err(refused(format!(
                "{id}'s versions.csv has the columns {}, not {}: its record is not written over",
                head.join(","),
                VERSION_COLUMNS.join(",")
            )));
        }
        rows.push(row);
        set_table(&mut out, &b.uid, "versions.csv", csv::write(&head, &rows));
    }
    if out == *now {
        return Err(refused(format!(
            "this changes nothing in {id}: a revision is saved for work"
        )));
    }
    Ok(Applied { file: out, about })
}

/// `e` written to the node at `path` in the copy as its next revision, when
/// its bytes are still the ones whose SHA-256 is `opened`, by `by` at `at`
/// with the program `app`, through `save::save`: one writer per file, the
/// second writer stopped, nothing but the node's own rows changed.
#[allow(clippy::too_many_arguments)]
pub fn edit_in_place(
    copy: &Path,
    path: &str,
    opened: &str,
    e: &Edits,
    by: &str,
    at: &str,
    app: &str,
    group: &crate::chain::Registry,
) -> Result<Saved, Error> {
    let now = match opened_file(copy, path, opened)? {
        Opened::Now(now) => *now,
        Opened::Changed(saved) => return Ok(saved),
    };
    let applied = apply(&now, path, e, by, at)?;
    // `save` compares the bytes again under the lock: a writer between this
    // read and that one is a second writer, and nothing is written.
    save::save(copy, path, &applied.file, opened, by, app, group)
}

enum Opened {
    Now(Box<File>),
    Changed(Saved),
}

fn opened_file(copy: &Path, path: &str, opened: &str) -> Result<Opened, Error> {
    if !crate::copied::path_ok(path) {
        return Err(refused(format!("{path:?} is not a path inside the copy")));
    }
    save::opened_ok(opened)?;
    let at = copy.join(path);
    let bytes = std::fs::read(&at)
        .map_err(|e| Error::new(ErrorKind::Io, format!("{}: {e}", at.display())))?;
    if crate::copied::digest(&bytes) != opened {
        return Ok(Opened::Changed(save::second(&at, bytes)?));
    }
    Ok(Opened::Now(Box::new(save::parsed(&bytes)?)))
}

/// What a dry run found: the file as it would be saved, what signing it would
/// find and its method's report on each case; or that it changed since it was
/// opened.
#[derive(Debug)]
pub enum Tried {
    Would {
        /// The next revision, as a save would write it: with what was typed
        /// even while a refusal stands, so it is checked as it is typed.
        file: Box<File>,
        revision: u64,
        /// Why a save of it would be refused now, if it would.
        refused: Option<String>,
        findings: Findings,
        report: Option<(Vec<vleo_sheet::method::Case>, vleo_sheet::method::Report)>,
        about: Vec<&'static str>,
    },
    SecondWriter {
        writer: String,
        revision: u64,
        digest: String,
    },
}

/// The dry run of [`edit_in_place`]: nothing is written. Refused, as a save
/// is, when the person may not write the node; otherwise what a save would
/// write and why it would be refused, if it would, beside what its checks
/// find.
#[allow(clippy::too_many_arguments)]
pub fn try_edit(
    copy: &Path,
    path: &str,
    opened: &str,
    e: &Edits,
    by: &str,
    at: &str,
    app: &str,
    group: &crate::chain::Registry,
) -> Result<Tried, Error> {
    let now = match opened_file(copy, path, opened)? {
        Opened::Now(now) => *now,
        Opened::Changed(Saved::SecondWriter {
            writer,
            revision,
            digest,
            ..
        }) => {
            return Ok(Tried::SecondWriter {
                writer,
                revision,
                digest,
            })
        }
        Opened::Changed(Saved::Written { .. }) => unreachable!("a changed file is never written"),
    };
    let id = save::the_node(&now, path)?;
    let uid = now.blocks[0].uid.clone();
    let (laid, refusal) = match apply(&now, path, e, by, at) {
        Ok(a) => (a.file, None),
        // What was typed, shown even while the declaration or the record is
        // still missing; when the content itself is refused, the file as it is.
        Err(why) => (
            content(&now, &uid, &id, e).unwrap_or_else(|_| now.clone()),
            Some(why.message().to_string()),
        ),
    };
    // Who may write it is asked as a save asks it: refused, not shown.
    let (next, revision) = save::would_save(&now, path, &laid, by, app, group)?;
    let findings = checks::check_node(&next);
    let report = checks::method_report(&next, &next.blocks[0]);
    Ok(Tried::Would {
        file: Box::new(next),
        revision,
        refused: refusal,
        findings,
        report,
        about: e.about(),
    })
}
