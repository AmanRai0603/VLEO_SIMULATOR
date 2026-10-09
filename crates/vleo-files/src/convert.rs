//! The design's files, read as the folders they were converted from.
//!
//! docs/PLAN_1_0.md, phase E. The design was node sheets and layer files in
//! the repository, converted once to files in the one schema (`schema.sql`):
//! the programme's branch, the systems branch and each subsystem group's,
//! each a group file (`<group>.vgroup`) holding its headings, mounts and
//! loops, with one node file (`<node>.vnode`) per row; and each case, a case
//! file (`.vcase`). The files are the design now, in `design/`, and the sheets
//! are gone. This reads them: [`open`] is the one reader every face, command
//! and test reads the design through.
//!
//! What maps to what, the sheet's form on the left:
//!
//! | today | in the files |
//! |---|---|
//! | a heading (`layers/<id>.toml`) | a `block` of its branch's group file, its behaviour its children |
//! | a row (`node.toml`) | a node file: its `block`, behaviour as the sheet makes it; its layer its perspective |
//! | `[output]`, `[[publishes]]` | `port`s out: symbol, unit, both ends and their reasons, state, maturity, the level that owns a parameter, while open its owner and gate |
//! | `[[input]]` | a `port` in, and the `wire` into it from the port it reads: `<block>.<port>` in its own group, `<group>.<block>.<port>` in another |
//! | `[value]` | the output's value |
//! | `[method]`, `[explain]` | `text`: `method`, `explain.simply`, `explain.breaks`, `explain.wrong` |
//! | `fixtures.toml` | `test_case`s, each with its provenance |
//! | `parity.csv` | a `tbl` |
//! | a group's crossing row | the `mount` its group hangs from, in the branch above |
//! | `[[iterate]]` | a `loop`, on the smallest block that holds every row it runs through |
//! | `cases/`, `sources/` | kept as they are, by their path: a case file each, and a text of the systems file |
//!
//! What the schema has no column for yet stays the sheet's own words, as
//! TOML, in a `text` of kind `sheet` (`heading`, `fixtures`, `loop`): a row's
//! label, its theory, its versions, its author's cases. It is the part a
//! later screen gives its own place, read and written by nothing else
//! meanwhile. And every file as it stood, comments and all, is kept beside
//! what it became, as a `text` of kind `as converted: <path>`, read by
//! nothing: the record of what the conversion was given.
//!
//! [`Served`] serves the files as the folders they were converted from,
//! which the loader reads as it read a checkout. What the conversion changed
//! in the design, each by name, stays as it made it:
//!
//! 1. each subsystem group hangs from the block it mounts on, not from the
//!    root (docs/SYSTEM_MODEL.md, section 8, "Where each group mounts");
//! 2. the architecture's loop is declared on the smallest block that holds
//!    it (section 5);
//! 3. every stated value is a parameter of the level whose branch states it
//!    (section 6, "A parameter belongs to the level that decides it");
//! 4. the five blocks the breakdown did not hold are there, as open blocks,
//!    each with why ([`PROPOSED`]; section 8, "Proposed: one level deeper").

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

use toml::value::Table as TomlTable;
use toml::Value;
use vleo_sheet::files::Files;

use crate::error::{Error, ErrorKind};
use crate::meta::Kind;
use crate::model::{File, Port, TestCase};

/// A group file's extension: the branch, its headings, mounts and loops.
pub const GROUP_FILE: &str = "vgroup";
/// A node file's extension: one row, all six parts.
pub const NODE_FILE: &str = "vnode";
/// A case file's extension: the inputs for a run.
pub const CASE_FILE: &str = "vcase";

/// The programme's branch: the management perspective, from the root.
pub const PROGRAMME: &str = "programme";
/// The systems branch: the system perspective.
pub const SYSTEMS: &str = "systems";

/// The scope a text or table of the whole file has.
const FILE: &str = "file";
/// The text a row, heading, fixture list or loop keeps its own words in.
const SHEET: &str = "sheet";
const HEADING: &str = "heading";
const FIXTURES: &str = "fixtures";
const LOOP: &str = "loop";
/// A text kept as it was, by its path.
const KEPT: &str = "as converted: ";
/// The keys a converted sheet's own words add, which the inverse takes out.
const CONVERTED: &str = "converted";

/// The blocks a group's breakdown does not hold yet, proposed when the design
/// is converted, each with why: the next level `docs/SYSTEM_MODEL.md`
/// (section 8, "Proposed: one level deeper") proposes, less every one its
/// owners' rows already cover — station-keeping delta-v is the drag make-up,
/// the deorbit delta-v the disposal, the relative performance error the
/// pointing stability, the radiator, heater and insulation rows the thermal
/// hardware, and so on. Each is an open block, for its owner to confirm,
/// rename, break down or remove; none computes. The breakdown itself is its
/// owners', and is not moved.
pub const PROPOSED: &[(&str, &[(&str, &str)])] = &[
    (
        "l3_x_envorbit",
        &[(
            "Radiation environment",
            "Proposed when the design was converted: the group states the atmosphere, \
             its atomic oxygen and the geomagnetic field, and nothing of the radiation \
             the spacecraft receives (trapped protons and electrons, solar energetic \
             particles, the dose behind its shielding), which the space environment \
             standard treats beside them (ECSS-E-ST-10-04C). Open until its owner \
             confirms it or removes it.",
        )],
    ),
    (
        "l3_fsw",
        &[
            (
                "Modes and autonomy",
                "Proposed when the design was converted: the group sizes the computer \
                 (processor throughput, memory, clock) and states nothing of the modes the \
                 spacecraft runs in, or of what it decides on board between passes, which \
                 a flight software breakdown usually holds beside the computer \
                 (ECSS-E-ST-70-11C, space segment operability). Open until its owner \
                 confirms it or removes it.",
            ),
            (
                "Fault detection and recovery",
                "Proposed when the design was converted: nothing in the group says how a \
                 fault is detected, isolated and recovered from on board, which a flight \
                 software breakdown usually holds as its own branch (ECSS-E-ST-70-11C). \
                 Open until its owner confirms it or removes it.",
            ),
        ],
    ),
    (
        "l3_struct",
        &[
            (
                "Mechanisms and deployments",
                "Proposed when the design was converted: the group states the primary \
                 structure (its material, panels and dimensions) and nothing of the \
                 mechanisms that deploy or hold anything, the array, the antennas, the \
                 hold-down and release, which a structure breakdown usually holds as its \
                 own branch (ECSS-E-ST-33-01C). Open until its owner confirms it or \
                 removes it.",
            ),
            (
                "Launch adapter interface",
                "Proposed when the design was converted: nothing in the group states the \
                 interface to the launcher's adapter (its ring, its separation system, the \
                 loads it carries), usually its own item at this level. Open until its \
                 owner confirms it or removes it.",
            ),
        ],
    ),
];

/// The id a proposed block is given: its group's, then its name.
pub fn proposed_id(group: &str, name: &str) -> String {
    let mut slug = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.ends_with('_') {
            slug.push('_');
        }
    }
    format!("{group}_{}", slug.trim_matches('_'))
}

fn malformed(why: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, why)
}

#[cfg(not(target_arch = "wasm32"))]
fn io_error(path: &Path, e: io::Error) -> Error {
    Error::new(ErrorKind::Io, format!("{}: {e}", path.display()))
}

/// The perspective a layer is (docs/SYSTEM_MODEL.md, section 2: depth and
/// perspective are different things, and the layer was both).
pub fn perspective(layer: u8) -> Result<&'static str, Error> {
    match layer {
        0 => Ok(""),
        1 => Ok("management"),
        2 => Ok("system"),
        3 => Ok("subsystem"),
        l => Err(malformed(format!("layer {l} is no perspective"))),
    }
}

fn layer_of(perspective: &str) -> Result<i64, Error> {
    match perspective {
        "" => Ok(0),
        "management" => Ok(1),
        "system" => Ok(2),
        "subsystem" => Ok(3),
        p => Err(malformed(format!("{p:?} is no perspective"))),
    }
}

fn number_of(text: &str, what: &str) -> Result<Value, Error> {
    text.parse::<f64>()
        .map(Value::Float)
        .map_err(|_| malformed(format!("{what} is {text:?}, not a number")))
}

fn reasons_of(text: &str) -> Result<(String, String), Error> {
    if text.is_empty() {
        return Ok((String::new(), String::new()));
    }
    let rest = text
        .strip_prefix("lower: ")
        .ok_or_else(|| malformed(format!("a range's reason {text:?} does not start lower:")))?;
    let (lower, upper) = rest
        .split_once("\nupper: ")
        .ok_or_else(|| malformed(format!("a range's reason {text:?} has no upper:")))?;
    Ok((lower.into(), upper.into()))
}

fn inputs_of(text: &str, what: &str) -> Result<TomlTable, Error> {
    let mut t = TomlTable::new();
    for part in text.split("; ").filter(|p| !p.is_empty()) {
        let (k, v) = part
            .split_once(" = ")
            .ok_or_else(|| malformed(format!("{what}: {part:?} is not name = value")))?;
        t.insert(k.into(), number_of(v, what)?);
    }
    Ok(t)
}

fn take_table(t: &mut TomlTable, key: &str, what: &str) -> Result<Option<TomlTable>, Error> {
    match t.remove(key) {
        None => Ok(None),
        Some(Value::Table(x)) => Ok(Some(x)),
        Some(other) => Err(malformed(format!("{what}: {key} is {other}, not a table"))),
    }
}

/// A table inside an element of an array of tables, written on its element as
/// an inline table, as the sheets write one: `inputs = { a = 1, b = 2 }` on
/// the `[[case]]` it belongs to, not a `[case.inputs]` header after it. The
/// two read alike; but what edits a sheet's text knows the one form, and a
/// case edited where its inputs stood under their own header came out with
/// the key twice.
fn inline_element_tables(text: &str) -> String {
    // The array whose element the lines so far are in: its last header.
    fn array_name(out: &[String]) -> String {
        out.iter()
            .rev()
            .find_map(|l| l.trim().strip_prefix("[[")?.strip_suffix("]]"))
            .unwrap_or_default()
            .to_string()
    }
    let mut out: Vec<String> = Vec::new();
    let mut array: Option<String> = None;
    let mut folding: Option<(String, Vec<String>)> = None;
    let flush = |out: &mut Vec<String>, f: &mut Option<(String, Vec<String>)>| {
        if let Some((key, pairs)) = f.take() {
            // A value over several lines cannot be one line: left as written.
            if pairs
                .iter()
                .any(|p| p.contains("\"\"\"") || p.contains("'''"))
            {
                out.push(format!("[{}.{key}]", array_name(out)));
                out.extend(pairs);
                out.push(String::new());
                return;
            }
            // On the element, before the blank line that ends it.
            let at = out
                .iter()
                .rposition(|l| !l.trim().is_empty())
                .map_or(out.len(), |i| i + 1);
            out.insert(at, format!("{key} = {{ {} }}", pairs.join(", ")));
        }
    };
    for line in text.lines() {
        let l = line.trim();
        if let Some(name) = l.strip_prefix("[[").and_then(|x| x.strip_suffix("]]")) {
            flush(&mut out, &mut folding);
            array = Some(name.to_string());
        } else if let Some(name) = l.strip_prefix('[').and_then(|x| x.strip_suffix(']')) {
            flush(&mut out, &mut folding);
            match array
                .as_deref()
                .and_then(|a| name.strip_prefix(&format!("{a}.")))
            {
                Some(key) if !key.contains('.') => {
                    folding = Some((key.to_string(), Vec::new()));
                    continue;
                }
                _ => array = None,
            }
        } else if let Some((_, pairs)) = folding.as_mut() {
            if !l.is_empty() {
                pairs.push(line.to_string());
            }
            continue;
        }
        out.push(line.to_string());
    }
    flush(&mut out, &mut folding);
    let mut text = out.join("\n");
    text.push('\n');
    text
}

fn toml_text(t: &TomlTable, what: &str) -> Result<String, Error> {
    toml::to_string(t).map_err(|e| malformed(format!("{what}: {e}")))
}

fn toml_table(text: &str, what: &str) -> Result<TomlTable, Error> {
    text.parse::<TomlTable>()
        .map_err(|e| malformed(format!("{what}: {e}")))
}

/// What a sheet's content makes its behaviour, read from the sheet as the
/// inverse writes it: the same reading as `Sheet::behaviour`, so a node file
/// whose block says one thing and whose content another is refused by name.
fn behaviour_of_sheet(t: &TomlTable) -> &'static str {
    let s = |k: &str| t.get(k).and_then(Value::as_str).unwrap_or("");
    let seeded = s("state") == "empty" || s("state").is_empty();
    let method = t
        .get("method")
        .and_then(|m| m.get("text"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if seeded {
        "open"
    } else if t.contains_key("lookup") {
        "lookup"
    } else if t.contains_key("children") {
        "children"
    } else if s("kind") == "declared" {
        "stated"
    } else if !method.trim().is_empty() {
        "method"
    } else {
        "built-in"
    }
}

/// A port out, written back into the keys it came from.
fn port_back(p: &Port, t: &mut TomlTable, write_state: bool) -> Result<(), Error> {
    let what = format!("{}.{}", p.block_uid, p.name);
    let mut put = |k: &str, v: &str| {
        if !v.is_empty() {
            t.insert(k.into(), Value::String(v.into()));
        }
    };
    put("symbol", &p.symbol);
    put("unit", &p.unit);
    let (rl, ru) = reasons_of(&p.range_reason)?;
    put("reason_lower", &rl);
    put("reason_upper", &ru);
    if write_state {
        put("state", &p.state);
    }
    put("maturity", &p.maturity);
    put("parameter", &p.parameter);
    put("open_owner", &p.open_owner);
    put("open_due", &p.open_due);
    if !p.lower.is_empty() {
        t.insert("lower".into(), number_of(&p.lower, &what)?);
    }
    if !p.upper.is_empty() {
        t.insert("upper".into(), number_of(&p.upper, &what)?);
    }
    Ok(())
}

/// The variable a wire reads, as a row reads it: the row's answer by its id,
/// what it publishes as `<row>.<id>`.
fn var_of(from_ref: &str, what: &str) -> Result<String, Error> {
    let mut parts = from_ref.rsplit('.');
    match (parts.next(), parts.next()) {
        (Some(port), Some(row)) if !port.is_empty() && !row.is_empty() => Ok(if port == row {
            row.to_string()
        } else {
            format!("{row}.{port}")
        }),
        _ => Err(malformed(format!(
            "{what} is wired from {from_ref:?}, which names no port"
        ))),
    }
}

/// The design's files, read as the folders they were converted from.
///
/// The loader reads the design through it as it reads a checkout: each node
/// folder's sheet, cases and parity grid, each layer file, the cases and the
/// sources, assembled from the files and from nothing else. A path the files
/// do not hold is not there: it never falls back to a folder on disk, so what
/// is read is the design and only the design.
pub struct Served {
    root: PathBuf,
    files: BTreeMap<PathBuf, Vec<u8>>,
    dirs: BTreeSet<PathBuf>,
}

fn text_in<'a>(f: &'a File, scope: &str, kind: &str) -> Option<&'a str> {
    f.texts
        .iter()
        .find(|t| t.scope == scope && t.kind == kind)
        .map(|t| t.body.as_str())
}

fn table_mut<'a>(t: &'a mut TomlTable, key: &str) -> Result<&'a mut TomlTable, Error> {
    match t
        .entry(key.to_string())
        .or_insert_with(|| Value::Table(TomlTable::new()))
    {
        Value::Table(x) => Ok(x),
        other => Err(malformed(format!("{key} is {other}, not a table"))),
    }
}

fn array_mut<'a>(t: &'a mut TomlTable, key: &str, len: usize) -> Result<&'a mut Vec<Value>, Error> {
    let a = match t
        .entry(key.to_string())
        .or_insert_with(|| Value::Array(Vec::new()))
    {
        Value::Array(a) => a,
        other => return Err(malformed(format!("{key} is {other}, not a list"))),
    };
    if a.len() > len {
        return Err(malformed(format!(
            "{key} keeps {} entries, and the file holds {len}",
            a.len()
        )));
    }
    a.resize_with(len, || Value::Table(TomlTable::new()));
    Ok(a)
}

fn entry_mut(v: &mut Value) -> Result<&mut TomlTable, Error> {
    match v {
        Value::Table(t) => Ok(t),
        other => Err(malformed(format!("{other} is not a table"))),
    }
}

/// A node file, as the sheet, cases and parity grid of its folder.
fn node_back(f: &File) -> Result<(String, Option<String>, Option<String>), Error> {
    let b = f
        .blocks
        .first()
        .ok_or_else(|| malformed("a node file holds no block"))?;
    let id = b.uid.as_str();
    let what = format!("{id}'s node file");
    let mut t = toml_table(
        text_in(f, id, SHEET).ok_or_else(|| malformed(format!("{what} keeps no sheet")))?,
        &what,
    )?;
    let derived: BTreeSet<String> = take_table(&mut t, CONVERTED, &what)?
        .and_then(|mut c| c.remove("derived"))
        .and_then(|d| d.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();

    // ── the block
    t.insert("id".into(), Value::String(b.id.clone()));
    t.insert("parent".into(), Value::String(b.parent_uid.clone()));
    t.insert("order".into(), Value::Integer(b.ord));
    t.insert("layer".into(), Value::Integer(layer_of(&b.perspective)?));
    if !b.question.is_empty() {
        table_mut(&mut t, "question")?.insert("text".into(), Value::String(b.question.clone()));
    }

    // ── its ports out
    let mut outs: Vec<&Port> = f
        .ports
        .iter()
        .filter(|p| p.block_uid == id && p.direction == "out")
        .collect();
    outs.sort_by_key(|p| p.ord);
    let (first, rest) = outs
        .split_first()
        .ok_or_else(|| malformed(format!("{what} has no port out")))?;
    if first.name != id {
        return Err(malformed(format!(
            "{what}: its answer is the port named for it, not {}",
            first.name
        )));
    }
    port_back(
        first,
        table_mut(&mut t, "output")?,
        !derived.contains("output.state"),
    )?;
    if !first.value.is_empty() {
        table_mut(&mut t, "value")?.insert("number".into(), number_of(&first.value, &what)?);
    }
    if !rest.is_empty() {
        let a = array_mut(&mut t, "publishes", rest.len())?;
        for (k, (p, e)) in rest.iter().zip(a.iter_mut()).enumerate() {
            let e = entry_mut(e)?;
            e.insert("id".into(), Value::String(p.name.clone()));
            if !p.says.is_empty() {
                e.insert("label".into(), Value::String(p.says.clone()));
            }
            port_back(p, e, !derived.contains(&format!("publishes.{k}.state")))?;
        }
    }

    // ── its ports in
    let mut ins: Vec<&Port> = f
        .ports
        .iter()
        .filter(|p| p.block_uid == id && p.direction == "in")
        .collect();
    ins.sort_by_key(|p| p.ord);
    if !ins.is_empty() {
        let a = array_mut(&mut t, "input", ins.len())?;
        for (p, e) in ins.iter().zip(a.iter_mut()) {
            let w = f
                .wires
                .iter()
                .find(|w| w.to_block == id && w.to_port == p.name)
                .ok_or_else(|| malformed(format!("{what}: {} has no wire into it", p.name)))?;
            let e = entry_mut(e)?;
            e.insert("binding".into(), Value::String(p.name.clone()));
            e.insert(
                "var".into(),
                Value::String(var_of(&w.from_ref, &format!("{id}.{}", p.name))?),
            );
        }
    }

    // ── its method and its explanation
    if let Some(m) = text_in(f, id, "method") {
        table_mut(&mut t, "method")?.insert("text".into(), Value::String(m.into()));
    }
    for part in ["simply", "breaks", "wrong"] {
        if let Some(x) = text_in(f, id, &format!("explain.{part}")) {
            table_mut(&mut t, "explain")?.insert(part.into(), Value::String(x.into()));
        }
    }
    let says = behaviour_of_sheet(&t);
    if says != b.behaviour {
        return Err(malformed(format!(
            "{what}: its block says it is {}, and what it holds makes it {says}",
            b.behaviour
        )));
    }
    let sheet = inline_element_tables(&toml_text(&t, &what)?);

    // ── its cases
    let fixtures = match text_in(f, id, FIXTURES) {
        None if f.cases.is_empty() => None,
        None => {
            return Err(malformed(format!(
                "{what} holds cases and keeps no fixtures"
            )))
        }
        Some(text) => {
            let mut fx = toml_table(text, &what)?;
            let mut cases: Vec<(usize, &TestCase)> = f
                .cases
                .iter()
                .filter(|c| c.block_uid == id)
                .map(|c| {
                    let n = c
                        .name
                        .split_once(" · ")
                        .and_then(|(n, _)| n.parse::<usize>().ok())
                        .ok_or_else(|| {
                            malformed(format!("{what}: a case is named {:?}", c.name))
                        })?;
                    Ok((n, c))
                })
                .collect::<Result<_, Error>>()?;
            cases.sort_by_key(|(n, _)| *n);
            let a = array_mut(&mut fx, "fixture", cases.len())?;
            for ((_, c), e) in cases.iter().zip(a.iter_mut()) {
                let e = entry_mut(e)?;
                let label = c.name.split_once(" · ").map(|(_, l)| l).unwrap_or("");
                if !label.is_empty() {
                    e.insert("label".into(), Value::String(label.into()));
                }
                if !c.expected.is_empty() {
                    e.insert("expect".into(), number_of(&c.expected, &what)?);
                }
                if !c.tolerance.is_empty() {
                    e.insert("tolerance".into(), number_of(&c.tolerance, &what)?);
                }
                e.insert("provenance".into(), Value::String(c.provenance.clone()));
                if !c.source.is_empty() {
                    e.insert("source".into(), Value::String(c.source.clone()));
                }
                if !c.inputs.is_empty() {
                    e.insert("inputs".into(), Value::Table(inputs_of(&c.inputs, &what)?));
                }
            }
            Some(toml_text(&fx, &what)?)
        }
    };
    let parity = f
        .tables
        .iter()
        .find(|t| t.scope == id && t.path == "parity.csv")
        .map(|t| t.csv.clone());
    Ok((sheet, fixtures, parity))
}

/// A group file's headings, as their layer files.
fn headings_back(
    f: &File,
    mount_of: &BTreeMap<String, String>,
) -> Result<Vec<(String, String)>, Error> {
    let mut by_file: BTreeMap<String, TomlTable> = BTreeMap::new();
    for b in &f.blocks {
        let what = format!("{}'s heading", b.uid);
        let mut g = toml_table(
            text_in(f, &b.uid, HEADING)
                .ok_or_else(|| malformed(format!("{what} keeps nothing")))?,
            &what,
        )?;
        let stem = take_table(&mut g, CONVERTED, &what)?
            .and_then(|mut c| c.remove("file"))
            .and_then(|v| v.as_str().map(str::to_string))
            .ok_or_else(|| malformed(format!("{what} keeps no layer file")))?;
        let relates = g.remove("relates");
        g.insert("id".into(), Value::String(b.id.clone()));
        let parent = if b.parent_uid.is_empty() {
            mount_of.get(&b.uid).cloned().unwrap_or_default()
        } else {
            b.parent_uid.clone()
        };
        g.insert("parent".into(), Value::String(parent));
        g.insert("order".into(), Value::Integer(b.ord));
        g.insert("layer".into(), Value::Integer(layer_of(&b.perspective)?));
        let mut its = Vec::new();
        for l in f.loops.iter().filter(|l| l.block_uid == b.uid) {
            let mut it = toml_table(text_in(f, &l.uid, LOOP).unwrap_or(""), &l.uid)?;
            it.insert(
                "nodes".into(),
                Value::Array(
                    l.members
                        .split(',')
                        .filter(|m| !m.is_empty())
                        .map(|m| Value::String(m.into()))
                        .collect(),
                ),
            );
            it.insert("converge_on".into(), Value::String(l.settles.clone()));
            if !l.tolerance.is_empty() {
                it.insert("tolerance".into(), number_of(&l.tolerance, &l.uid)?);
            }
            it.insert("max_iter".into(), Value::Integer(l.max_iterations));
            its.push(Value::Table(it));
        }
        if !its.is_empty() {
            g.insert("iterate".into(), Value::Array(its));
        }
        let file = by_file.entry(stem).or_default();
        match file
            .entry("group".to_string())
            .or_insert_with(|| Value::Array(Vec::new()))
        {
            Value::Array(a) => a.push(Value::Table(g)),
            _ => unreachable!("a layer file's headings are a list"),
        }
        if let Some(r) = relates {
            file.insert("relates".into(), r);
        }
    }
    by_file
        .into_iter()
        .map(|(stem, t)| Ok((stem.clone(), toml_text(&t, &stem)?)))
        .collect()
}

impl Served {
    /// The files, served at `root` as the folders they were converted from.
    pub fn new(root: &Path, files: &[(String, File)]) -> Result<Served, Error> {
        let mut s = Served {
            root: root.to_path_buf(),
            files: BTreeMap::new(),
            dirs: BTreeSet::new(),
        };
        let mut mount_of: BTreeMap<String, String> = BTreeMap::new();
        for (_, f) in files {
            for m in &f.mounts {
                let top = files
                    .iter()
                    .filter(|(_, g)| g.meta.get("group_id") == Some(&m.group_id))
                    .filter(|(_, g)| {
                        g.meta.get("file_kind").map(String::as_str) == Some(Kind::Group.name())
                    })
                    .flat_map(|(_, g)| g.blocks.iter().filter(|b| b.parent_uid.is_empty()))
                    .map(|b| b.uid.clone())
                    .collect::<Vec<_>>();
                match top.as_slice() {
                    [one] => {
                        mount_of.insert(one.clone(), m.block_uid.clone());
                    }
                    other => {
                        return Err(malformed(format!(
                            "{} mounts on {} from {} top headings: {other:?}",
                            m.group_id,
                            m.block_uid,
                            other.len()
                        )))
                    }
                }
            }
        }
        for (path, f) in files {
            match f.kind()? {
                Kind::Node => {
                    let (sheet, fixtures, parity) = node_back(f)?;
                    let krate = f
                        .meta
                        .get("sheet.crate")
                        .ok_or_else(|| malformed(format!("{path} names no crate for its code")))?;
                    let id = &f.blocks[0].uid;
                    let dir = root.join("crates").join(krate).join("nodes").join(id);
                    s.put(dir.join("node.toml"), sheet.into_bytes());
                    if let Some(fx) = fixtures {
                        s.put(dir.join("fixtures.toml"), fx.into_bytes());
                    }
                    if let Some(pc) = parity {
                        s.put(dir.join("parity.csv"), pc.into_bytes());
                    }
                }
                Kind::Group => {
                    for (stem, text) in headings_back(f, &mount_of)? {
                        s.put(
                            root.join("layers").join(format!("{stem}.toml")),
                            text.into_bytes(),
                        );
                    }
                    s.kept(f);
                }
                Kind::Case => s.kept(f),
                other => {
                    return Err(malformed(format!(
                        "{path} is a {other}, and the design is group, node and case files"
                    )))
                }
            }
        }
        for d in [
            root.join("layers"),
            root.join("cases"),
            root.join("sources"),
        ] {
            s.dirs.insert(d);
        }
        Ok(s)
    }

    /// One path's bytes replaced, as an edit to that file in the folders
    /// would replace them, or the path taken away when `bytes` is `None`.
    /// For a test that shows what the loader refuses, on the design itself
    /// rather than on a copy of folders that are not the design.
    pub fn replace(&mut self, p: &Path, bytes: Option<Vec<u8>>) {
        match bytes {
            Some(b) => self.put(p.to_path_buf(), b),
            None => {
                self.files.remove(p);
            }
        }
    }

    /// What a file keeps by its path: a case, the sources.
    fn kept(&mut self, f: &File) {
        for t in &f.texts {
            if let Some(rel) = t.kind.strip_prefix(KEPT) {
                if t.scope == FILE && (rel.starts_with("cases/") || rel.starts_with("sources/")) {
                    self.put(self.root.join(rel), t.body.clone().into_bytes());
                }
            }
        }
        for t in &f.tables {
            if t.scope == FILE && t.path.starts_with("cases/") {
                self.put(self.root.join(&t.path), t.csv.clone().into_bytes());
            }
        }
    }

    fn put(&mut self, p: PathBuf, bytes: Vec<u8>) {
        let mut d = p.parent();
        while let Some(x) = d {
            if x == self.root || !self.dirs.insert(x.to_path_buf()) {
                break;
            }
            d = x.parent();
        }
        self.files.insert(p, bytes);
    }
}

impl Files for Served {
    fn read(&self, p: &Path) -> io::Result<Vec<u8>> {
        self.files.get(p).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not in the design's files", p.display()),
            )
        })
    }

    fn entries(&self, p: &Path) -> io::Result<Vec<PathBuf>> {
        let out: BTreeSet<PathBuf> = self
            .files
            .keys()
            .chain(self.dirs.iter())
            .filter(|k| k.parent() == Some(p))
            .cloned()
            .collect();
        if out.is_empty() && !self.is_dir(p) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not a folder", p.display()),
            ));
        }
        Ok(out.into_iter().collect())
    }

    fn is_dir(&self, p: &Path) -> bool {
        self.dirs.contains(p)
    }

    fn is_file(&self, p: &Path) -> bool {
        self.files.contains_key(p)
    }
}

/// The design under `root`, as everything in the repository reads it: the
/// files in `root/design/`, served as the folders they were converted from
/// and loaded by the one loader, with the fingerprint of the files. Nothing
/// else is read: a sheet left in a node folder is not the design.
#[cfg(not(target_arch = "wasm32"))]
pub fn open(root: &Path) -> Result<(vleo_sheet::Tree, String), Error> {
    let (served, fingerprint) = serve(root)?;
    let tree = vleo_sheet::load::load_all_from(&served, root).map_err(|e| {
        malformed(format!(
            "the design in {} does not load: {e}",
            root.join("design").display()
        ))
    })?;
    Ok((tree, fingerprint))
}

/// The files in `root/design/`, served at `root` as the folders they were
/// converted from, and their fingerprint: what [`open`] loads, for a reader
/// that reads the folders themselves.
#[cfg(not(target_arch = "wasm32"))]
pub fn serve(root: &Path) -> Result<(Served, String), Error> {
    let (files, fingerprint) = read_folder(&root.join("design"))?;
    Ok((Served::new(root, &files)?, fingerprint))
}

/// Every group, node and case file in `dir`, laid out as `design/` lays them
/// out, each with its path there, and the fingerprint of all of them: the
/// SHA-256 of every file's path and bytes, in order. Installed only: a page
/// has no folder, and is handed the files' rows instead (`crate::rows`).
#[cfg(not(target_arch = "wasm32"))]
pub fn read_folder(dir: &Path) -> Result<(Vec<(String, File)>, String), Error> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Error> {
        for e in std::fs::read_dir(dir).map_err(|e| io_error(dir, e))? {
            let p = e.map_err(|e| io_error(dir, e))?.path();
            if p.is_dir() {
                walk(&p, out)?;
            } else if p
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| [GROUP_FILE, NODE_FILE, CASE_FILE].contains(&x))
            {
                out.push(p);
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    walk(dir, &mut paths)?;
    paths.sort();
    if paths.is_empty() {
        return Err(malformed(format!(
            "{} holds no group, node or case file",
            dir.display()
        )));
    }
    let mut all = Vec::new();
    let mut files = Vec::new();
    for p in paths {
        let rel = p
            .strip_prefix(dir)
            .map_err(|_| malformed(format!("{} is outside {}", p.display(), dir.display())))?
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = std::fs::read(&p).map_err(|e| io_error(&p, e))?;
        all.extend_from_slice(rel.as_bytes());
        all.push(0);
        all.extend_from_slice(&crate::keys::sha256(&bytes));
        files.push((rel, crate::sqlite::read(&p)?));
    }
    Ok((files, crate::keys::hex(&crate::keys::sha256(&all))))
}
