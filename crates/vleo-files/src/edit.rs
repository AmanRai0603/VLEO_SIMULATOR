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
//! - **the cases**: its `results/isolation.csv`, whole; or a table pasted
//!   from a spreadsheet, or a CSV file's text, read as them against the
//!   node's ports ([`table_of`], F8-b1), each number in its column's unit
//!   said in its port's, and where its expected values came from said once
//!   or by its own `origin` column, never filled in.
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
use crate::model::{File, Port, Tbl, Text};
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
    /// A formula as the person types it (F8-a3), written as the method text
    /// [`formula_of`] makes of it, and held to everything a method is.
    Formula(Formula),
    /// A table pasted from a spreadsheet, or a CSV file's text (F8-b1),
    /// written as the cases [`table_of`] makes of it, and held to everything
    /// cases are.
    Table(Pasted),
}

/// A table as a person pastes it, or a CSV file's text (docs/briefs/F8.md,
/// F8-b item 1), and where its expected values came from when the table
/// does not say in its own `origin` column (question 72): never filled in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pasted {
    pub text: String,
    pub origin: String,
}

/// A formula as a person types it (docs/briefs/F8.md, F8-a items 3 and 4):
/// `typed`, `NAME = EXPR`, a bare expression, or lines whose last is the
/// answer; and, for an empirical fit, the unit each input is read in and the
/// answer's, as the person names them (question 69). The program never
/// chooses a unit.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Formula {
    pub typed: String,
    pub fit: Option<vleo_sheet::method::formula::Fit>,
}

impl Edit {
    /// The template form's field it is.
    pub fn field(&self) -> &'static str {
        match self {
            Edit::Question(_) => "question",
            Edit::Explain(p, _) => p.field(),
            Edit::Method(_) | Edit::Formula(_) => "method_text",
            Edit::Cases(_) | Edit::Table(_) => "case",
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

/// Whether `body` holds a `## ` heading outside a fence, or leaves a fence
/// open: either would make one section's text another's.
fn body_refused(body: &str) -> Option<&'static str> {
    let mut fence = false;
    for l in body.split('\n') {
        let lead = l.trim_start();
        if lead.starts_with("```") || lead.starts_with("~~~") {
            fence = !fence;
            continue;
        }
        if !fence
            && l.strip_prefix("##")
                .is_some_and(|r| r.starts_with(|c: char| c.is_whitespace()))
        {
            return Some("it holds a \"## \" heading, which would begin another section");
        }
    }
    fence.then_some("it leaves a code fence open, which would swallow every section after it")
}

/// `explanation`'s `title` section set to `body`, everything else kept byte
/// for byte; a section it does not hold is placed where `order` puts it. The
/// line endings are the text's own. Refused when the body would begin or
/// hide a section, or when the text read again does not hold every other
/// section as it was (a fence the text already leaves open, say).
fn with_section(text: &str, title: &str, body: &str, order: &[String]) -> Result<String, String> {
    let body = body.replace('\r', "");
    let body = body.trim_matches('\n');
    if let Some(why) = body_refused(body) {
        return Err(why.to_string());
    }
    let crlf = text.contains("\r\n");
    let line = |l: &str| {
        if crlf {
            format!("{l}\r")
        } else {
            l.to_string()
        }
    };
    let lines: Vec<&str> = if text.is_empty() {
        Vec::new()
    } else {
        text.split('\n').collect()
    };
    let secs = crate::folder::sections(text);
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
            out.push(line(""));
        }
        out.push(line(&format!("## {title}")));
    }
    out.push(line(""));
    out.extend(body.split('\n').map(line));
    if to < lines.len() {
        out.push(line(""));
    }
    out.extend(lines[to..].iter().map(|l| l.to_string()));
    let mut joined = out.join("\n");
    if to >= lines.len() && !joined.ends_with('\n') {
        joined.push_str(if crlf { "\r\n" } else { "\n" });
    }
    // Read again: every other section as it was, in its place, and this one
    // holding what was typed.
    let after = crate::folder::sections(&joined);
    let mut want: Vec<String> = secs.iter().map(|s| s.0.to_lowercase()).collect();
    if heading {
        let k = secs.iter().position(|s| s.2 > from).unwrap_or(secs.len());
        want.insert(k, title.to_lowercase());
    }
    let got: Vec<String> = after.iter().map(|s| s.0.to_lowercase()).collect();
    let kept = got == want
        && after.iter().all(|a| {
            if a.0.eq_ignore_ascii_case(title) {
                a.1.trim_matches('\n') == body
            } else {
                secs.iter()
                    .find(|b| b.0 == a.0)
                    .is_some_and(|b| b.1.trim_end_matches('\n') == a.1.trim_end_matches('\n'))
            }
        });
    if !kept {
        return Err(format!(
            "read again, the explanation would not hold its other sections as they were ({} before, {} after): a fence or heading in it already runs across sections",
            secs.iter().map(|s| s.0.as_str()).collect::<Vec<_>>().join(", "),
            after.iter().map(|s| s.0.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }
    Ok(joined)
}

/// Whether every row of `csv_text` reads, and says where its expected value
/// came from: one of [`checks::ORIGINS`]. Each refusal names its line.
fn origins_refused(csv_text: &str) -> Result<(), String> {
    let p = csv::read(csv_text);
    if let Some((line, why)) = p.problems.first() {
        return Err(format!("the cases do not read as CSV, line {line}: {why}"));
    }
    let Some(i) = p.head.iter().position(|h| csv::name_of(h) == "origin") else {
        return Err(
            "the cases say no origin: each case says where its expected value came from".into(),
        );
    };
    let bad: Vec<String> = p
        .rows
        .iter()
        .enumerate()
        .filter_map(|(k, r)| {
            let o = r.get(i).map_or("", |v| v.trim());
            let line = p.lines.get(k).copied().unwrap_or(k + 2);
            (!checks::ORIGINS.contains(&o)).then(|| format!("line {line}: {o:?}"))
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

/// A formula typed for node `uid` of `f`, read against the node's own ports
/// (`checks::node_signature`): its inputs found by name, else by symbol, its
/// answer the node's first output, and the method text it becomes, checked
/// by `compile` (`vleo_sheet::method::formula::read`). Nothing is written
/// and nothing runs: what the cases say of it is the dry run's. Refused by
/// name when the node's ports give no signature.
pub fn formula_of(
    f: &File,
    uid: &str,
    fm: &Formula,
) -> Result<vleo_sheet::method::formula::Read, Error> {
    use vleo_sheet::method::formula;
    let b = f
        .blocks
        .iter()
        .find(|b| b.uid == uid)
        .ok_or_else(|| refused(format!("no node {uid:?} is in the file")))?;
    let typed = fm.typed.replace('\r', "");
    if typed.trim().is_empty() {
        return Err(refused(format!(
            "{}'s formula is empty: type it as NAME = EXPR, as D = 0.5 * rho * v^2 * Cd * A",
            b.id
        )));
    }
    let mut findings = Findings::default();
    let sig = checks::node_signature(f, b, &mut findings);
    let bad: Vec<String> = findings.errors().map(|x| x.what.clone()).collect();
    let sig = match sig {
        Some(sig) if bad.is_empty() => sig,
        _ => {
            return Err(refused(format!(
                "{}'s formula is not read, for its ports give no method signature: {}",
                b.id,
                bad.join("; ")
            )))
        }
    };
    // A formula answers one output. A node that publishes values beside its
    // answer, by its ports or by its cases' columns, is written as a method
    // (the brief's "Not in F8-a"), and never read here as one that does not.
    let published: Vec<String> = sig
        .publishes
        .iter()
        .map(|(n, _)| n.clone())
        .chain(
            f.tables
                .iter()
                .filter(|t| t.scope == uid && t.path == "results/isolation.csv")
                .flat_map(|t| {
                    t.csv
                        .lines()
                        .next()
                        .unwrap_or("")
                        .split(',')
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .filter_map(|h| {
                    h.trim()
                        .strip_prefix("answer.")
                        .map(|m| m.split(' ').next().unwrap_or("").to_string())
                }),
        )
        .collect();
    if !published.is_empty() {
        return Err(refused(format!(
            "{}'s formula is not read: it publishes {} beside its answer, and a formula answers one output, so a method with several values is written as a method",
            b.id,
            published.join(", ")
        )));
    }
    let port = |p: &Port| formula::Port {
        name: p.name.clone(),
        symbol: p.symbol.trim().to_string(),
        unit: p.unit.trim().to_string(),
    };
    let ours = || f.ports.iter().filter(|p| p.block_uid == uid);
    let ins: Vec<formula::Port> = ours().filter(|p| p.direction == "in").map(port).collect();
    let output = ours()
        .find(|p| p.direction == "out")
        .map(port)
        .expect("a signature has an output");
    Ok(formula::read(&typed, &sig, &ins, &output, fm.fit.as_ref()))
}

/// A table pasted for node `uid` of `f`, read as its cases against the
/// node's own ports (`crate::table::cases`): each cell said back, each input
/// and the answer said in its port's unit. Nothing is written and nothing
/// runs: what the method says of each case is the dry run's. Refused by name
/// when the node has no output, or the table does not read.
pub fn table_of(f: &File, uid: &str, t: &Pasted) -> Result<crate::table::Read, Error> {
    let b = f
        .blocks
        .iter()
        .find(|b| b.uid == uid)
        .ok_or_else(|| refused(format!("no node {uid:?} is in the file")))?;
    let ours = || f.ports.iter().filter(|p| p.block_uid == uid);
    let inputs: Vec<&Port> = ours().filter(|p| p.direction == "in").collect();
    let output = ours().find(|p| p.direction == "out").ok_or_else(|| {
        refused(format!(
            "{} has no output, so a table gives it no answer to check",
            b.id
        ))
    })?;
    crate::table::cases(&t.text, &inputs, output, &t.origin)
        .map_err(|why| refused(format!("{}'s table is not read: {why}", b.id)))
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
    if e.edits.iter().any(|x| matches!(x, Edit::Method(_)))
        && e.edits.iter().any(|x| matches!(x, Edit::Formula(_)))
    {
        return Err(refused(format!(
            "{id} is given a method and a formula, both: a formula becomes its method, so give one"
        )));
    }
    if e.edits.iter().any(|x| matches!(x, Edit::Cases(_)))
        && e.edits.iter().any(|x| matches!(x, Edit::Table(_)))
    {
        return Err(refused(format!(
            "{id} is given cases and a table, both: a table becomes its cases, so give one"
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
                    Some(t) => {
                        t.body =
                            with_section(&t.body, part.heading(), body, &order).map_err(|why| {
                                refused(format!(
                                    "{id}'s \"{}\" is not written: {why}",
                                    part.heading()
                                ))
                            })?
                    }
                    None => out.texts.push(Text {
                        scope: uid.to_string(),
                        kind: "explanation".into(),
                        body: with_section("", part.heading(), body, &order).map_err(|why| {
                            refused(format!(
                                "{id}'s \"{}\" is not written: {why}",
                                part.heading()
                            ))
                        })?,
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
            Edit::Formula(fm) => {
                let r = formula_of(&out, uid, fm)?;
                let text = match (r.ok(), &r.method) {
                    (true, Some(text)) => text,
                    _ => {
                        return Err(refused(format!(
                            "{id}'s formula is not written: {}",
                            r.diags
                                .iter()
                                .filter(|d| d.severity == vleo_sheet::method::Severity::Error)
                                .map(|d| if d.line == 0 {
                                    d.msg.clone()
                                } else {
                                    format!("line {}: {}", d.line, d.msg)
                                })
                                .collect::<Vec<_>>()
                                .join("; ")
                        )))
                    }
                };
                let body = format!("{}\n", text.trim_end_matches('\n'));
                // The method's own text, as every method edit writes it; and,
                // while one node shape is not picked (F3-g0), design/'s
                // `method` with the same body when the file holds one, so the
                // checks and the engine read one method (question 58). No node
                // written here holds one today: design/'s nodes hold a sheet,
                // and are refused above. The write is for a node brought to
                // the release's shape before F3-g0; the checks' refusal of two
                // bodies that differ is what holds now.
                match out
                    .texts
                    .iter_mut()
                    .find(|t| t.scope == uid && t.kind == "pseudocode")
                {
                    Some(t) => t.body = body.clone(),
                    None => out.texts.push(Text {
                        scope: uid.to_string(),
                        kind: "pseudocode".into(),
                        body: body.clone(),
                    }),
                }
                for t in out
                    .texts
                    .iter_mut()
                    .filter(|t| t.scope == uid && t.kind == "method")
                {
                    t.body = body.clone();
                }
            }
            Edit::Cases(text) => {
                origins_refused(text)
                    .map_err(|why| refused(format!("{id}'s cases are not written: {why}")))?;
                let body = format!("{}\n", text.replace('\r', "").trim_end_matches('\n'));
                set_table(&mut out, uid, "results/isolation.csv", body);
            }
            Edit::Table(t) => {
                let read = table_of(now, uid, t)?;
                origins_refused(&read.csv)
                    .map_err(|why| refused(format!("{id}'s cases are not written: {why}")))?;
                set_table(&mut out, uid, "results/isolation.csv", read.csv);
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

/// Which of the template form's fields `out` holds otherwise than `now`, for
/// the node `uid`: what an edit changed, not what it sent.
fn changed_fields(now: &File, out: &File, uid: &str) -> Vec<&'static str> {
    let q = |f: &File| {
        f.blocks
            .iter()
            .find(|b| b.uid == uid)
            .map(|b| b.question.clone())
    };
    let text = |f: &File, k: &str| {
        f.texts
            .iter()
            .find(|t| t.scope == uid && t.kind == k)
            .map(|t| t.body.clone())
    };
    let table = |f: &File, p: &str| {
        f.tables
            .iter()
            .find(|t| t.scope == uid && t.path == p)
            .map(|t| t.csv.clone())
    };
    let mut out_fields = Vec::new();
    if q(now) != q(out) {
        out_fields.push("question");
    }
    for part in [Part::Simply, Part::Breaks, Part::Wrong] {
        if section(now, uid, part) != section(out, uid, part) {
            out_fields.push(part.field());
        }
    }
    if text(now, "pseudocode") != text(out, "pseudocode") {
        out_fields.push("method_text");
    }
    if table(now, "results/isolation.csv") != table(out, "results/isolation.csv") {
        out_fields.push("case");
    }
    out_fields
}

/// Which kinds of decision `fields` change, each once
/// (`vleo_sheet::derisk::about_of`).
fn about_of(fields: &[&str]) -> Vec<&'static str> {
    let mut out = Vec::new();
    for f in fields {
        if let Some(a) = vleo_sheet::derisk::about_of(f) {
            if !out.contains(&a) {
                out.push(a);
            }
        }
    }
    out
}

/// `d` written as the node's declaration, dated `at`.
fn declare(out: &mut File, uid: &str, d: &Declared, at: &str) {
    let date = at.get(..10).unwrap_or(at).to_string();
    let row = vec![
        d.author.trim().to_string(),
        d.ai.trim().to_string(),
        date,
        d.source.trim().to_string(),
        d.checked_by.trim().to_string(),
    ];
    set_table(
        out,
        uid,
        "declaration.csv",
        csv::write(&["author", "ai", "date", "source", "checked_by"], &[row]),
    );
}

/// `e` applied to `now`, the node file at `path`, as `by` at `at` (an ISO
/// 8601 time): every refusal above but a save's own, with nothing changed.
/// Pure: the copy is not read or written.
pub fn apply(now: &File, path: &str, e: &Edits, by: &str, at: &str) -> Result<Applied, Error> {
    let id = save::the_node(now, path)?;
    let b = now.blocks[0].clone();
    let mut out = content(now, &b.uid, &id, e)?;
    // What changed is what the content changed: a part sent as it already
    // is changes nothing, and asks for no declaration and no record.
    let changed = changed_fields(now, &out, &b.uid);
    if changed.is_empty() {
        return Err(refused(format!(
            "this changes nothing in {id}: a revision is saved for work"
        )));
    }
    if changed.iter().any(|c| *c == "method_text" || *c == "case") {
        let Some(d) = &e.declared else {
            return Err(refused(format!(
                "{id}'s method and its numbers are somebody's: say who made them, and how an assistant helped, if at all. Silence is not \"none\""
            )));
        };
        declare(&mut out, &b.uid, d, at);
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
    let about = about_of(&changed);
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
        /// Whether what was typed could be checked: false when its content
        /// is refused, and then no finding or report is given in its place.
        checked: bool,
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
    // What was typed, laid on the file even while the declaration or the
    // record is still missing, its declaration as typed, so it is checked as
    // it is typed. When the content itself is refused, nothing typed can be
    // checked, and none of the file's own checks is shown in its place.
    let preview = content(&now, &uid, &id, e).ok().map(|mut f| {
        if let Some(d) = &e.declared {
            if changed_fields(&now, &f, &uid)
                .iter()
                .any(|c| *c == "method_text" || *c == "case")
            {
                declare(&mut f, &uid, d, at);
            }
        }
        f
    });
    let (laid, refusal, about) = match apply(&now, path, e, by, at) {
        Ok(a) => (Some(a.file), None, a.about),
        Err(why) => (
            preview.clone(),
            Some(why.message().to_string()),
            preview
                .as_ref()
                .map(|f| about_of(&changed_fields(&now, f, &uid)))
                .unwrap_or_default(),
        ),
    };
    // Who may write it is asked as a save asks it: refused, not shown.
    let (next, revision) =
        save::would_save(&now, path, laid.as_ref().unwrap_or(&now), by, app, group)?;
    let checked = laid.is_some();
    let (findings, report) = if checked {
        (
            checks::check_node(&next),
            checks::method_report(&next, &next.blocks[0]),
        )
    } else {
        (Findings::default(), None)
    };
    Ok(Tried::Would {
        file: Box::new(next),
        revision,
        refused: refusal,
        checked,
        findings,
        report,
        about,
    })
}
