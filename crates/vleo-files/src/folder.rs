//! The group folder's checks: what the page's group application says of a
//! group's folder, said by the library.
//!
//! docs/PLAN_1_0.md, phase C: "About 70 checks exist only in the page.
//! `web/js/gcheck.js` and the seal rules in `web/js/gseal.js` move into the
//! library." This is `gcheck.js`, check for check, read from the same pattern
//! (`groups/SPEC.toml`): which files a folder must have, which columns each
//! table carries and what they allow, which headings each text needs and in
//! what order, and the checks that cross files — an input from a node that is
//! not there, an embed that names nothing, a figure without its columns, a
//! citation of nothing. Every finding has the level, the place, the words and
//! the line the page gives it, so the two can be held to each other finding
//! for finding (`tools/files_check.mjs`).
//!
//! Not here yet: the page's maths reader (`web/js/texmath.js`), which only
//! ever warns — a LaTeX formula it cannot typeset, a gap in an equation drawn
//! from the method. It is a parser of its own, and stays in the page until it
//! moves on its own.
//!
//! Three levels, as the page has them: an error stops the folder being
//! sealed, a warning is shown and does not, a note is worth knowing.

use std::collections::{BTreeMap, BTreeSet};

use crate::csv::{self, Parsed};
use crate::error::{Error, ErrorKind};
use crate::format_1::Folder;

/// The group folder pattern, as the page carries it.
pub const SPEC: &str = include_str!("../../../groups/SPEC.toml");

/// How much a finding weighs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warning,
    Note,
}

impl Level {
    pub fn name(self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Note => "note",
        }
    }
}

/// One thing the checks found: where, what, and on which line (0 for none).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub level: Level,
    pub place: String,
    pub msg: String,
    pub line: usize,
}

struct Column {
    name: String,
    says: String,
    optional: bool,
    prefix: bool,
    one_of: Option<Vec<String>>,
}

struct FileRule {
    path: String,
    level: String,
    required: bool,
    kinds: Option<Vec<String>>,
    columns: Option<Vec<Column>>,
    rows: String,
    says: String,
    assistants: Vec<String>,
}

struct TextRule {
    role: String,
    headings: Vec<String>,
    forbid: Vec<String>,
    forbid_says: String,
}

/// The pattern's rules: files, texts and limits.
pub struct Spec {
    files: Vec<FileRule>,
    texts: Vec<TextRule>,
    file_bytes: Option<f64>,
    file_bytes_says: String,
    refused_types: Vec<String>,
    refused_says: String,
}

fn strs(v: Option<&toml::Value>) -> Vec<String> {
    v.and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn text(v: &toml::Value, k: &str) -> String {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

impl Spec {
    /// The pattern the library carries.
    pub fn carried() -> Result<Spec, Error> {
        Spec::read(SPEC)
    }

    /// A pattern from its TOML.
    pub fn read(toml_text: &str) -> Result<Spec, Error> {
        let v: toml::Value = toml_text.parse().map_err(|e| {
            Error::new(
                ErrorKind::Malformed,
                format!("the group pattern is not TOML: {e}"),
            )
        })?;
        let arr = |k: &str| {
            v.get(k)
                .and_then(|a| a.as_array())
                .cloned()
                .unwrap_or_default()
        };
        let files = arr("file")
            .iter()
            .map(|r| FileRule {
                path: text(r, "path"),
                level: text(r, "level"),
                required: r.get("required").and_then(|x| x.as_bool()).unwrap_or(false),
                kinds: r.get("kinds").map(|k| strs(Some(k))),
                columns: r.get("columns").and_then(|c| c.as_array()).map(|cs| {
                    cs.iter()
                        .map(|c| Column {
                            name: text(c, "name"),
                            says: text(c, "says"),
                            optional: c.get("optional").and_then(|x| x.as_bool()).unwrap_or(false),
                            prefix: c.get("prefix").and_then(|x| x.as_bool()).unwrap_or(false),
                            one_of: c.get("one_of").map(|o| strs(Some(o))),
                        })
                        .collect()
                }),
                rows: text(r, "rows"),
                says: text(r, "says"),
                assistants: strs(r.get("assistants")),
            })
            .collect();
        let texts = arr("text")
            .iter()
            .map(|t| TextRule {
                role: text(t, "role"),
                headings: strs(t.get("headings")),
                forbid: strs(t.get("forbid")),
                forbid_says: text(t, "forbid_says"),
            })
            .collect();
        let lim = v.get("limits");
        Ok(Spec {
            files,
            texts,
            file_bytes: lim
                .and_then(|l| l.get("file_bytes"))
                .and_then(|x| x.as_integer().map(|i| i as f64).or_else(|| x.as_float())),
            file_bytes_says: lim.map_or(String::new(), |l| text(l, "file_bytes_says")),
            refused_types: strs(lim.and_then(|l| l.get("refused_types"))),
            refused_says: lim.map_or(String::new(), |l| text(l, "refused_says")),
        })
    }
}

type Record = (BTreeMap<String, String>, usize);

fn records(t: &Parsed) -> Vec<Record> {
    t.rows
        .iter()
        .zip(&t.lines)
        .map(|(r, &line)| {
            let mut o = BTreeMap::new();
            for (k, h) in t.head.iter().enumerate() {
                o.insert(
                    csv::name_of(h).to_string(),
                    r.get(k).cloned().unwrap_or_default(),
                );
            }
            (o, line)
        })
        .collect()
}

fn get<'a>(r: &'a BTreeMap<String, String>, k: &str) -> &'a str {
    r.get(k).map_or("", String::as_str)
}

fn column(t: &Parsed, name: &str) -> Option<usize> {
    t.head.iter().position(|h| csv::name_of(h) == name)
}

/// A number from a cell, as the page reads one: blank, or not a finite
/// number, is none.
fn num(cell: &str) -> Option<f64> {
    let s = cell.trim();
    if s.is_empty() {
        return None;
    }
    s.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// The page's id: lower case letters, digits and underscores, from a letter.
fn is_id(s: &str) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| f.is_ascii_lowercase())
        && c.all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || x == '_')
}

fn first_line(s: &str) -> &str {
    s.trim().split('\n').next().unwrap_or("")
}

fn is_word(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Whether `name` appears in `code` as `\bname\b` does.
fn mentions(code: &str, name: &str) -> bool {
    let (first, last) = (name.chars().next(), name.chars().last());
    code.match_indices(name).any(|(i, _)| {
        let before = code[..i].chars().last();
        let after = code[i + name.len()..].chars().next();
        is_word(before) != is_word(first) && is_word(last) != is_word(after)
    })
}

/// The `## ` sections of a Markdown text, outside fenced code: each title, its
/// body and the line it starts on.
fn sections(text: &str) -> Vec<(String, String, usize)> {
    let mut out: Vec<(String, String, usize)> = Vec::new();
    let mut fence = false;
    for (n, l) in text.replace('\r', "").split('\n').enumerate() {
        let lead = l.trim_start();
        if lead.starts_with("```") || lead.starts_with("~~~") {
            fence = !fence;
        }
        if !fence {
            if let Some(rest) = l.strip_prefix("##") {
                if rest.starts_with(|c: char| c.is_whitespace()) {
                    let t = rest.trim().trim_end_matches('#').trim();
                    if !t.is_empty() {
                        out.push((t.to_string(), String::new(), n + 1));
                        continue;
                    }
                }
            }
        }
        if let Some(cur) = out.last_mut() {
            cur.1.push_str(l);
            cur.1.push('\n');
        }
    }
    out
}

/// Remove every `open … close` span, shortest first, as a lazy pattern does.
fn strip(text: &str, open: &str, close: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find(open) {
        match rest[i + open.len()..].find(close) {
            Some(j) => {
                out.push_str(&rest[..i]);
                out.push(' ');
                rest = &rest[i + open.len() + j + close.len()..];
            }
            None => break,
        }
    }
    out.push_str(rest);
    out
}

/// The long sentences of a text, made plain, as `web/js/md.js` gives them:
/// what two texts must not both say.
fn long_sentences(text: &str) -> Vec<String> {
    let t = strip(&strip(text, "```", "```"), "$$", "$$");
    // {{…}} with nothing but non-braces inside.
    let mut no_embeds = String::new();
    let mut rest = t.as_str();
    while let Some(i) = rest.find("{{") {
        let after = &rest[i + 2..];
        match after.find('}') {
            Some(j) if after[j..].starts_with("}}") => {
                no_embeds.push_str(&rest[..i]);
                no_embeds.push(' ');
                rest = &after[j + 2..];
            }
            _ => {
                no_embeds.push_str(&rest[..i + 2]);
                rest = after;
            }
        }
    }
    no_embeds.push_str(rest);
    // Split after . ! ? and whitespace, or at a blank line.
    let chars: Vec<char> = no_embeds.chars().collect();
    let mut pieces = Vec::new();
    let (mut start, mut i) = (0, 0);
    while i < chars.len() {
        if i > 0 && matches!(chars[i - 1], '.' | '!' | '?') && chars[i].is_whitespace() {
            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            pieces.push(chars[start..i].iter().collect::<String>());
            start = j;
            i = j;
            continue;
        }
        if chars[i] == '\n' {
            let mut j = i + 1;
            let mut last_nl = None;
            while j < chars.len() && chars[j].is_whitespace() {
                if chars[j] == '\n' {
                    last_nl = Some(j);
                }
                j += 1;
            }
            if let Some(k) = last_nl {
                pieces.push(chars[start..i].iter().collect::<String>());
                start = k + 1;
                i = k + 1;
                continue;
            }
        }
        i += 1;
    }
    pieces.push(chars[start..].iter().collect::<String>());
    pieces
        .into_iter()
        .map(|s| {
            let lower = s.to_lowercase();
            let mut plain = String::new();
            let mut gap = false;
            for c in lower.chars() {
                if c.is_ascii_lowercase() || c.is_ascii_digit() || c == ' ' {
                    if gap {
                        plain.push(' ');
                        gap = false;
                    }
                    plain.push(c);
                } else {
                    gap = true;
                }
            }
            if gap {
                plain.push(' ');
            }
            plain.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .filter(|s| s.split(' ').count() >= 8)
        .collect()
}

/// Every `{{name arg}}` on a line, as the page finds them.
fn embeds(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(i) = line[at..].find("{{") {
        let s = at + i + 2;
        let rest = &line[s..];
        let body = rest.trim_start();
        let name: String = body
            .chars()
            .take_while(|c| c.is_ascii_lowercase())
            .collect();
        let after_name = &body[name.len()..];
        let matched = (|| {
            if name.is_empty() || !after_name.starts_with(|c: char| c.is_whitespace()) {
                return None;
            }
            let arg_start = after_name.trim_start();
            let close = arg_start.find('}')?;
            if !arg_start[close..].starts_with("}}") {
                return None;
            }
            let arg = arg_start[..close].trim_end().to_string();
            let consumed = line.len() - arg_start.len() + close + 2;
            Some((arg, consumed))
        })();
        match matched {
            Some((arg, end)) => {
                out.push((name, arg));
                at = end;
            }
            None => at = s - 1,
        }
    }
    out
}

/// The input and answer columns of a results table, with their units, and
/// where its other columns are.
struct Columns {
    inputs: Vec<(usize, String, String)>,
    answers: Vec<(usize, String, String)>,
    other: BTreeMap<String, usize>,
}

fn result_columns(t: &Parsed) -> Columns {
    let mut c = Columns {
        inputs: Vec::new(),
        answers: Vec::new(),
        other: BTreeMap::new(),
    };
    for (k, h) in t.head.iter().enumerate() {
        let (name, unit) = csv::split_unit(h);
        if name == "answer" || name.starts_with("answer.") || name.starts_with("answer ") {
            c.answers.push((k, name.to_string(), unit.to_string()));
        } else if ["tolerance", "refuses", "origin", "note", "label", "says"].contains(&name) {
            c.other.insert(name.to_string(), k);
        } else {
            c.inputs.push((k, name.to_string(), unit.to_string()));
        }
    }
    c
}

const GROUP_CSV: &[&str] = &[
    "group.csv",
    "members.csv",
    "nodes.csv",
    "publishes.csv",
    "requirements.csv",
    "loops.csv",
    "constants.csv",
    "sources.csv",
    "versions.csv",
    "equations.csv",
    "symbols.csv",
    "figures.csv",
    "reviews.csv",
    "results/group.csv",
];
const NODE_CSV: &[&str] = &[
    "inputs.csv",
    "evidence.csv",
    "equations.csv",
    "symbols.csv",
    "figures.csv",
    "sources.csv",
    "results/isolation.csv",
    "declaration.csv",
];

struct Node {
    id: String,
    row: BTreeMap<String, String>,
    dir: String,
    files: BTreeMap<&'static str, Parsed>,
    text: BTreeMap<&'static str, String>,
    present: bool,
    inputs: Vec<Record>,
}

struct Group<'a> {
    folder: &'a Folder,
    files: Vec<(&'static str, Parsed)>,
    text: BTreeMap<&'static str, String>,
    nodes: Vec<Node>,
    stray: Vec<String>,
    edges: Vec<(String, String)>,
}

impl Group<'_> {
    fn has(&self, p: &str) -> bool {
        self.folder.contains_key(p)
    }
    fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }
    fn table(&self, p: &str) -> Vec<Record> {
        self.files
            .iter()
            .find(|(q, _)| *q == p)
            .map_or(Vec::new(), |(_, t)| records(t))
    }
    fn equation(&self, node: Option<&Node>, id: &str) -> bool {
        node.and_then(|n| n.files.get("equations.csv"))
            .is_some_and(|t| records(t).iter().any(|(r, _)| get(r, "id") == id))
            || self
                .table("equations.csv")
                .iter()
                .any(|(r, _)| get(r, "id") == id)
    }
    fn figure(&self, node: Option<&Node>, id: &str) -> bool {
        node.and_then(|n| n.files.get("figures.csv"))
            .is_some_and(|t| records(t).iter().any(|(r, _)| get(r, "id") == id))
            || self
                .table("figures.csv")
                .iter()
                .any(|(r, _)| get(r, "id") == id)
    }
}

fn read_text(f: &Folder, p: &str) -> Option<String> {
    f.get(p).map(|b| String::from_utf8_lossy(b).into_owned())
}

fn load(folder: &Folder) -> Group<'_> {
    let mut files = Vec::new();
    for p in GROUP_CSV {
        if let Some(t) = read_text(folder, p) {
            files.push((*p, csv::read(&t)));
        }
    }
    let mut text = BTreeMap::new();
    for p in ["explanation.md", "theory.md", "flow.txt"] {
        if let Some(t) = read_text(folder, p) {
            text.insert(p, t);
        }
    }
    let node_rows = files
        .iter()
        .find(|(p, _)| *p == "nodes.csv")
        .map_or(Vec::new(), |(_, t)| records(t));
    let mut nodes: Vec<Node> = Vec::new();
    for (row, _) in node_rows {
        let id = get(&row, "id").to_string();
        if id.is_empty() {
            continue;
        }
        let dir = format!("nodes/{id}/");
        let mut nf = BTreeMap::new();
        for p in NODE_CSV {
            if let Some(t) = read_text(folder, &format!("{dir}{p}")) {
                nf.insert(*p, csv::read(&t));
            }
        }
        let mut nt = BTreeMap::new();
        for p in [
            "explanation.md",
            "theory.md",
            "pseudocode.txt",
            "results/how-run.md",
        ] {
            if let Some(t) = read_text(folder, &format!("{dir}{p}")) {
                nt.insert(p, t);
            }
        }
        let inputs = nf.get("inputs.csv").map_or(Vec::new(), records);
        let n = Node {
            present: folder.keys().any(|k| k.starts_with(&dir)),
            id: id.clone(),
            row,
            dir,
            files: nf,
            text: nt,
            inputs,
        };
        // As a map keyed by id: a second row of the same id takes the first's place.
        match nodes.iter_mut().find(|x| x.id == id) {
            Some(slot) => *slot = n,
            None => nodes.push(n),
        }
    }
    let mut stray = Vec::new();
    for p in folder.keys().filter(|k| k.starts_with("nodes/")) {
        if let Some(id) = p.split('/').nth(1) {
            if !id.is_empty()
                && !nodes.iter().any(|n| n.id == id)
                && !stray.iter().any(|s: &String| s == id)
            {
                stray.push(id.to_string());
            }
        }
    }
    let mut edges = Vec::new();
    for n in &nodes {
        for (inp, _) in &n.inputs {
            let from = get(inp, "from").trim();
            if from.is_empty() || from == "case" {
                continue;
            }
            if nodes.iter().any(|x| x.id == from) {
                edges.push((from.to_string(), n.id.clone()));
            }
        }
    }
    Group {
        folder,
        files,
        text,
        nodes,
        stray,
        edges,
    }
}

struct Out(Vec<Finding>);

impl Out {
    fn add(&mut self, level: Level, place: &str, msg: impl Into<String>, line: usize) {
        self.0.push(Finding {
            level,
            place: place.to_string(),
            msg: msg.into(),
            line,
        });
    }
    fn error(&mut self, place: &str, msg: impl Into<String>, line: usize) {
        self.add(Level::Error, place, msg, line);
    }
    fn warning(&mut self, place: &str, msg: impl Into<String>, line: usize) {
        self.add(Level::Warning, place, msg, line);
    }
}

/// Every check of a group folder, against the pattern.
pub fn check(folder: &Folder, spec: &Spec) -> Vec<Finding> {
    let m = load(folder);
    let mut out = Out(Vec::new());
    let files = &spec.files;

    // ── files, by level ──
    for rule in files
        .iter()
        .filter(|r| r.level == "group" || r.level == "both")
    {
        if rule.path.ends_with('/') {
            continue;
        }
        if rule.required && !m.has(&rule.path) {
            out.error(&rule.path, format!("is missing — {}", rule.says), 0);
        }
    }
    for n in &m.nodes {
        if !n.present {
            out.error(
                &n.dir,
                format!("has no folder, though nodes.csv lists {}", n.id),
                0,
            );
            continue;
        }
        for rule in files.iter().filter(|r| r.level == "node") {
            if rule.path.ends_with('/') {
                continue;
            }
            if let Some(kinds) = &rule.kinds {
                if !kinds.iter().any(|k| k == get(&n.row, "kind")) {
                    continue;
                }
            }
            let p = format!("{}{}", n.dir, rule.path);
            if rule.required && !m.has(&p) {
                out.error(&p, format!("is missing — {}", first_line(&rule.says)), 0);
            }
        }
    }
    for id in &m.stray {
        out.warning(
            &format!("nodes/{id}/"),
            "is a folder nodes.csv does not list",
            0,
        );
    }

    // ── tables: columns and allowed values ──
    let rule_for = |path: &str, level: &str| {
        files
            .iter()
            .find(|r| r.path == path && (r.level == level || r.level == "both"))
    };
    let check_table = |out: &mut Out, path: &str, t: &Parsed, rule: Option<&FileRule>| {
        for (line, msg) in &t.problems {
            out.error(path, msg.clone(), *line);
        }
        let Some(rule) = rule else { return };
        let Some(cols) = &rule.columns else { return };
        for c in cols {
            if c.prefix {
                if !t.head.iter().any(|h| csv::name_of(h).starts_with(&c.name)) {
                    out.error(path, format!("has no {} column — {}", c.name, c.says), 1);
                }
                continue;
            }
            let Some(k) = column(t, &c.name) else {
                if !c.optional {
                    out.error(path, format!("has no column {} — {}", c.name, c.says), 1);
                }
                continue;
            };
            if let Some(one_of) = &c.one_of {
                for (i, r) in t.rows.iter().enumerate() {
                    if !r[k].is_empty() && !one_of.contains(&r[k]) {
                        out.error(
                            path,
                            format!(
                                "{} is \"{}\"; it must be one of {}",
                                c.name,
                                r[k],
                                one_of.join(", ")
                            ),
                            t.lines[i],
                        );
                    }
                }
            }
            // A refusal has no answer and so no tolerance: blank is right there.
            let ref_k = column(t, "refuses");
            if !c.optional && c.one_of.is_none() {
                for (i, r) in t.rows.iter().enumerate() {
                    if r[k].is_empty() && ref_k.is_none_or(|rk| r[rk] != "yes") {
                        out.warning(path, format!("{} is blank", c.name), t.lines[i]);
                    }
                }
            }
        }
        if rule.rows == "one" && t.rows.len() != 1 {
            out.error(
                path,
                format!("must have exactly one row; it has {}", t.rows.len()),
                0,
            );
        }
    };
    for (p, t) in &m.files {
        check_table(&mut out, p, t, rule_for(p, "group"));
    }
    for n in &m.nodes {
        for p in NODE_CSV {
            if let Some(t) = n.files.get(p) {
                check_table(&mut out, &format!("{}{p}", n.dir), t, rule_for(p, "node"));
            }
        }
    }

    // ── the group's identity and people ──
    let meta = m
        .table("group.csv")
        .into_iter()
        .next()
        .map(|(r, _)| r)
        .unwrap_or_default();
    let id = get(&meta, "id");
    if !id.is_empty() && !is_id(id) {
        out.error(
            "group.csv",
            format!("the id \"{id}\" is not lower case letters, digits and underscores"),
            0,
        );
    }
    let versions = m.table("versions.csv");
    let version = get(&meta, "version");
    if !version.is_empty()
        && !versions.is_empty()
        && !versions.iter().any(|(v, _)| get(v, "version") == version)
    {
        out.error(
            "versions.csv",
            format!("has no row for version {version} — say what this version changed"),
            0,
        );
    }
    let members = m.table("members.csv");
    if m.files.iter().any(|(p, _)| *p == "members.csv")
        && !members.iter().any(|(r, _)| get(r, "role") == "owner")
    {
        out.error(
            "members.csv",
            "names no owner — somebody must sign the whole group",
            0,
        );
    }
    for (mem, line) in &members {
        for nid in get(mem, "nodes").split_whitespace().filter(|x| *x != "*") {
            if m.node(nid).is_none() {
                out.error(
                    "members.csv",
                    format!(
                        "{} is the node engineer of \"{nid}\", which is not a node",
                        get(mem, "name")
                    ),
                    *line,
                );
            }
        }
    }

    // ── nodes.csv ──
    let mut seen = BTreeSet::new();
    for (r, line) in m.table("nodes.csv") {
        let rid = get(&r, "id");
        if !is_id(rid) {
            out.error(
                "nodes.csv",
                format!("the id \"{rid}\" is not lower case letters, digits and underscores"),
                line,
            );
        }
        if !seen.insert(rid.to_string()) {
            out.error("nodes.csv", format!("{rid} is listed twice"), line);
        }
        let q = get(&r, "question");
        if q.is_empty() || !q.trim_end().ends_with('?') {
            out.warning(
                "nodes.csv",
                format!("{rid}: the question should be a question, ending in ?"),
                line,
            );
        }
        if get(&r, "kind") == "declared" && num(get(&r, "value")).is_none() {
            out.error(
                "nodes.csv",
                format!("{rid} is declared, so it needs a value"),
                line,
            );
        }
        if let (Some(lo), Some(hi)) = (num(get(&r, "lower")), num(get(&r, "upper"))) {
            if lo > hi {
                out.error("nodes.csv", format!("{rid}: lower is above upper"), line);
            }
        }
    }

    // ── a transcription names its source and the person who checked it ──
    let assistants = files
        .iter()
        .find(|r| r.path == "declaration.csv")
        .map_or(Vec::new(), |r| r.assistants.clone());
    for n in &m.nodes {
        let Some(t) = n.files.get("declaration.csv") else {
            continue;
        };
        for (r, line) in records(t) {
            if get(&r, "ai") != "transcribed" {
                continue;
            }
            let place = format!("{}declaration.csv", n.dir);
            if get(&r, "source").trim().is_empty() {
                out.error(
                    &place,
                    "says transcribed but names no source — say what the relation was copied from",
                    line,
                );
            }
            let who = get(&r, "checked_by").trim();
            let lower = who.to_lowercase();
            if who.is_empty() {
                out.error(
                    &place,
                    "says transcribed but names nobody who checked the copy against its source",
                    line,
                );
            } else if assistants.iter().any(|a| {
                lower == *a
                    || lower.starts_with(&format!("{a} "))
                    || lower.contains(&format!("{a}/"))
            }) {
                out.error(
                    &place,
                    format!("\"{who}\" is an assistant's name; the copy is checked by a person"),
                    line,
                );
            }
        }
    }

    // ── the texts: headings, what each may contain, embeds ──
    let check_text =
        |out: &mut Out, path: &str, text: Option<&String>, role: &str, node: Option<&Node>| {
            let Some(text) = text else { return };
            let Some(rule) = spec.texts.iter().find(|t| t.role == role) else {
                return;
            };
            let s = sections(text);
            let titles: Vec<String> = s.iter().map(|x| x.0.to_lowercase()).collect();
            let mut last: Option<usize> = None;
            for h in &rule.headings {
                match titles.iter().position(|t| *t == h.to_lowercase()) {
                    None => out.error(path, format!("has no \"## {h}\" section"), 0),
                    Some(k) if last.is_some_and(|l| k < l) => {
                        out.warning(path, format!("\"## {h}\" is out of order"), 0)
                    }
                    Some(k) => {
                        last = Some(k);
                        if s[k].1.trim().is_empty() {
                            out.error(path, format!("says nothing under \"## {h}\""), s[k].2);
                        }
                    }
                }
            }
            for bad in &rule.forbid {
                if let Some(at) = text.split('\n').position(|l| l.contains(bad.as_str())) {
                    out.error(
                        path,
                        format!("contains {bad} — {}", rule.forbid_says),
                        at + 1,
                    );
                }
            }
            for (i, l) in text.split('\n').enumerate() {
                for (name, arg) in embeds(l) {
                    match name.as_str() {
                        "eq" if !m.equation(node, &arg) => out.error(
                            path,
                            format!("{{{{eq {arg}}}}} names no equation in equations.csv"),
                            i + 1,
                        ),
                        "fig" if !m.figure(node, &arg) => out.error(
                            path,
                            format!("{{{{fig {arg}}}}} names no row of figures.csv"),
                            i + 1,
                        ),
                        "node" if m.node(&arg).is_none() => {
                            out.error(path, format!("{{{{node {arg}}}}} names no node"), i + 1)
                        }
                        "guess" if !arg.contains("||") => out.error(
                            path,
                            "{{guess …}} needs the question, then ||, then the answer",
                            i + 1,
                        ),
                        "eq" | "fig" | "node" | "guess" => {}
                        other => out.error(
                            path,
                            format!("{{{{{other}}}}} is not one of eq, fig, node, guess"),
                            i + 1,
                        ),
                    }
                }
            }
        };
    let check_overlap = |out: &mut Out, place: &str, a: Option<&String>, b: Option<&String>| {
        let (Some(a), Some(b)) = (a, b) else { return };
        if a.is_empty() || b.is_empty() {
            return;
        }
        let theory: BTreeSet<String> = long_sentences(b).into_iter().collect();
        for s in long_sentences(a)
            .into_iter()
            .filter(|s| theory.contains(s))
            .take(3)
        {
            out.warning(
                place,
                format!(
                    "says the same sentence in the explanation and the theory: \"{}…\"",
                    s.chars().take(80).collect::<String>()
                ),
                0,
            );
        }
    };
    check_text(
        &mut out,
        "explanation.md",
        m.text.get("explanation.md"),
        "explanation",
        None,
    );
    check_text(
        &mut out,
        "theory.md",
        m.text.get("theory.md"),
        "theory",
        None,
    );
    check_overlap(
        &mut out,
        "the group",
        m.text.get("explanation.md"),
        m.text.get("theory.md"),
    );
    for n in &m.nodes {
        check_text(
            &mut out,
            &format!("{}explanation.md", n.dir),
            n.text.get("explanation.md"),
            "explanation",
            Some(n),
        );
        check_text(
            &mut out,
            &format!("{}theory.md", n.dir),
            n.text.get("theory.md"),
            "theory",
            Some(n),
        );
        check_overlap(
            &mut out,
            &n.dir,
            n.text.get("explanation.md"),
            n.text.get("theory.md"),
        );
    }

    // ── each node's algorithm, inputs and results ──
    for n in &m.nodes {
        if get(&n.row, "kind") != "computed" || !n.present {
            continue;
        }
        if let Some(code) = n.text.get("pseudocode.txt").filter(|c| !c.is_empty()) {
            let place = format!("{}pseudocode.txt", n.dir);
            let returns = code.split('\n').any(|l| {
                let t = l.trim_start();
                t.strip_prefix("return")
                    .is_some_and(|rest| !is_word(rest.chars().next()))
            });
            if !returns {
                out.error(
                    &place,
                    "never returns an answer — every path ends in return or refuse",
                    0,
                );
            }
            for (inp, _) in &n.inputs {
                let name = get(inp, "name");
                if !name.is_empty() && !mentions(code, name) {
                    out.warning(&place, format!("never uses the input {name}"), 0);
                }
            }
            // The language's own reading: every line parses and every unit
            // agrees. The members a node publishes beside its answer are its
            // results' answer.<member> columns.
            let unit = |u: &str| format!("[{}]", if u.trim().is_empty() { "1" } else { u.trim() });
            let mut plain = format!("output {}\n", unit(get(&n.row, "unit")));
            for (inp, _) in &n.inputs {
                plain.push_str(&format!(
                    "input {} {}\n",
                    get(inp, "name").trim(),
                    unit(get(inp, "unit"))
                ));
            }
            if let Some(t) = n.files.get("results/isolation.csv") {
                for (_, name, u) in result_columns(t).answers {
                    if let Some(member) = name.strip_prefix("answer.") {
                        plain.push_str(&format!("publish {} {}\n", member.trim(), unit(&u)));
                    }
                }
            }
            plain.push_str("method\n");
            plain.push_str(code);
            match vleo_sheet::method::report_plain(&plain) {
                Err(e) => out.error(&place, e.message().to_string(), 0),
                Ok(r) => {
                    for d in r.diags {
                        let level = if d.severity == vleo_sheet::method::Severity::Error {
                            Level::Error
                        } else {
                            Level::Note
                        };
                        out.add(level, &place, d.msg, d.line);
                    }
                }
            }
        }
        // Inputs: where each comes from, and its default in its range.
        for (inp, line) in &n.inputs {
            let place = format!("{}inputs.csv", n.dir);
            let name = get(inp, "name");
            let from = get(inp, "from").trim();
            let external = from
                .split_once('.')
                .is_some_and(|(a, b)| is_id(a) && is_id(b));
            if from.is_empty() {
                out.error(
                    &place,
                    format!("{name} does not say where it comes from"),
                    *line,
                );
            } else if from != "case" && m.node(from).is_none() && !external {
                out.error(
                    &place,
                    format!("{name} comes from \"{from}\", which is no node here; another group's node is written group.node"),
                    *line,
                );
            }
            let (d, lo, hi) = (
                num(get(inp, "default")),
                num(get(inp, "min")),
                num(get(inp, "max")),
            );
            match d {
                None => out.error(&place, format!("{name} has no default value"), *line),
                Some(d) => {
                    if lo.is_some_and(|lo| d < lo) {
                        out.error(&place, format!("{name}: the default is below min"), *line);
                    }
                    if hi.is_some_and(|hi| d > hi) {
                        out.error(&place, format!("{name}: the default is above max"), *line);
                    }
                }
            }
            if let Some(up) = m.node(from) {
                let (uu, iu) = (get(&up.row, "unit"), get(inp, "unit"));
                if !uu.is_empty() && !iu.is_empty() && uu != iu {
                    out.warning(
                        &place,
                        format!("{name} is in {iu} but {from} answers in {uu}"),
                        *line,
                    );
                }
            }
        }
        // The isolation results: the reference the developer's code must match.
        let Some(t) = n.files.get("results/isolation.csv") else {
            continue;
        };
        let place = format!("{}results/isolation.csv", n.dir);
        let cols = result_columns(t);
        for (inp, _) in &n.inputs {
            let (name, unit) = (get(inp, "name"), get(inp, "unit"));
            match cols.inputs.iter().find(|c| c.1 == name) {
                None => out.error(
                    &place,
                    format!(
                        "has no column for the input {name} [{}]",
                        if unit.is_empty() { "unit" } else { unit }
                    ),
                    0,
                ),
                Some(c) if !unit.is_empty() && c.2 != unit => out.error(
                    &place,
                    format!("{name} is in [{}] here and [{unit}] in inputs.csv", c.2),
                    0,
                ),
                Some(_) => {}
            }
        }
        for c in &cols.inputs {
            if !n.inputs.iter().any(|(i, _)| get(i, "name") == c.1) {
                out.warning(
                    &place,
                    format!("has a column {} that is not an input", c.1),
                    0,
                );
            }
        }
        let row_unit = get(&n.row, "unit");
        if cols.answers.len() == 1 && !row_unit.is_empty() && cols.answers[0].2 != row_unit {
            out.error(
                &place,
                format!(
                    "the answer is in [{}] here and [{row_unit}] in nodes.csv",
                    cols.answers[0].2
                ),
                0,
            );
        }
        let (ref_k, tol_k, ori_k) = (
            cols.other.get("refuses"),
            cols.other.get("tolerance"),
            cols.other.get("origin"),
        );
        let (mut answers, mut refusals, mut at_defaults) = (0, 0, false);
        let mut cover: BTreeMap<String, (bool, bool)> = BTreeMap::new();
        for (i, r) in t.rows.iter().enumerate() {
            let line = t.lines[i];
            let refuses = ref_k.is_some_and(|&k| r[k] == "yes");
            // A refusal may test a value that is not a finite number: the node must refuse that too.
            let odd = |v: &str| {
                let v = v.trim().to_lowercase();
                let v = v.strip_prefix(['+', '-']).unwrap_or(&v);
                refuses && matches!(v, "nan" | "inf" | "infinity")
            };
            for c in &cols.inputs {
                if num(&r[c.0]).is_none() && !odd(&r[c.0]) {
                    out.error(&place, format!("{} is not a number", c.1), line);
                }
            }
            if refuses {
                refusals += 1;
                if cols.answers.iter().any(|a| !r[a.0].is_empty()) {
                    out.warning(&place, "a refusal should leave the answer blank", line);
                }
            } else {
                answers += 1;
                for a in &cols.answers {
                    if num(&r[a.0]).is_none() {
                        out.error(&place, format!("{} is not a number", a.1), line);
                    }
                }
                if let Some(&k) = tol_k {
                    if !num(&r[k]).is_some_and(|x| x > 0.0) {
                        out.error(&place, "the tolerance must be a number above zero", line);
                    }
                }
            }
            if !n.inputs.is_empty()
                && n.inputs.iter().all(|(inp, _)| {
                    cols.inputs
                        .iter()
                        .find(|c| c.1 == get(inp, "name"))
                        .is_some_and(|c| {
                            num(&r[c.0]).is_some() && num(&r[c.0]) == num(get(inp, "default"))
                        })
                })
            {
                at_defaults = true;
            }
            for (inp, _) in &n.inputs {
                let name = get(inp, "name");
                let Some(c) = cols.inputs.iter().find(|c| c.1 == name) else {
                    continue;
                };
                let v = num(&r[c.0]);
                let e = cover.entry(name.to_string()).or_default();
                if v.is_some() && num(get(inp, "min")).is_some() && v == num(get(inp, "min")) {
                    e.0 = true;
                }
                if v.is_some() && num(get(inp, "max")).is_some() && v == num(get(inp, "max")) {
                    e.1 = true;
                }
            }
        }
        if !at_defaults && !n.inputs.is_empty() {
            out.error(&place, "has no row with every input at its default", 0);
        }
        if answers < 3 {
            out.error(
                &place,
                format!("has {answers} ordinary case(s); it needs at least 3"),
                0,
            );
        }
        if refusals < 1 {
            out.warning(
                &place,
                "has no refusal — a row where the node must refuse, with refuses = yes",
                0,
            );
        }
        for (inp, _) in &n.inputs {
            let name = get(inp, "name");
            let c = cover.get(name).copied().unwrap_or_default();
            if num(get(inp, "min")).is_some() && !c.0 {
                out.warning(
                    &place,
                    format!("never tests {name} at its min, {}", get(inp, "min")),
                    0,
                );
            }
            if num(get(inp, "max")).is_some() && !c.1 {
                out.warning(
                    &place,
                    format!("never tests {name} at its max, {}", get(inp, "max")),
                    0,
                );
            }
        }
        if let Some(&k) = ori_k {
            let how = n
                .text
                .get("results/how-run.md")
                .is_some_and(|t| !t.is_empty());
            if t.rows.iter().any(|r| r[k] == "code") && !how {
                out.error(
                    &format!("{}results/how-run.md", n.dir),
                    "is missing — these results came from code, so say how it was run",
                    0,
                );
            }
        }
    }

    // ── the group's own results: the developer's group test ──
    if let Some((_, gt)) = m.files.iter().find(|(p, _)| *p == "results/group.csv") {
        let place = "results/group.csv";
        let cols = result_columns(gt);
        let unit_of = |id: &str| {
            m.node(id)
                .map(|n| get(&n.row, "unit").to_string())
                .unwrap_or_default()
        };
        for c in &cols.inputs {
            match m.node(&c.1) {
                None => out.error(
                    place,
                    format!("the column {} is no node of the group; an input column is named by its node id", c.1),
                    0,
                ),
                Some(nd) if get(&nd.row, "kind") != "declared" => out.warning(
                    place,
                    format!("{} is {}, not declared: a group test sets what the group is given", c.1, get(&nd.row, "kind")),
                    0,
                ),
                Some(_) => {
                    let u = unit_of(&c.1);
                    if !u.is_empty() && c.2 != u {
                        out.error(place, format!("{} is in [{}] here and [{u}] in nodes.csv", c.1, c.2), 0);
                    }
                }
            }
        }
        if cols.answers.is_empty() {
            out.error(
                place,
                "has no answer column — write answer.<node id> [unit] for each node the test reads",
                0,
            );
        }
        for a in &cols.answers {
            let id = a.1.strip_prefix("answer").unwrap_or(&a.1);
            let id = id.strip_prefix('.').unwrap_or(id).trim_start();
            match m.node(id) {
                None => out.error(
                    place,
                    format!(
                        "{} names no node: an answer column is answer.<node id>",
                        a.1
                    ),
                    0,
                ),
                Some(_) => {
                    let u = unit_of(id);
                    if !u.is_empty() && a.2 != u {
                        out.error(
                            place,
                            format!("{} is in [{}] here and [{u}] in nodes.csv", a.1, a.2),
                            0,
                        );
                    }
                }
            }
        }
        let (ref_k, tol_k) = (cols.other.get("refuses"), cols.other.get("tolerance"));
        for (i, r) in gt.rows.iter().enumerate() {
            let line = gt.lines[i];
            let refuses = ref_k.is_some_and(|&k| r[k] == "yes");
            for c in &cols.inputs {
                if num(&r[c.0]).is_none() {
                    out.error(place, format!("{} is not a number", c.1), line);
                }
            }
            if !refuses {
                for a in &cols.answers {
                    if num(&r[a.0]).is_none() {
                        out.error(place, format!("{} is not a number", a.1), line);
                    }
                }
                if !tol_k.is_some_and(|&k| num(&r[k]).is_some_and(|x| x > 0.0)) {
                    out.error(place, "the tolerance must be a number above zero", line);
                }
            }
        }
    }

    // ── requirements, loops, publishes, flow, sources, figures ──
    for (r, line) in m.table("requirements.csv") {
        for side in ["required", "achieved"] {
            let id = get(&r, side);
            match m.node(id) {
                None => out.error(
                    "requirements.csv",
                    format!("{}: the {side} node \"{id}\" does not exist", get(&r, "id")),
                    line,
                ),
                Some(nd) if get(&nd.row, "kind") != side => out.warning(
                    "requirements.csv",
                    format!(
                        "{}: {id} is {}, not {side}",
                        get(&r, "id"),
                        get(&nd.row, "kind")
                    ),
                    line,
                ),
                Some(_) => {}
            }
        }
    }
    for (l, line) in m.table("loops.csv") {
        for id in get(&l, "nodes").split_whitespace() {
            if m.node(id).is_none() {
                out.error(
                    "loops.csv",
                    format!("the loop names \"{id}\", which is not a node"),
                    line,
                );
            }
        }
    }
    for (p, line) in m.table("publishes.csv") {
        if m.node(get(&p, "node")).is_none() {
            out.error(
                "publishes.csv",
                format!("\"{}\" is not a node", get(&p, "node")),
                line,
            );
        }
    }
    if let Some(flow) = m.text.get("flow.txt").filter(|f| !f.is_empty()) {
        let mut declared = BTreeSet::new();
        for (i, l) in flow.split('\n').enumerate() {
            let s = l.split('#').next().unwrap_or("").trim();
            if s.is_empty() {
                continue;
            }
            let step = s.split_once("<-").and_then(|(a, b)| {
                let a = a.trim_end();
                is_id(a).then(|| (a.to_string(), b.trim_start().to_string()))
            });
            let Some((to, froms)) = step else {
                out.warning(
                    "flow.txt",
                    "a step is written `node <- input, input`",
                    i + 1,
                );
                continue;
            };
            if m.node(&to).is_none() {
                out.error("flow.txt", format!("\"{to}\" is not a node"), i + 1);
            }
            for from in froms
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|x| !x.is_empty())
            {
                declared.insert(format!("{from}>{to}"));
                if m.node(from).is_none() && from != "case" && !from.contains('.') {
                    out.error("flow.txt", format!("\"{from}\" is not a node"), i + 1);
                }
            }
        }
        for (from, to) in &m.edges {
            if !declared.contains(&format!("{from}>{to}")) {
                out.warning(
                    "flow.txt",
                    format!(
                        "does not show {from} feeding {to}, which {to}/inputs.csv says it does"
                    ),
                    0,
                );
            }
        }
    }
    let mut source_ids: BTreeSet<String> = m
        .table("sources.csv")
        .iter()
        .map(|(s, _)| get(s, "id").to_string())
        .collect();
    for (s, line) in m.table("sources.csv") {
        let file = get(&s, "file");
        if !file.is_empty() && !m.has(file) && !m.has(&format!("sources/{file}")) {
            out.error(
                "sources.csv",
                format!("{}: the file {file} is not in the folder", get(&s, "id")),
                line,
            );
        }
    }
    // A node's own sources count for the whole group: a paper cited once is cited.
    for n in &m.nodes {
        for (s, line) in n.files.get("sources.csv").map_or(Vec::new(), records) {
            let file = get(&s, "file");
            if !file.is_empty()
                && !m.has(&format!("{}{file}", n.dir))
                && !m.has(&format!("{}sources/{file}", n.dir))
            {
                out.error(
                    &format!("{}sources.csv", n.dir),
                    format!("{}: the file {file} is not in the folder", get(&s, "id")),
                    line,
                );
            }
            if !get(&s, "id").is_empty() {
                source_ids.insert(get(&s, "id").to_string());
            }
        }
    }
    let cite = |out: &mut Out, place: &str, rows: Vec<Record>| {
        for (r, line) in rows {
            let src = get(&r, "source");
            if !src.is_empty() && !source_ids.contains(src) {
                out.error(
                    place,
                    format!("cites \"{src}\", which sources.csv does not list"),
                    line,
                );
            }
        }
    };
    cite(&mut out, "equations.csv", m.table("equations.csv"));
    cite(&mut out, "constants.csv", m.table("constants.csv"));
    for n in &m.nodes {
        cite(
            &mut out,
            &format!("{}equations.csv", n.dir),
            n.files.get("equations.csv").map_or(Vec::new(), records),
        );
        cite(
            &mut out,
            &format!("{}evidence.csv", n.dir),
            n.files.get("evidence.csv").map_or(Vec::new(), records),
        );
    }
    let check_figures = |out: &mut Out, rows: Vec<Record>, base: &str, place: &str| {
        for (fg, line) in &rows {
            let (id, file, kind) = (get(fg, "id"), get(fg, "file"), get(fg, "kind"));
            let path = format!("{base}{file}");
            if !m.has(&path) {
                out.error(place, format!("{id}: {file} is not in the folder"), *line);
                continue;
            }
            if kind == "image" || kind == "video" {
                continue;
            }
            let t = csv::read(&read_text(m.folder, &path).unwrap_or_default());
            let need: &[&str] = match kind {
                "line" | "scatter" | "bar" => &["x", "y"],
                "heatmap" | "animation" => &["x", "y", "z"],
                _ => &[],
            };
            for key in need {
                for col in get(fg, key).split_whitespace() {
                    if column(&t, col).is_none() {
                        out.error(place, format!("{id}: {file} has no column {col}"), *line);
                    }
                }
            }
            for key in need {
                if get(fg, key).is_empty() {
                    out.error(
                        place,
                        format!("{id}: a {kind} needs the column {key}"),
                        *line,
                    );
                }
            }
            if kind == "flow" && (column(&t, "from").is_none() || column(&t, "to").is_none()) {
                out.error(
                    place,
                    format!("{id}: a flow is a table with from and to"),
                    *line,
                );
            }
            if kind == "steps" && (column(&t, "step").is_none() || column(&t, "caption").is_none())
            {
                out.error(
                    place,
                    format!("{id}: steps are a table with step and caption"),
                    *line,
                );
            }
            let on = get(fg, "on");
            if kind == "steps"
                && !on.is_empty()
                && !m.figure(None, on)
                && !rows.iter().any(|(r, _)| get(r, "id") == on)
            {
                out.error(
                    place,
                    format!("{id}: walks through \"{on}\", which is not a figure"),
                    *line,
                );
            }
            if kind == "scene3d"
                && ["body", "x", "y", "z"]
                    .iter()
                    .any(|c| column(&t, c).is_none())
            {
                out.error(
                    place,
                    format!("{id}: a 3D scene is a table with body, x, y, z"),
                    *line,
                );
            }
        }
    };
    check_figures(&mut out, m.table("figures.csv"), "", "figures.csv");
    for n in &m.nodes {
        check_figures(
            &mut out,
            n.files.get("figures.csv").map_or(Vec::new(), records),
            &n.dir,
            &format!("{}figures.csv", n.dir),
        );
    }

    // ── what the folder may hold ──
    for (p, bytes) in m.folder {
        let ext = p.rsplit('.').next().unwrap_or("").to_lowercase();
        if let Some(max) = spec.file_bytes {
            if bytes.len() as f64 > max {
                out.error(
                    p,
                    format!(
                        "is {:.1} MB — {}",
                        bytes.len() as f64 / 1_048_576.0,
                        spec.file_bytes_says
                    ),
                    0,
                );
            }
        }
        if spec.refused_types.contains(&ext) {
            out.error(p, format!("is a .{ext} — {}", spec.refused_says), 0);
        }
    }

    let mut found = out.0;
    found.sort_by(|a, b| (a.level, &a.place, a.line).cmp(&(b.level, &b.place, b.line)));
    found
}
