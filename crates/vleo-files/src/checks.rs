//! What a release must hold before it is taken: the checks the developer's
//! intake made of a group's release, made of the file itself.
//!
//! docs/PLAN_1_0.md, phase C: "the library refuses everything today's intake
//! refuses". The developer's intake refused a release for what its files say —
//! a method an assistant supplied, a node that says nothing of who made it, a
//! transcription nobody checked, results that are not a reference, an input
//! that connects to nothing, a change that does not say which belief broke —
//! and for conflicts with the repository's own sheets. The first kind is about
//! the file and is checked here, the same installed and in the page. The second
//! ended with that intake, which is retired.
//!
//! A finding names its place — the node by its id, or the file — and says
//! what is wrong in words a person can act on. An error refuses the release; a
//! warning is shown and does not. Nothing is guessed: a value that is not
//! there is said to be missing, never taken as a default.

use std::collections::BTreeMap;

use crate::csv;
use crate::model::{Block, File, Port};

/// How much a finding weighs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Refuses the release.
    Error,
    /// Shown, and does not refuse.
    Warning,
}

/// One thing a check found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub level: Level,
    /// The node's id, or `file` for the release as a whole.
    pub place: String,
    pub what: String,
}

/// Everything the checks found, in the order they found it.
#[derive(Debug, Default)]
pub struct Findings(pub Vec<Finding>);

impl Findings {
    fn error(&mut self, place: &str, what: impl Into<String>) {
        self.0.push(Finding {
            level: Level::Error,
            place: place.to_string(),
            what: what.into(),
        });
    }
    fn warning(&mut self, place: &str, what: impl Into<String>) {
        self.0.push(Finding {
            level: Level::Warning,
            place: place.to_string(),
            what: what.into(),
        });
    }
    /// The errors: what refuses the release.
    pub fn errors(&self) -> impl Iterator<Item = &Finding> {
        self.0.iter().filter(|f| f.level == Level::Error)
    }
    /// Whether nothing refuses the release.
    pub fn holds(&self) -> bool {
        self.errors().next().is_none()
    }
}

/// The places that are not a node.
pub const FILE: &str = "file";

/// A table the release holds as CSV, by its scope and path, as records keyed
/// by column name (a unit in square brackets taken off), with each column's
/// unit beside.
struct Csv {
    head: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Csv {
    fn of(f: &File, scope: &str, path: &str) -> Option<Csv> {
        let t = f
            .tables
            .iter()
            .find(|t| t.scope == scope && t.path == path)?;
        let (head, rows) = csv::parse(&t.csv);
        Some(Csv { head, rows })
    }
    fn col(&self, name: &str) -> Option<usize> {
        self.head.iter().position(|h| csv::name_of(h) == name)
    }
    fn get<'a>(&'a self, row: &'a [String], name: &str) -> &'a str {
        self.col(name)
            .and_then(|i| row.get(i))
            .map_or("", String::as_str)
    }
    fn records(&self) -> Vec<BTreeMap<String, String>> {
        self.rows
            .iter()
            .map(|r| {
                self.head
                    .iter()
                    .enumerate()
                    .map(|(i, h)| {
                        (
                            csv::name_of(h).to_string(),
                            r.get(i).cloned().unwrap_or_default(),
                        )
                    })
                    .collect()
            })
            .collect()
    }
}

/// The unit a CSV header carries in square brackets, or none.
fn unit_of(header: &str) -> &str {
    let h = header.trim();
    match (h.rfind('['), h.ends_with(']')) {
        (Some(i), true) => h[i + 1..h.len() - 1].trim(),
        _ => "",
    }
}

fn number(v: &str) -> Option<f64> {
    let t = v.trim();
    if t.is_empty() {
        return None;
    }
    t.parse::<f64>().ok().filter(|x| x.is_finite())
}

/// Whether `who` is one of the names an assistant arrives under, which no
/// person signs as (AGENTS.md, rule 6). The one list, `vleo_sheet`'s.
pub fn is_assistant(who: &str) -> bool {
    vleo_sheet::form::refuse_agent_attribution(std::path::Path::new(""), who).is_err()
        && !who.trim().is_empty()
}

/// Every check of a release's content, with every finding: each node's, and
/// the release's own (its requirements, and the record of what changed).
pub fn check_release(f: &File) -> Findings {
    let mut out = Findings::default();
    blocks(f, &mut out);
    lessons_placed(f, &mut out);
    signers(f, &mut out);
    requirements(f, &mut out);
    derisking(f, &mut out);
    out
}

/// Every check of a node file's content: each of its blocks — where its inputs
/// come from, who made its method, that its method reproduces its cases — that
/// no case's expected value came from the code under test, and that no
/// assistant signed it. What a node is held to before it is signed; the
/// release's own checks (`check_release`) ask for what only a release has.
///
/// A node is signed for what it decides or computes, so one that holds
/// nothing a check can read is refused here by name, never signed unread: an
/// archived block, an open one, a stated one that states no number, and one
/// whose behaviour nothing checks yet.
pub fn check_node(f: &File) -> Findings {
    let mut out = Findings::default();
    for b in &f.blocks {
        if b.archived != 0 {
            out.error(&b.id, "is archived: an archived node is not signed");
            continue;
        }
        match b.behaviour.as_str() {
            "method" => {}
            "stated" => {
                let states = f.ports.iter().any(|p| {
                    p.block_uid == b.uid && p.direction == "out" && number(&p.value).is_some()
                });
                if !states {
                    out.error(
                        &b.id,
                        "states no value that is a number: a stated node is signed for the value it states",
                    );
                }
            }
            "open" => out.error(
                &b.id,
                "is open: nothing in it is decided or computed yet, so there is nothing to sign",
            ),
            other => out.error(
                &b.id,
                format!("is a {other} node, and nothing checks a {other} node yet: it is not signed until something does"),
            ),
        }
    }
    blocks(f, &mut out);
    lessons_placed(f, &mut out);
    cases(f, &mut out);
    signers(f, &mut out);
    out
}

/// Where a case's expected value may come from, in a node's results: never
/// from the code under test (AGENTS.md, rule 4). The one list.
pub const ORIGINS: &[&str] = &["code", "hand", "spreadsheet", "paper"];

/// The provenances that are not a reference at all: the code under test's
/// own answer, or one an assistant made. The one list.
pub const NOT_A_REFERENCE: &[&str] = &["self-snapshot", "agent-generated"];

/// No case's expected value comes from the code under test (AGENTS.md, rule
/// 4): a case whose provenance is `self-snapshot` or `agent-generated` is
/// refused, by name.
fn cases(f: &File, out: &mut Findings) {
    for c in &f.cases {
        if NOT_A_REFERENCE.contains(&c.provenance.as_str()) {
            out.error(
                &place_of(f, &c.block_uid),
                format!(
                    "its case {:?} says its expected value is {}: an expected value never comes from the code under test",
                    c.name, c.provenance
                ),
            );
        }
    }
}

/// The checks of each live block.
fn blocks(f: &File, out: &mut Findings) {
    let live: Vec<&Block> = f.blocks.iter().filter(|b| b.archived == 0).collect();
    for b in &live {
        let ports: Vec<&Port> = f.ports.iter().filter(|p| p.block_uid == b.uid).collect();
        let ins: Vec<&Port> = ports
            .iter()
            .copied()
            .filter(|p| p.direction == "in")
            .collect();
        let out_port = ports.iter().copied().find(|p| p.direction == "out");
        inputs(f, b, &ins, out);
        lesson(f, b, out);
        match b.behaviour.as_str() {
            "method" => {
                declaration(f, b, out);
                if !f.texts.iter().any(|t| {
                    t.scope == b.uid && t.kind == "pseudocode" && !t.body.trim().is_empty()
                }) {
                    out.error(
                        &b.id,
                        "has no method: a computed node is its method in the method language",
                    );
                }
                // Until one node shape is picked (F3-g0), a node may hold its
                // method twice, as a release's `pseudocode` and as design/'s
                // `method`: the checks read the one and the engine runs the
                // other, so the two must be the same (F8, question 58). Their
                // texts are compared whole, trimmed: two that differ only in a
                // comment are refused too, until F3-g0 keeps one.
                let body = |kind: &str| {
                    f.texts
                        .iter()
                        .find(|t| t.scope == b.uid && t.kind == kind)
                        .map(|t| t.body.trim().to_string())
                };
                if let (Some(p), Some(m)) = (body("pseudocode"), body("method")) {
                    if p != m {
                        out.error(
                            &b.id,
                            "holds two methods that differ, its pseudocode and its method: the checks read the one and today's design runs the other, so they are one body",
                        );
                    }
                }
                results(f, b, &ins, out_port, out);
                method(f, b, &ins, out_port, out);
            }
            "stated" => {
                if let Some(p) = out_port {
                    if p.state == "decided" && number(&p.value).is_none() {
                        out.error(
                            &b.id,
                            "is declared, so it needs a value — and has none that is a number",
                        );
                    }
                    if let (Some(lo), Some(hi)) = (number(&p.lower), number(&p.upper)) {
                        if lo > hi {
                            out.error(&b.id, "its lower bound is above its upper");
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Each input says where it comes from, and that place exists; its default is
/// a number inside its range.
fn inputs(f: &File, b: &Block, ins: &[&Port], out: &mut Findings) {
    for p in ins {
        let Some(w) = f
            .wires
            .iter()
            .find(|w| w.to_block == b.uid && w.to_port == p.name)
        else {
            out.error(
                &b.id,
                format!("the input {} does not say where it comes from", p.name),
            );
            continue;
        };
        let from = w.from_ref.trim();
        let connects = from == "case"
            || match from.split_once('.') {
                Some((head, port)) => match f.blocks.iter().find(|x| x.uid == head) {
                    // In this file: the block, and an output of it by that name;
                    // or, for a child of a broken-down node, an input of its
                    // node, which the child reads as the node does (F4).
                    Some(x) => f.ports.iter().any(|q| {
                        q.block_uid == x.uid
                            && q.name == port
                            && (q.direction == "out"
                                || (q.direction == "in" && x.uid == b.parent_uid))
                    }),
                    // Another group's: group.node, or group.node.port.
                    None => !head.is_empty() && !port.is_empty(),
                },
                None => false,
            };
        if !connects {
            out.error(
                &b.id,
                format!(
                    "the input {} comes from {from:?}, which connects to nothing: a node of this group, another group's group.node, or case",
                    p.name
                ),
            );
        }
        match number(&p.value) {
            None => out.error(&b.id, format!("the input {} has no default value", p.name)),
            Some(d) => {
                if number(&p.lower).is_some_and(|lo| d < lo) {
                    out.error(
                        &b.id,
                        format!("the input {}: the default is below its min", p.name),
                    );
                }
                if number(&p.upper).is_some_and(|hi| d > hi) {
                    out.error(
                        &b.id,
                        format!("the input {}: the default is above its max", p.name),
                    );
                }
            }
        }
    }
}

/// Who made the node's method and its numbers, and whether an assistant
/// helped. Silence is not "none": a node that does not say is taken as one an
/// assistant helped with, and refused (AGENTS.md, rule 6).
fn declaration(f: &File, b: &Block, out: &mut Findings) {
    let Some(t) = Csv::of(f, &b.uid, "declaration.csv") else {
        out.error(
            &b.id,
            "has no declaration: a node that does not say who made its method is taken as one an assistant supplied",
        );
        return;
    };
    let rows = t.records();
    let [d] = rows.as_slice() else {
        out.error(
            &b.id,
            format!("its declaration must be one row; it has {}", rows.len()),
        );
        return;
    };
    let get = |k: &str| d.get(k).map_or("", |s| s.trim());
    let author = get("author");
    if author.is_empty() {
        out.error(
            &b.id,
            "its declaration names no author: the method and its numbers are somebody's",
        );
    } else if is_assistant(author) {
        out.error(&b.id, format!("its declaration names {author:?}, an assistant's name, as the author; an assistant may never supply mathematics"));
    }
    match get("ai") {
        "none" | "wording" => {}
        "relation" => out.error(
            &b.id,
            "its declaration says an assistant supplied the relation — the method, the equations, the results or the evidence. A person derives it, or it does not go in",
        ),
        "transcribed" => {
            let (source, checker) = (get("source"), get("checked_by"));
            if source.is_empty() {
                out.error(&b.id, "says it was transcribed but names no source it was copied from");
            }
            if checker.is_empty() {
                out.error(&b.id, "says it was transcribed but names nobody who checked the copy against its source");
            } else if is_assistant(checker) {
                out.error(&b.id, format!("says {checker:?} checked the transcription; that is an assistant's name, and the copy is checked by a person"));
            }
        }
        "" => out.error(&b.id, "its declaration does not say whether an assistant helped, and silence is not none"),
        other => out.error(&b.id, format!("its declaration says ai = {other:?}; it is none, wording, relation or transcribed")),
    }
}

/// The node on its own: the reference its code must match. At least the
/// defaults, three ordinary cases and one refusal; every input a column in its
/// unit; every answer a number with a tolerance above zero.
fn results(f: &File, b: &Block, ins: &[&Port], out_port: Option<&Port>, out: &mut Findings) {
    let Some(t) = Csv::of(f, &b.uid, "results/isolation.csv") else {
        out.error(&b.id, "has no results/isolation.csv: a computed node carries the cases its code must reproduce");
        return;
    };
    for p in ins {
        match t.head.iter().find(|h| csv::name_of(h) == p.name) {
            None => out.error(
                &b.id,
                format!("its results have no column for the input {}", p.name),
            ),
            Some(h) if !p.unit.is_empty() && unit_of(h) != p.unit => out.error(
                &b.id,
                format!(
                    "its results give {} in [{}] where the input is in [{}]",
                    p.name,
                    unit_of(h),
                    p.unit
                ),
            ),
            Some(_) => {}
        }
    }
    let answers: Vec<usize> = t
        .head
        .iter()
        .enumerate()
        .filter(|(_, h)| {
            let n = csv::name_of(h);
            n == "answer" || n.starts_with("answer.")
        })
        .map(|(i, _)| i)
        .collect();
    if answers.is_empty() {
        out.error(&b.id, "its results have no answer column");
    }
    if let (Some(p), Some(&i)) = (out_port, answers.first()) {
        let u = unit_of(&t.head[i]);
        if !p.unit.is_empty() && csv::name_of(&t.head[i]) == "answer" && u != p.unit {
            out.error(
                &b.id,
                format!(
                    "its results give the answer in [{u}] where the node answers in [{}]",
                    p.unit
                ),
            );
        }
    }
    for c in ["tolerance", "refuses", "origin"] {
        if t.col(c).is_none() {
            out.error(&b.id, format!("its results have no {c} column"));
        }
    }
    let (mut ordinary, mut refusals, mut at_defaults) = (0, 0, false);
    for (k, r) in t.rows.iter().enumerate() {
        let line = k + 2;
        let refuses = match t.get(r, "refuses") {
            "yes" => true,
            "no" => false,
            v => {
                out.error(
                    &b.id,
                    format!("results line {line}: refuses is {v:?}; it is yes or no"),
                );
                false
            }
        };
        let origin = t.get(r, "origin");
        if !ORIGINS.contains(&origin) {
            out.error(&b.id, format!(
                "results line {line}: origin is {origin:?}; an expected value comes from code, hand, a spreadsheet or a paper — never from the code under test"
            ));
        }
        for p in ins {
            let v = t.get(r, &p.name);
            let odd = refuses
                && matches!(
                    v.trim().to_lowercase().as_str(),
                    "nan" | "inf" | "+inf" | "-inf" | "infinity" | "-infinity"
                );
            if number(v).is_none() && !odd {
                out.error(
                    &b.id,
                    format!("results line {line}: {} is {v:?}, not a number", p.name),
                );
            }
        }
        if refuses {
            refusals += 1;
        } else {
            ordinary += 1;
            for &i in &answers {
                if number(r.get(i).map_or("", String::as_str)).is_none() {
                    out.error(
                        &b.id,
                        format!(
                            "results line {line}: {} is not a number",
                            csv::name_of(&t.head[i])
                        ),
                    );
                }
            }
            if !number(t.get(r, "tolerance")).is_some_and(|x| x > 0.0) {
                out.error(
                    &b.id,
                    format!("results line {line}: the tolerance must be a number above zero"),
                );
            }
        }
        if !ins.is_empty()
            && ins.iter().all(|p| {
                number(t.get(r, &p.name)).is_some() && number(t.get(r, &p.name)) == number(&p.value)
            })
        {
            at_defaults = true;
        }
    }
    if !ins.is_empty() && !at_defaults {
        out.error(
            &b.id,
            "its results have no row with every input at its default",
        );
    }
    if ordinary < 3 {
        out.error(
            &b.id,
            format!("its results have {ordinary} ordinary case(s); a node needs at least 3"),
        );
    }
    if refusals < 1 {
        out.error(&b.id, "its results have no case the node must refuse (refuses = yes): a method is checked on what it refuses as well as what it answers");
    }
    for p in ins {
        for (bound, end) in [(&p.lower, "min"), (&p.upper, "max")] {
            if let Some(x) = number(bound) {
                if !t.rows.iter().any(|r| number(t.get(r, &p.name)) == Some(x)) {
                    out.warning(
                        &b.id,
                        format!("its results never test {} at its {end}, {x}", p.name),
                    );
                }
            }
        }
    }
}

/// A value as a case writes it: a number, or — where the node must refuse it —
/// a value that is not a finite one.
fn case_value(v: &str) -> Option<f64> {
    match v.trim().to_lowercase().as_str() {
        "nan" => Some(f64::NAN),
        "inf" | "+inf" | "infinity" => Some(f64::INFINITY),
        "-inf" | "-infinity" => Some(f64::NEG_INFINITY),
        t => t.parse().ok(),
    }
}

/// The method, read and run by the method language — `vleo_sheet::method`, the
/// one implementation the group pages, the gate and intake read it by: every
/// line parses, every unit agrees, every path ends, and every case comes out
/// as the node engineer's own code gave it. The release's units are its
/// contract; every value is taken to SI by them, as intake takes it.
fn method(f: &File, b: &Block, ins: &[&Port], out_port: Option<&Port>, out: &mut Findings) {
    use vleo_sheet::method as m;
    let Some((_, report)) = report_of(f, b, ins, out_port, out) else {
        return;
    };
    for d in &report.diags {
        let what = format!("its method, line {}: {}", d.line, d.msg);
        if d.severity == m::Severity::Error {
            out.error(&b.id, what);
        } else {
            out.warning(&b.id, what);
        }
    }
    // Too few cases, no refusal, no tolerance: already said by the results'
    // own checks, once.
    for (c, v) in &report.cases {
        if !v.agrees() {
            out.error(
                &b.id,
                format!("its method does not reproduce «{}»: {}", c.label, v.text(c)),
            );
        }
    }
}

/// The method of the node `b` in `f` run on its own cases, as its checks run
/// it: the cases, and the method language's report on each. `None` when it
/// holds no method or no results, or a unit does not read. What a dry run of
/// an edit answers case by case (`edit::try_edit`), by the code its checks
/// use.
pub fn method_report(
    f: &File,
    b: &Block,
) -> Option<(Vec<vleo_sheet::method::Case>, vleo_sheet::method::Report)> {
    let ports: Vec<&Port> = f.ports.iter().filter(|p| p.block_uid == b.uid).collect();
    let ins: Vec<&Port> = ports
        .iter()
        .copied()
        .filter(|p| p.direction == "in")
        .collect();
    let out_port = ports.iter().copied().find(|p| p.direction == "out");
    report_of(f, b, &ins, out_port, &mut Findings::default())
}

fn report_of(
    f: &File,
    b: &Block,
    ins: &[&Port],
    out_port: Option<&Port>,
    out: &mut Findings,
) -> Option<(Vec<vleo_sheet::method::Case>, vleo_sheet::method::Report)> {
    let (src, sig, cases) = method_cases(f, b, ins, out_port, out)?;
    let report = vleo_sheet::method::report(&src, &sig, &cases);
    Some((cases, report))
}

/// The declaration of the node `b` in `f`, held to what every node's is held
/// to (rule 6): who made its method, and how an assistant helped, if at all.
pub fn declared(f: &File, b: &Block) -> Findings {
    let mut out = Findings::default();
    declaration(f, b, &mut out);
    out
}

/// A node's method, its signature and its cases in SI, read from its file:
/// its `pseudocode`, its ports' units and its `results/isolation.csv`. The
/// one reader of a node's own cases, for its checks and for whether its
/// children reproduce them (`breakdown::reproduces`). `None`, with why in
/// `out` where a unit does not read, when it holds no method or no results.
pub fn method_cases(
    f: &File,
    b: &Block,
    ins: &[&Port],
    out_port: Option<&Port>,
    out: &mut Findings,
) -> Option<(
    String,
    vleo_sheet::method::Signature,
    Vec<vleo_sheet::method::Case>,
)> {
    use vleo_sheet::method as m;
    let src = f
        .texts
        .iter()
        .find(|t| t.scope == b.uid && t.kind == "pseudocode" && !t.body.trim().is_empty())?;
    let t = Csv::of(f, &b.uid, "results/isolation.csv")?;
    let unit = |u: &str| m::parse_unit(if u.trim().is_empty() { "1" } else { u });
    let mut bad = false;
    let mut dim = |what: &str, u: &str, out: &mut Findings| match unit(u) {
        Ok(fd) => Some(fd),
        Err(e) => {
            out.error(
                &b.id,
                format!(
                    "{what} is in [{u}], which the method language does not read: {}",
                    e.message()
                ),
            );
            bad = true;
            None
        }
    };
    let mut inputs = Vec::new();
    for p in ins {
        if let Some((_, d)) = dim(&format!("the input {}", p.name), &p.unit, out) {
            inputs.push((p.name.clone(), d));
        }
    }
    let out_unit = out_port.map_or("", |p| p.unit.as_str());
    let output = dim("the answer", out_unit, out).map(|(_, d)| d);
    // The members a node publishes beside its answer: its results'
    // answer.<member> [unit] columns.
    let mut members = Vec::new();
    for (i, h) in t.head.iter().enumerate() {
        if let Some(name) = csv::name_of(h).strip_prefix("answer.") {
            if let Some((factor, d)) = dim(&format!("the member {name}"), unit_of(h), out) {
                members.push((i, name.to_string(), factor, d));
            }
        }
    }
    let (Some(output), false) = (output, bad) else {
        return None;
    };
    // Every column's factor to SI, by its own header.
    let factor_of = |i: usize| unit(unit_of(&t.head[i])).map_or(1.0, |(fct, _)| fct);
    let answer = t.head.iter().position(|h| csv::name_of(h) == "answer");
    let mut cases = Vec::new();
    for (k, r) in t.rows.iter().enumerate() {
        let refuses = t.get(r, "refuses") == "yes";
        let mut inputs_si = Vec::new();
        for p in ins {
            let Some(i) = t.col(&p.name) else { continue };
            if let Some(v) = case_value(r.get(i).map_or("", String::as_str)) {
                inputs_si.push((p.name.clone(), v * factor_of(i)));
            }
        }
        let value =
            |i: usize| case_value(r.get(i).map_or("", String::as_str)).map(|v| v * factor_of(i));
        let label = t.get(r, "says");
        cases.push(m::Case {
            label: if label.is_empty() {
                format!("results line {}", k + 2)
            } else {
                label.to_string()
            },
            inputs: inputs_si,
            expect: if refuses {
                None
            } else {
                answer.and_then(value)
            },
            tolerance: if refuses {
                0.0
            } else {
                number(t.get(r, "tolerance")).unwrap_or(0.0)
            },
            also: if refuses {
                Vec::new()
            } else {
                members
                    .iter()
                    .filter_map(|(i, name, _, _)| value(*i).map(|v| (name.clone(), v)))
                    .collect()
            },
            origin: t.get(r, "origin").to_string(),
        });
    }
    let sig = m::Signature {
        inputs,
        output,
        publishes: members
            .iter()
            .map(|(_, name, _, d)| (name.clone(), *d))
            .collect(),
    };
    Some((src.body.clone(), sig, cases))
}

/// A node's method signature, read from its own ports: its inputs by name
/// and dimension, in the file's order; its first output the answer, and the
/// others the members it publishes. What a child's method is checked against
/// (`child_ready`) and what a typed formula is read against (F8-a3). A port
/// in a unit the method language does not read is a finding, by name, and a
/// node with no output has no signature.
pub fn node_signature(
    f: &File,
    b: &Block,
    out: &mut Findings,
) -> Option<vleo_sheet::method::Signature> {
    use vleo_sheet::method as m;
    let unit = |u: &str| m::parse_unit(if u.trim().is_empty() { "1" } else { u.trim() });
    let dims = |p: &Port, out: &mut Findings| match unit(&p.unit) {
        Ok((_, d)) => Some((p.name.clone(), d)),
        Err(e) => {
            out.error(
                &b.id,
                format!(
                    "its {} {} is in [{}], which the method language does not read: {}",
                    if p.direction == "in" {
                        "input"
                    } else {
                        "output"
                    },
                    p.name,
                    p.unit,
                    e.message()
                ),
            );
            None
        }
    };
    let ports: Vec<&Port> = f.ports.iter().filter(|p| p.block_uid == b.uid).collect();
    let inputs: Vec<_> = ports
        .iter()
        .filter(|p| p.direction == "in")
        .filter_map(|p| dims(p, out))
        .collect();
    let outs: Vec<_> = ports
        .iter()
        .filter(|p| p.direction == "out")
        .filter_map(|p| dims(p, out))
        .collect();
    let Some(((_, output), publishes)) = outs.split_first() else {
        out.error(&b.id, "has no output for its method to answer");
        return None;
    };
    Some(m::Signature {
        inputs,
        output: *output,
        publishes: publishes.to_vec(),
    })
}

/// A child of a broken-down node, held to what it must be before it is run
/// on its node's cases (`breakdown::reproduces`): it says who made its
/// method (rule 6, as `declaration` holds a node to), and its method reads
/// and checks against its own ports' units, its first output its answer and
/// the others the members it publishes. Its own cases are not asked for:
/// its node's are what it is judged on.
pub fn child_ready(f: &File, b: &Block) -> Findings {
    use vleo_sheet::method as m;
    let mut out = Findings::default();
    declaration(f, b, &mut out);
    let Some(src) = f
        .texts
        .iter()
        .find(|t| t.scope == b.uid && t.kind == "pseudocode" && !t.body.trim().is_empty())
    else {
        // No method is said where the child is needed, by its behaviour.
        return out;
    };
    let Some(sig) = node_signature(f, b, &mut out) else {
        return out;
    };
    match m::parse(&src.body) {
        Err(d) => out.error(&b.id, format!("its method, line {}: {}", d.line, d.msg)),
        Ok(program) => {
            for d in m::check(&program, &sig) {
                if d.severity == m::Severity::Error {
                    out.error(&b.id, format!("its method, line {}: {}", d.line, d.msg));
                }
            }
        }
    }
    out
}

/// A lesson belongs to a row (docs/LESSONS.md): in a node file, on its own
/// block; in a release, on one of its nodes. One on a group file, on the file
/// itself or on a block the file does not hold is refused by name, never
/// passed and left unread.
fn lessons_placed(f: &File, out: &mut Findings) {
    let kind = f.kind().ok();
    for t in f.texts.iter().filter(|t| t.kind == crate::convert::LESSON) {
        let block = f.blocks.iter().find(|b| b.uid == t.scope);
        let placed = match (kind, block) {
            (Some(crate::meta::Kind::Node), Some(b)) => f.meta.get("block_uid") == Some(&b.uid),
            (Some(crate::meta::Kind::GroupRelease), Some(b)) => b.behaviour != "children",
            _ => false,
        };
        if !placed {
            out.error(
                &place_of(f, &t.scope),
                format!(
                    "holds a lesson on {:?}: a lesson belongs to a row, in its node file",
                    t.scope
                ),
            );
        }
    }
}

/// A block's lesson (docs/LESSONS.md), held to the check the gate runs on
/// it, all but what it names in the rest of the design: a key the reader does
/// not know, markup, a claim not tagged or a sourced one with no source, a
/// check whose answer is not an option. Whether a widget's rows are there and
/// may be moved is the design's to say, and the gate checks it once the design
/// holds the lesson, as the program does when it is written.
fn lesson(f: &File, b: &Block, out: &mut Findings) {
    let Some(t) = f
        .texts
        .iter()
        .find(|t| t.scope == b.uid && t.kind == crate::convert::LESSON)
    else {
        return;
    };
    match vleo_sheet::lesson::read(&t.body, &b.id) {
        Err(e) => out.error(&b.id, format!("its lesson does not read: {}", e.message())),
        Ok(l) => {
            let any_row = |_: &str| {
                Some(vleo_sheet::lesson::RowKind {
                    declared: true,
                    seeded: false,
                })
            };
            for p in vleo_sheet::lesson::problems_with(&l, &any_row) {
                out.error(&b.id, format!("its lesson: {p}"));
            }
        }
    }
}

/// No signature is made under an assistant's name (AGENTS.md, rule 6).
fn signers(f: &File, out: &mut Findings) {
    for s in &f.signatures {
        if is_assistant(&s.signer) {
            out.error(
                &place_of(f, &s.scope),
                format!(
                    "is signed by {:?}, an assistant's name; a signature is a person's",
                    s.signer
                ),
            );
        }
    }
}

/// Where a signature of `scope` is: its block, the node a request is on, or
/// the file.
fn place_of(f: &File, scope: &str) -> String {
    let block = scope
        .strip_prefix(crate::chain::REQUEST)
        .and_then(|uid| f.requests.iter().find(|r| r.uid == uid))
        .map_or(scope, |r| r.scope.as_str());
    f.blocks
        .iter()
        .find(|b| b.uid == block)
        .map_or(FILE.to_string(), |b| b.id.clone())
}

/// Every requirement says which way it binds, and names nodes that exist.
fn requirements(f: &File, out: &mut Findings) {
    let Some(t) = Csv::of(f, FILE, "requirements.csv") else {
        return;
    };
    for (k, r) in t.records().iter().enumerate() {
        let id = r.get("id").map_or("", String::as_str);
        let id = if id.is_empty() {
            format!("line {}", k + 2)
        } else {
            id.to_string()
        };
        match r.get("sense").map(|s| s.trim()) {
            Some("<=") | Some(">=") => {}
            Some("") | None => out.error(FILE, format!(
                "requirement {id} does not say which way it binds; a sense is never defaulted — <= stays under, >= reaches"
            )),
            Some(other) => out.error(FILE, format!("requirement {id} has the sense {other:?}; it is <= or >=")),
        }
        for side in ["required", "achieved"] {
            let n = r.get(side).map_or("", String::as_str);
            if !f.blocks.iter().any(|b| b.id == n) {
                out.error(
                    FILE,
                    format!("requirement {id}: the {side} node {n:?} does not exist"),
                );
            }
        }
    }
}

/// The release says which belief broke: its version has a row in the
/// group's record, saying what it believed, tested, learned and changed, what
/// it rests on and what would break it — or, for a group's first version,
/// what it rests on and what would break it (docs/DERISKING.md).
fn derisking(f: &File, out: &mut Findings) {
    let version = f.meta.get("version").cloned().unwrap_or_default();
    let Some(t) = Csv::of(f, FILE, "versions.csv") else {
        out.error(
            FILE,
            "has no versions.csv: every change says which belief broke",
        );
        return;
    };
    let rows = t.records();
    let Some((k, row)) = rows
        .iter()
        .enumerate()
        .find(|(_, r)| r.get("version").map(|v| v.trim()) == Some(version.as_str()))
    else {
        out.error(FILE, format!("versions.csv has no row for version {version}: say what this version changed and why"));
        return;
    };
    let need: &[&str] = if k == 0 {
        &["rests_on", "breaks_if"]
    } else {
        &[
            "believed",
            "tested",
            "learned",
            "changed",
            "rests_on",
            "breaks_if",
        ]
    };
    let missing: Vec<&str> = need
        .iter()
        .copied()
        .filter(|c| row.get(*c).is_none_or(|v| v.trim().is_empty()))
        .collect();
    if !missing.is_empty() {
        out.error(
            FILE,
            format!(
                "version {version} does not say which belief broke — its record is missing: {}",
                missing.join(", ")
            ),
        );
    }
}
