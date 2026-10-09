//! The design, from its sheets to its files: the last time it passes through
//! the code.
//!
//! docs/PLAN_1_0.md, phase E. Until the switch-over the design is node sheets
//! and layer files in the repository; from it on, it is files on the shared
//! drive, in the one schema (`schema.sql`), written by their owners in the
//! application. [`convert`] makes the second from the first: the programme's
//! branch, the systems branch and each subsystem group's, each a group file
//! (`<group>.vgroup`) holding its headings, mounts and loops, with one node
//! file (`<node>.vnode`) per row; and each case, a case file (`.vcase`).
//!
//! What maps to what:
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
//! The conversion has an exact inverse, [`Served`]: the files, read as the
//! repository's folders, which the loader reads as it reads a checkout. The
//! proof that nothing was dropped is that the tree read through it is the
//! tree the conversion was given — except for what the conversion is for,
//! each named, and nothing else (`tests/the_design_converts_to_its_files.rs`):
//!
//! 1. each subsystem group hangs from the block it mounts on, not from the
//!    root (docs/SYSTEM_MODEL.md, section 8, "Where each group mounts");
//! 2. the architecture's loop is declared on the smallest block that holds
//!    it (section 5);
//! 3. every stated value is a parameter of the level whose branch states it
//!    (section 6, "A parameter belongs to the level that decides it");
//! 4. the five blocks the breakdown does not hold yet are there, as open
//!    blocks, each with why ([`PROPOSED`]; section 8, "Proposed: one level
//!    deeper").
//!
//! The relations still in code are not design and are not converted: a
//! row's built-in relation is found in the code by its id
//! (`vleo-modules`), and the inverse reads it from the code's own folders.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

use toml::value::Table as TomlTable;
use toml::Value;
use vleo_sheet::files::Files;
use vleo_sheet::load::Tree;
use vleo_sheet::model::Sheet;

use crate::error::{Error, ErrorKind};
use crate::meta::Kind;
use crate::model::{Block, File, Loop, Mount, Port, Tbl, TestCase, Text, Wire};

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

/// The level a stated value in a branch is a parameter of.
fn level_of(branch: &str) -> &'static str {
    match branch {
        PROGRAMME => "programme",
        SYSTEMS => "system",
        _ => "subsystem",
    }
}

/// Where a heading's branch is: the programme's, the systems' or a
/// subsystem group's, by the heading at its top — the first whose parent is
/// in another layer.
fn branch_of(tree: &Tree, heading: &str) -> Result<String, Error> {
    let mut id = heading;
    loop {
        let g = tree
            .groups
            .get(id)
            .ok_or_else(|| malformed(format!("{id} is not a heading")))?;
        if g.parent.is_empty() {
            return Ok(PROGRAMME.into());
        }
        let p = tree
            .groups
            .get(&g.parent)
            .ok_or_else(|| malformed(format!("{id}'s parent {} is not a heading", g.parent)))?;
        if p.layer != g.layer {
            return match g.layer {
                1 => Ok(PROGRAMME.into()),
                2 => Ok(SYSTEMS.into()),
                3 => Ok(g.id.clone()),
                l => Err(malformed(format!("{id} is at layer {l}, in no branch"))),
            };
        }
        id = &g.parent;
    }
}

/// The text of a number as the sheet wrote it, exactly: a whole number as
/// it is, any other by the shortest text that reads back as the same number.
fn number_text(v: &Value, what: &str) -> Result<String, Error> {
    match v {
        Value::Integer(i) => Ok(i.to_string()),
        Value::Float(x) => Ok(format!("{x:?}")),
        other => Err(malformed(format!("{what} is {other}, not a number"))),
    }
}

fn number_of(text: &str, what: &str) -> Result<Value, Error> {
    text.parse::<f64>()
        .map(Value::Float)
        .map_err(|_| malformed(format!("{what} is {text:?}, not a number")))
}

fn text_of(v: Option<Value>, what: &str) -> Result<String, Error> {
    match v {
        None => Ok(String::new()),
        Some(Value::String(s)) => Ok(s),
        Some(other) => Err(malformed(format!("{what} is {other}, not text"))),
    }
}

/// Both reasons for a range, in the one column: `lower: …` and `upper: …`,
/// each on its own, so a reader reads the two.
fn reasons(lower: &str, upper: &str) -> String {
    if lower.is_empty() && upper.is_empty() {
        String::new()
    } else {
        format!("lower: {lower}\nupper: {upper}")
    }
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

/// A case's inputs, by name: `name = value; …`, each value as [`number_text`].
fn inputs_text(t: &TomlTable, what: &str) -> Result<String, Error> {
    let mut out = Vec::new();
    for (k, v) in t {
        out.push(format!("{k} = {}", number_text(v, what)?));
    }
    Ok(out.join("; "))
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

fn take_array(t: &mut TomlTable, key: &str, what: &str) -> Result<Vec<TomlTable>, Error> {
    match t.remove(key) {
        None => Ok(Vec::new()),
        Some(Value::Array(a)) => a
            .into_iter()
            .map(|v| match v {
                Value::Table(x) => Ok(x),
                other => Err(malformed(format!(
                    "{what}: a {key} is {other}, not a table"
                ))),
            })
            .collect(),
        Some(other) => Err(malformed(format!("{what}: {key} is {other}, not a list"))),
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

fn read_text(fs: &dyn Files, p: &Path) -> Result<String, Error> {
    fs.read_to_string(p).map_err(|e| io_error(p, e))
}

fn text_row(scope: &str, kind: &str, body: String) -> Text {
    Text {
        scope: scope.into(),
        kind: kind.into(),
        body,
    }
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

/// One port out, its keys taken from `t`.
fn port_out(
    block: &str,
    name: String,
    t: &mut TomlTable,
    derived_state: &str,
    ord: i64,
    what: &str,
) -> Result<(Port, bool), Error> {
    let mut take = |k: &str| text_of(t.remove(k), &format!("{what}: {k}"));
    let symbol = take("symbol")?;
    let unit = take("unit")?;
    let reason_lower = take("reason_lower")?;
    let reason_upper = take("reason_upper")?;
    let state = take("state")?;
    let maturity = take("maturity")?;
    let parameter = take("parameter")?;
    let open_owner = take("open_owner")?;
    let open_due = take("open_due")?;
    let mut end = |k: &str| -> Result<String, Error> {
        t.remove(k)
            .map(|v| number_text(&v, &format!("{what}: {k}")))
            .transpose()
            .map(Option::unwrap_or_default)
    };
    let lower = end("lower")?;
    let upper = end("upper")?;
    let stated_state = !state.is_empty();
    Ok((
        Port {
            block_uid: block.into(),
            direction: "out".into(),
            name,
            symbol,
            port_type: "number".into(),
            unit,
            lower,
            upper,
            range_reason: reasons(&reason_lower, &reason_upper),
            state: if stated_state {
                state
            } else {
                derived_state.into()
            },
            maturity,
            parameter,
            open_owner,
            open_due,
            ord,
            ..Port::default()
        },
        stated_state,
    ))
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

/// Where each variable is answered: by its row, in which branch.
struct Producers {
    /// Each variable, as a row reads it — the row's id for its answer,
    /// `<row>.<id>` for what it publishes — by its row, its port there, that
    /// row's branch, and where its value stands.
    of: BTreeMap<String, (String, String, String, &'static str)>,
}

impl Producers {
    fn new(tree: &Tree, branch: &BTreeMap<String, String>) -> Producers {
        let mut of = BTreeMap::new();
        for sh in tree.sheets.values() {
            let b = branch.get(&sh.id).cloned().unwrap_or_default();
            let state = sh.port_state(&sh.port);
            of.insert(
                sh.id.clone(),
                (sh.id.clone(), sh.id.clone(), b.clone(), state),
            );
            for pb in &sh.publishes {
                let state = sh.port_state(&pb.port);
                of.insert(
                    format!("{}.{}", sh.id, pb.id),
                    (sh.id.clone(), pb.id.clone(), b.clone(), state),
                );
            }
        }
        Producers { of }
    }

    /// The wire's source, as the one schema writes it.
    fn reference(&self, var: &str, from_branch: &str, what: &str) -> Result<String, Error> {
        let (row, port, branch, _) = self
            .of
            .get(var)
            .ok_or_else(|| malformed(format!("{what} reads {var}, which no row answers")))?;
        Ok(if branch == from_branch {
            format!("{row}.{port}")
        } else {
            format!("{branch}.{row}.{port}")
        })
    }
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

/// One row, as its node file.
#[allow(clippy::too_many_arguments)]
fn node_file(
    sheet: &Sheet,
    mut raw: TomlTable,
    raw_text: &str,
    fixtures: Option<&str>,
    parity: Option<String>,
    branch: &str,
    producers: &Producers,
    app: &str,
) -> Result<File, Error> {
    let id = sheet.id.as_str();
    let what = format!("{id}'s sheet");
    let mut f = File::new(Kind::Node, app);
    if sheet.owner.trim().is_empty() {
        return Err(malformed(format!("{id} names no owner to write it")));
    }
    for (k, v) in [
        ("group_id", branch),
        ("block_uid", id),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", sheet.owner.as_str()),
        ("sheet.crate", sheet.crate_name.as_str()),
    ] {
        f.meta.insert(k.into(), v.into());
    }
    let mut derived: Vec<Value> = Vec::new();

    // ── the block
    raw.remove("id");
    let parent = text_of(raw.remove("parent"), &format!("{what}: parent"))?;
    let ord = match raw.remove("order") {
        None => 0,
        Some(Value::Integer(i)) => i,
        Some(other) => return Err(malformed(format!("{what}: order is {other}"))),
    };
    raw.remove("layer");
    let mut question = String::new();
    if let Some(Value::Table(q)) = raw.get_mut("question") {
        question = text_of(q.remove("text"), &format!("{what}: question"))?;
    }
    f.blocks.push(Block {
        uid: id.into(),
        id: id.into(),
        parent_uid: parent,
        question,
        behaviour: sheet.behaviour().into(),
        perspective: perspective(sheet.layer)?.into(),
        ord,
        archived: (sheet.state == "deprecated") as i64,
        contract_version: 1,
        revision: 1,
    });

    // ── its ports out: the output, its value, then what it publishes
    let derived_state = sheet.port_state(&sheet.port);
    let mut output = take_table(&mut raw, "output", &what)?.unwrap_or_default();
    let (mut port, stated) = port_out(id, id.into(), &mut output, derived_state, 0, &what)?;
    if !stated {
        derived.push(Value::String("output.state".into()));
    }
    // A stated value is a parameter of the level whose branch states it.
    if sheet.is_declared() && !sheet.is_seeded() && port.parameter.is_empty() {
        port.parameter = level_of(branch).into();
    }
    if let Some(Value::Table(v)) = raw.get_mut("value") {
        if let Some(n) = v.remove("number") {
            port.value = number_text(&n, &format!("{what}: value"))?;
        }
    }
    f.ports.push(port);
    raw.insert("output".into(), Value::Table(output));
    let mut publishes = take_array(&mut raw, "publishes", &what)?;
    for (k, (pb, sh_pb)) in publishes.iter_mut().zip(&sheet.publishes).enumerate() {
        let name = text_of(pb.remove("id"), &format!("{what}: publishes id"))?;
        let says = text_of(pb.remove("label"), &format!("{what}: publishes label"))?;
        let (mut p, stated) = port_out(
            id,
            name,
            pb,
            sheet.port_state(&sh_pb.port),
            k as i64 + 1,
            &what,
        )?;
        if !stated {
            derived.push(Value::String(format!("publishes.{k}.state")));
        }
        p.says = says;
        f.ports.push(p);
    }
    if !publishes.is_empty() {
        raw.insert(
            "publishes".into(),
            Value::Array(publishes.into_iter().map(Value::Table).collect()),
        );
    }

    // ── its ports in, and the wire into each
    let mut inputs = take_array(&mut raw, "input", &what)?;
    for (k, i) in inputs.iter_mut().enumerate() {
        let binding = text_of(i.remove("binding"), &format!("{what}: input binding"))?;
        let var = text_of(i.remove("var"), &format!("{what}: input var"))?;
        // An input stands where the value it reads stands.
        let state = producers.of.get(&var).map(|p| p.3).unwrap_or("open");
        f.wires.push(Wire {
            to_block: id.into(),
            to_port: binding.clone(),
            from_ref: producers.reference(&var, branch, &format!("{id}.{binding}"))?,
        });
        f.ports.push(Port {
            block_uid: id.into(),
            direction: "in".into(),
            name: binding,
            port_type: "number".into(),
            state: state.into(),
            ord: k as i64,
            ..Port::default()
        });
    }
    if !inputs.is_empty() {
        raw.insert(
            "input".into(),
            Value::Array(inputs.into_iter().map(Value::Table).collect()),
        );
    }

    // ── its method and its explanation
    if let Some(Value::Table(m)) = raw.get_mut("method") {
        let text = text_of(m.remove("text"), &format!("{what}: method"))?;
        if !text.is_empty() {
            f.texts.push(text_row(id, "method", text));
        }
    }
    if let Some(Value::Table(x)) = raw.get_mut("explain") {
        for part in ["simply", "breaks", "wrong"] {
            let text = text_of(x.remove(part), &format!("{what}: explain.{part}"))?;
            if !text.is_empty() {
                f.texts.push(text_row(id, &format!("explain.{part}"), text));
            }
        }
    }

    // ── its cases
    if let Some(text) = fixtures {
        let fwhat = format!("{id}'s fixtures");
        let mut fx = toml_table(text, &fwhat)?;
        let mut rows = take_array(&mut fx, "fixture", &fwhat)?;
        for (k, r) in rows.iter_mut().enumerate() {
            let label = text_of(r.remove("label"), &fwhat)?;
            let provenance = text_of(r.remove("provenance"), &fwhat)?;
            let source = text_of(r.remove("source"), &fwhat)?;
            let mut num = |key: &str| -> Result<String, Error> {
                r.remove(key)
                    .map(|v| number_text(&v, &format!("{fwhat}: {key}")))
                    .transpose()
                    .map(Option::unwrap_or_default)
            };
            let expected = num("expect")?;
            let tolerance = num("tolerance")?;
            let inputs = match r.remove("inputs") {
                None => String::new(),
                Some(Value::Table(t)) => inputs_text(&t, &fwhat)?,
                Some(other) => return Err(malformed(format!("{fwhat}: inputs is {other}"))),
            };
            f.cases.push(TestCase {
                block_uid: id.into(),
                name: format!("{} · {label}", k + 1),
                inputs,
                expected,
                tolerance,
                provenance,
                source,
            });
        }
        fx.insert(
            "fixture".into(),
            Value::Array(rows.into_iter().map(Value::Table).collect()),
        );
        f.texts
            .push(text_row(id, FIXTURES, toml_text(&fx, &fwhat)?));
        f.texts.push(text_row(
            id,
            &format!("{KEPT}fixtures.toml"),
            text.to_string(),
        ));
    }
    if let Some(csv) = parity {
        f.tables.push(Tbl {
            scope: id.into(),
            path: "parity.csv".into(),
            csv,
        });
    }

    // ── what has no column yet, and the sheet as it stood
    if !derived.is_empty() {
        let mut c = TomlTable::new();
        c.insert("derived".into(), Value::Array(derived));
        raw.insert(CONVERTED.into(), Value::Table(c));
    }
    f.texts.push(text_row(id, SHEET, toml_text(&raw, &what)?));
    f.texts
        .push(text_row(id, &format!("{KEPT}node.toml"), raw_text.into()));
    f.check_meta()?;
    f.check_values()?;
    Ok(f)
}

/// The sheet of an open block the conversion adds, seeded as every row not
/// decided yet is, with why it was proposed as its question's note.
fn proposed_sheet(
    id: &str,
    name: &str,
    why: &str,
    interface: &Sheet,
    heading: &str,
    owner: &str,
    order: u32,
) -> String {
    let mut t = TomlTable::new();
    let s = |v: &str| Value::String(v.into());
    t.insert("id".into(), s(id));
    t.insert("label".into(), s(name));
    t.insert("folder".into(), s(id));
    t.insert("subsystem".into(), s(&interface.subsystem));
    t.insert("parent".into(), s(heading));
    t.insert("kind".into(), s("computed"));
    t.insert("owner".into(), s(owner));
    t.insert("tier".into(), s(""));
    t.insert("layer".into(), Value::Integer(3));
    t.insert("order".into(), Value::Integer(order as i64));
    t.insert("state".into(), s("empty"));
    let mut q = TomlTable::new();
    q.insert("text".into(), s(""));
    q.insert("note".into(), s(why));
    t.insert("question".into(), Value::Table(q));
    let mut m = TomlTable::new();
    m.insert("expression".into(), s(""));
    m.insert("source".into(), s(""));
    t.insert("maths".into(), Value::Table(m));
    let mut o = TomlTable::new();
    for k in ["symbol", "type", "unit", "reason_lower", "reason_upper"] {
        o.insert(k.into(), s(""));
    }
    o.insert("lower".into(), Value::Float(0.0));
    o.insert("upper".into(), Value::Float(0.0));
    t.insert("output".into(), Value::Table(o));
    let mut v = TomlTable::new();
    v.insert("kind".into(), s("number"));
    t.insert("view".into(), Value::Table(v));
    toml::to_string(&t).expect("a seeded sheet is TOML")
}

/// The smallest heading that holds every row in `members`, in the tree as
/// the conversion hangs it.
fn smallest_holding(
    tree: &Tree,
    parent_of: &BTreeMap<String, String>,
    members: &[String],
) -> Result<String, Error> {
    let chain = |row: &str| -> Result<Vec<String>, Error> {
        let sh = tree
            .sheets
            .get(row)
            .ok_or_else(|| malformed(format!("a loop runs through {row}, which is no row")))?;
        let mut out = vec![sh.parent.clone()];
        while let Some(p) = parent_of.get(out.last().unwrap()).filter(|p| !p.is_empty()) {
            out.push(p.clone());
        }
        out.reverse();
        Ok(out)
    };
    let mut common: Option<Vec<String>> = None;
    for m in members {
        let c = chain(m)?;
        common = Some(match common {
            None => c,
            Some(prev) => prev
                .into_iter()
                .zip(c)
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a)
                .collect(),
        });
    }
    common
        .and_then(|c| c.last().cloned())
        .ok_or_else(|| malformed("a loop runs through no row"))
}

/// The design, as its files, each with the path it has on the drive.
///
/// `fs` reads the tree's folders (the loader's `Disk`); `app` is the
/// application writing them, as `written_by_app` names it.
pub fn convert(tree: &Tree, fs: &dyn Files, app: &str) -> Result<Vec<(String, File)>, Error> {
    let root = &tree.root;

    // ── which branch every heading and row is in
    let mut branch: BTreeMap<String, String> = BTreeMap::new();
    for id in tree.groups.keys() {
        branch.insert(id.clone(), branch_of(tree, id)?);
    }
    for sh in tree.sheets.values() {
        let b = branch.get(&sh.parent).cloned().ok_or_else(|| {
            malformed(format!("{}'s parent {} is not a heading", sh.id, sh.parent))
        })?;
        branch.insert(sh.id.clone(), b);
    }
    let branches: BTreeSet<String> = tree.groups.keys().map(|g| branch[g].clone()).collect();

    // ── where each branch hangs: a subsystem group on the heading its
    // crossing row crosses to, any other on its parent
    let mut parent_of: BTreeMap<String, String> = tree
        .groups
        .values()
        .map(|g| (g.id.clone(), g.parent.clone()))
        .collect();
    let mut mounts: Vec<(String, String)> = Vec::new();
    for g in tree.groups.values() {
        let Some(p) = tree.groups.get(&g.parent) else {
            continue;
        };
        let b = &branch[&g.id];
        if p.layer == g.layer || &branch[&p.id] == b {
            continue;
        }
        let on = if g.layer == 3 {
            let crossings: BTreeSet<&str> = tree
                .sheets
                .values()
                .filter(|s| &branch[&s.id] == b && !s.crosses_to.is_empty())
                .map(|s| s.crosses_to.as_str())
                .collect();
            match crossings.into_iter().collect::<Vec<_>>().as_slice() {
                [one] if tree.groups.contains_key(*one) => one.to_string(),
                other => {
                    return Err(malformed(format!(
                    "{b} crosses to {other:?}: a group hangs from one heading, its crossing row's"
                )))
                }
            }
        } else {
            g.parent.clone()
        };
        parent_of.insert(g.id.clone(), on.clone());
        mounts.push((on, b.clone()));
    }

    // ── the files
    let producers = Producers::new(tree, &branch);
    let mut group_files: BTreeMap<String, File> = BTreeMap::new();
    for b in &branches {
        // Its top: the heading whose parent is in no branch, or another's.
        let top = tree
            .groups
            .values()
            .find(|g| &branch[&g.id] == b && branch.get(&g.parent) != Some(b))
            .ok_or_else(|| malformed(format!("{b} has no heading at its top")))?;
        let mut f = File::new(Kind::Group, app);
        f.meta.insert("group_id".into(), b.clone());
        f.meta.insert("writer".into(), top.owner.clone());
        f.meta.insert("based_on".into(), String::new());
        group_files.insert(b.clone(), f);
    }
    for (on, b) in &mounts {
        group_files
            .get_mut(&branch[on])
            .expect("every branch has a file")
            .mounts
            .push(Mount {
                block_uid: on.clone(),
                group_id: b.clone(),
                release: String::new(),
            });
    }

    // ── the headings, from their layer files, and the loops
    let mut layer_files: Vec<PathBuf> = fs
        .entries(&root.join("layers"))
        .map_err(|e| io_error(&root.join("layers"), e))?
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    layer_files.sort();
    let mut loops: Vec<(TomlTable, String)> = Vec::new();
    for p in &layer_files {
        let text = read_text(fs, p)?;
        let stem = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let what = format!("layers/{stem}.toml");
        let mut v = toml_table(&text, &what)?;
        for it in take_array(&mut v, "iterate", &what)? {
            loops.push((it, what.clone()));
        }
        let groups = take_array(&mut v, "group", &what)?;
        let relates = v.remove("relates");
        if !v.is_empty() {
            return Err(malformed(format!(
                "{what} holds {:?}, which a layer file has not",
                v.keys().collect::<Vec<_>>()
            )));
        }
        let kept_in = match groups.as_slice() {
            [] if relates.is_none() => None,
            [g] => Some(text_of(g.get("id").cloned(), &what)?),
            _ => {
                return Err(malformed(format!(
                    "{what} holds {} headings: a layer file holds one",
                    groups.len()
                )))
            }
        };
        for mut g in groups {
            let id = text_of(g.remove("id"), &what)?;
            g.remove("parent");
            let ord = match g.remove("order") {
                None => 0,
                Some(Value::Integer(i)) => i,
                Some(other) => return Err(malformed(format!("{what}: order is {other}"))),
            };
            g.remove("layer");
            for mut it in take_array(&mut g, "iterate", &what)? {
                it.insert("on".into(), Value::String(id.clone()));
                loops.push((it, what.clone()));
            }
            let grp = &tree.groups[&id];
            let b = &branch[&id];
            let parent = &parent_of[&id];
            let f = group_files.get_mut(b).expect("every branch has a file");
            f.blocks.push(Block {
                uid: id.clone(),
                id: id.clone(),
                parent_uid: if branch.get(parent) == Some(b) {
                    parent.clone()
                } else {
                    String::new()
                },
                question: String::new(),
                behaviour: "children".into(),
                perspective: perspective(grp.layer)?.into(),
                ord,
                archived: 0,
                contract_version: 1,
                revision: 1,
            });
            let mut c = TomlTable::new();
            c.insert("file".into(), Value::String(stem.clone()));
            g.insert(CONVERTED.into(), Value::Table(c));
            if let Some(r) = &relates {
                g.insert("relates".into(), r.clone());
            }
            f.texts.push(text_row(&id, HEADING, toml_text(&g, &what)?));
        }
        let keeper = match &kept_in {
            Some(id) => branch[id].clone(),
            None => SYSTEMS.to_string(),
        };
        group_files
            .get_mut(&keeper)
            .expect("every branch has a file")
            .texts
            .push(text_row(FILE, &format!("{KEPT}{what}"), text));
    }
    for (n, (mut it, what)) in loops.into_iter().enumerate() {
        let members: Vec<String> = match it.remove("nodes") {
            Some(Value::Array(a)) => a
                .into_iter()
                .map(|v| text_of(Some(v), &what))
                .collect::<Result<_, _>>()?,
            _ => Vec::new(),
        };
        let on = match it.remove("on") {
            Some(Value::String(on)) => on,
            _ => smallest_holding(tree, &parent_of, &members)?,
        };
        let settles = text_of(it.remove("converge_on"), &what)?;
        let tolerance = it
            .remove("tolerance")
            .map(|v| number_text(&v, &what))
            .transpose()?
            .unwrap_or_default();
        let max = match it.remove("max_iter") {
            None => 0,
            Some(Value::Integer(i)) => i,
            Some(other) => return Err(malformed(format!("{what}: max_iter is {other}"))),
        };
        let uid = format!("loop-{}", n + 1);
        let f = group_files
            .get_mut(&branch[&on])
            .expect("every branch has a file");
        f.loops.push(Loop {
            uid: uid.clone(),
            block_uid: on,
            members: members.join(","),
            settles,
            tolerance,
            max_iterations: max,
        });
        f.texts.push(text_row(&uid, LOOP, toml_text(&it, &what)?));
    }

    // ── the sources, and the cases, kept as they are by their path
    let sources = root.join("sources/sources.toml");
    group_files
        .get_mut(SYSTEMS)
        .expect("the systems branch has a file")
        .texts
        .push(text_row(
            FILE,
            &format!("{KEPT}sources/sources.toml"),
            read_text(fs, &sources)?,
        ));
    let mut out: Vec<(String, File)> = Vec::new();
    let mut cases: Vec<PathBuf> = fs
        .entries(&root.join("cases"))
        .map_err(|e| io_error(&root.join("cases"), e))?;
    if fs.is_dir(&root.join("cases/examples")) {
        cases.extend(
            fs.entries(&root.join("cases/examples"))
                .map_err(|e| io_error(&root.join("cases/examples"), e))?,
        );
    }
    cases.retain(|p| fs.is_file(p));
    cases.sort();
    for p in cases {
        let rel = p
            .strip_prefix(root)
            .map_err(|_| malformed(format!("{} is outside the tree", p.display())))?
            .to_string_lossy()
            .replace('\\', "/");
        let name = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let mut f = File::new(Kind::Case, app);
        f.meta.insert("name".into(), name.clone());
        let body = read_text(fs, &p)?;
        if rel.ends_with(".toml") {
            f.texts.push(text_row(FILE, &format!("{KEPT}{rel}"), body));
        } else {
            f.tables.push(Tbl {
                scope: FILE.into(),
                path: rel.clone(),
                csv: body,
            });
        }
        f.check_meta()?;
        let dir = rel.rsplit_once('/').map(|(d, _)| d).unwrap_or("cases");
        out.push((format!("{dir}/{name}.{CASE_FILE}"), f));
    }

    // ── every row, as its node file
    for sh in tree.sheets.values() {
        let b = &branch[&sh.id];
        let path = sh.dir.join("node.toml");
        let raw_text = read_text(fs, &path)?;
        let raw = toml_table(&raw_text, &path.display().to_string())?;
        let fx = sh.dir.join("fixtures.toml");
        let fixtures = if fs.is_file(&fx) {
            Some(read_text(fs, &fx)?)
        } else {
            None
        };
        let pc = sh.dir.join("parity.csv");
        let parity = if fs.is_file(&pc) {
            Some(read_text(fs, &pc)?)
        } else {
            None
        };
        let f = node_file(
            sh,
            raw,
            &raw_text,
            fixtures.as_deref(),
            parity,
            b,
            &producers,
            app,
        )?;
        out.push((format!("groups/{b}/nodes/{}.{NODE_FILE}", sh.id), f));
    }

    // ── the blocks a breakdown does not hold yet, as open blocks
    //
    // Each takes the first place after its heading's rows that no row of the
    // design holds: a place is the design's, not the group's (gate V14).
    let mut taken: BTreeSet<u32> = tree.sheets.values().map(|s| s.order).collect();
    for (group, names) in PROPOSED {
        let interface = tree
            .sheets
            .values()
            .find(|s| &branch[&s.id] == group && !s.crosses_to.is_empty())
            .ok_or_else(|| {
                malformed(format!("{group} has no crossing row to open a level below"))
            })?;
        let heading = &tree.groups[*group];
        let mut order = tree
            .sheets
            .values()
            .filter(|s| s.parent == heading.id)
            .map(|s| s.order)
            .max()
            .unwrap_or(0);
        for (name, why) in *names {
            let id = proposed_id(group, name);
            if tree.sheets.contains_key(&id) || tree.groups.contains_key(&id) {
                return Err(malformed(format!(
                    "the proposed block {id} is already in the design"
                )));
            }
            order += 1;
            while !taken.insert(order) {
                order += 1;
            }
            let text = proposed_sheet(
                &id,
                name,
                why,
                interface,
                &heading.id,
                &heading.owner,
                order,
            );
            let raw = toml_table(&text, &id)?;
            let sheet = Sheet {
                id: id.clone(),
                parent: heading.id.clone(),
                owner: heading.owner.clone(),
                state: "empty".into(),
                kind: "computed".into(),
                layer: 3,
                crate_name: interface.crate_name.clone(),
                ..Sheet::default()
            };
            let f = node_file(&sheet, raw, &text, None, None, group, &producers, app)?;
            out.push((format!("groups/{group}/nodes/{id}.{NODE_FILE}"), f));
        }
    }

    for (b, f) in group_files {
        f.check_meta()?;
        f.check_values()?;
        out.push((format!("groups/{b}/{b}.{GROUP_FILE}"), f));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// The design's files, read as the repository's folders: the inverse of
/// [`convert`].
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

/// Every group, node and case file in `dir`, laid out as [`convert`] lays
/// them out, each with its path there, and the fingerprint of all of them: the
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
