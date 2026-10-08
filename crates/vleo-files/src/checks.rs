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

/// Every check of a release's content, with every finding.
pub fn check_release(f: &File) -> Findings {
    let mut out = Findings::default();
    let live: Vec<&Block> = f.blocks.iter().filter(|b| b.archived == 0).collect();
    for b in &live {
        let ports: Vec<&Port> = f.ports.iter().filter(|p| p.block_uid == b.uid).collect();
        let ins: Vec<&Port> = ports
            .iter()
            .copied()
            .filter(|p| p.direction == "in")
            .collect();
        let out_port = ports.iter().copied().find(|p| p.direction == "out");
        inputs(f, b, &ins, &mut out);
        match b.behaviour.as_str() {
            "method" => {
                declaration(f, b, &mut out);
                if !f.texts.iter().any(|t| {
                    t.scope == b.uid && t.kind == "pseudocode" && !t.body.trim().is_empty()
                }) {
                    out.error(
                        &b.id,
                        "has no method: a computed node is its method in the method language",
                    );
                }
                results(f, b, &ins, out_port, &mut out);
                method(f, b, &ins, out_port, &mut out);
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
    signers(f, &mut out);
    requirements(f, &mut out);
    derisking(f, &mut out);
    out
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
                    // In this file: the block, and an output of it by that name.
                    Some(x) => f
                        .ports
                        .iter()
                        .any(|q| q.block_uid == x.uid && q.direction == "out" && q.name == port),
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
        if !["code", "hand", "spreadsheet", "paper"].contains(&origin) {
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
/// one implementation the node form, the gate and intake read it by: every
/// line parses, every unit agrees, every path ends, and every case comes out
/// as the node engineer's own code gave it. The release's units are its
/// contract; every value is taken to SI by them, as intake takes it.
fn method(f: &File, b: &Block, ins: &[&Port], out_port: Option<&Port>, out: &mut Findings) {
    use vleo_sheet::method as m;
    let Some(src) = f
        .texts
        .iter()
        .find(|t| t.scope == b.uid && t.kind == "pseudocode" && !t.body.trim().is_empty())
    else {
        return;
    };
    let Some(t) = Csv::of(f, &b.uid, "results/isolation.csv") else {
        return;
    };
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
        return;
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
    let report = m::report(&src.body, &sig, &cases);
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

fn place_of(f: &File, scope: &str) -> String {
    f.blocks
        .iter()
        .find(|b| b.uid == scope)
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
