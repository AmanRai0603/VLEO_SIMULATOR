//! A saved result: what one run returned, and the inputs it ran on.
//!
//! A run is a question somebody asked, and its answer is worth keeping — to
//! look at again, to send to someone, to compare with the next one. So a run
//! can be saved as ONE CSV: every input it ran on, every value it returned with
//! its unit and credibility, every row it could not run and why, and the run's
//! identity — the chain hash that says exactly which engine, tree and inputs
//! produced it. It opens in any spreadsheet. Uploaded again, it is shown as it
//! was, WITHOUT RUNNING ANYTHING: a saved result is a record, and re-running it
//! on today's engine would be a different result under the old one's name.
//!
//! Its input rows are the case it ran on, so they can be loaded back as the
//! case — which is how a result from somebody else becomes a starting point.
//!
//! Every value is written twice: in the row's own display unit for a person,
//! and in SI for the tool, so a result read back is the same numbers bit for
//! bit, whatever the display rounded.
//!
//! Like the case, results are kept outside the repository. Git never sees one.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::inputs::{case_inputs, num, template};
use crate::vars;
use crate::Vleo;
use crate::{Error, ErrorKind};

/// What a result file declares itself to be.
pub const FORMAT: &str = "vleo-result/1";

/// One row of a result: an input it ran on, a value it returned, or a row it
/// could not run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    pub id: String,
    pub name: String,
    /// As a person reads it, in `unit`.
    pub value: String,
    pub unit: String,
    /// Exact, when there is a value.
    pub si: Option<f64>,
    pub credibility: String,
    pub governing: String,
    /// For an input, `changed`, `default`, or `supplied` for a declared row
    /// the run was given that is not one of the case's inputs; for a blocked
    /// row, why.
    pub note: String,
    /// The eight credibility factors, one digit each (`34233333`), so a result
    /// shown again draws the same bars the run did. Empty in a result saved
    /// before it was recorded.
    pub cred: String,
}

/// One answer across a range of one input: what a sweep returned, kept so it
/// can be drawn again without running.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sweep {
    /// The input moved, and its label.
    pub over: String,
    pub over_name: String,
    /// Display units and their factor to SI, for the axis a person reads.
    pub x_unit: String,
    pub x_factor: f64,
    pub y_unit: String,
    pub y_factor: f64,
    /// The range asked for, in SI, and how many points across it.
    pub from: f64,
    pub to: f64,
    pub points: usize,
    /// Every point that answered, in SI.
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    /// Every point that was refused, with why. Kept, never dropped: a gap in
    /// the line is a fact about the design.
    pub refused: Vec<(f64, String)>,
}

/// What a sweep file declares itself to be.
pub const SWEEP_FORMAT: &str = "vleo-sweep/1";

/// A saved result.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Saved {
    pub target: String,
    pub mode: String,
    /// When it was saved, as the saver's clock said.
    pub saved: String,
    /// A name somebody gave it, or nothing.
    pub name: String,
    pub chain: String,
    pub kernel: String,
    pub graph: String,
    /// The fingerprint of the design's files the run read, or empty when it read
    /// the folders of a checkout. Says which design the answer belongs to.
    pub design: String,
    pub case: String,
    pub data: Vec<String>,
    /// The inputs template it ran against — see `inputs::template`.
    pub template: String,
    pub ran: usize,
    pub blocked_count: usize,
    pub inputs: Vec<Row>,
    pub outputs: Vec<Row>,
    pub blocked: Vec<Row>,
    /// The recorded version of every node the run went through that has one,
    /// with the release that carried it: `(node, version, release)`.
    pub versions: Vec<(String, u32, String)>,
    /// Whether the result says which versions it rested on at all — a
    /// `#! versions` line, empty or not. One saved before the tool recorded
    /// beliefs has none, and nothing can be said about what has moved since.
    pub versions_known: bool,
    /// The sweep, when the result is one.
    pub sweep: Option<Sweep>,
    /// When the result was thinned to its summary, and how many values went —
    /// `2026-10-29 104` — or empty for a result kept whole. See [`thin`].
    pub thinned: String,
}

impl Saved {
    /// The target's own answer, when the run produced one.
    pub fn answer(&self) -> Option<&Row> {
        self.outputs.iter().find(|r| r.id == self.target)
    }
    /// How many inputs it ran at a value other than their default.
    pub fn changed(&self) -> usize {
        self.inputs.iter().filter(|r| r.note == "changed").count()
    }
    /// The question this result answers — see [`question`].
    pub fn question(&self) -> String {
        question(
            &self.target,
            &self.mode,
            &self.kernel,
            &self.graph,
            &self.data,
            &self.inputs,
            self.sweep
                .as_ref()
                .map(|w| (w.over.as_str(), w.from, w.to, w.points)),
        )
    }
    /// The inputs it ran on, in SI — to load back as the case.
    pub fn case_values(&self) -> Vec<(String, f64)> {
        self.inputs
            .iter()
            .filter(|r| r.note == "changed")
            .filter_map(|r| r.si.map(|v| (r.id.clone(), v)))
            .collect()
    }
}

/// A result from a run: the run's values, and the inputs it ran at — the saved
/// case with whatever the run itself set on top, in SI.
pub fn from_run(r: &vleo_bus::Results, supply: &[(String, f64)], saved: &str, name: &str) -> Saved {
    let m = &r.manifest;
    let mut s = Saved {
        target: m.node.clone(),
        mode: m.mode.to_string(),
        saved: saved.to_string(),
        name: name.to_string(),
        chain: m.chain.clone(),
        kernel: m.kernel.clone(),
        graph: m.graph.clone(),
        case: m.case.clone(),
        data: m.data.clone(),
        template: template(),
        ran: m.ran,
        blocked_count: m.blocked_count,
        ..Default::default()
    };
    s.inputs = ran_inputs(supply);
    for v in &r.values {
        let (shown, unit) = match Vleo::find(&v.id) {
            Some(k) => {
                let u = vars()[k as usize].unit;
                (num(v.value / u.si_factor()), u.symbol())
            }
            None => (num(v.value), v.unit),
        };
        s.outputs.push(Row {
            id: v.id.clone(),
            name: v.label.clone(),
            value: shown,
            unit: unit.to_string(),
            si: Some(v.value),
            credibility: v.cred.governing_score().to_string(),
            governing: v.governing.to_string(),
            cred: v
                .cred
                .0
                .iter()
                .map(|c| char::from(b'0' + (*c).min(9)))
                .collect(),
            ..Default::default()
        });
    }
    for o in &s.outputs {
        if let Some((n, rel)) = node_version(&o.id) {
            s.versions.push((o.id.clone(), n, rel.to_string()));
        }
    }
    s.versions_known = true;
    for b in &r.blocked {
        s.blocked.push(Row {
            id: b.id.clone(),
            name: Vleo::find(&b.id)
                .map(|k| vars()[k as usize].label.to_string())
                .unwrap_or_default(),
            note: b.message.clone(),
            ..Default::default()
        });
    }
    s
}

/// The inputs a run applied: every one of the case's inputs at the value it ran
/// at — the last value supplied wins, exactly as the run applied them — then
/// any other declared row the run was given, which a result records too.
pub fn ran_inputs(supply: &[(String, f64)]) -> Vec<Row> {
    let last = |id: &str| supply.iter().rev().find(|(k, _)| k == id).map(|(_, v)| *v);
    let all = case_inputs();
    let mut rows = Vec::new();
    for i in &all {
        let si = last(i.id).unwrap_or(i.default);
        rows.push(Row {
            id: i.id.to_string(),
            name: i.label.to_string(),
            value: num(i.shown(si)),
            unit: i.unit.to_string(),
            si: Some(si),
            note: if si != i.default {
                "changed"
            } else {
                "default"
            }
            .to_string(),
            ..Default::default()
        });
    }
    let mut extra: Vec<&str> = supply
        .iter()
        .map(|(k, _)| k.as_str())
        .filter(|k| !all.iter().any(|i| i.id == *k))
        .collect();
    extra.sort_unstable();
    extra.dedup();
    for id in extra {
        let si = last(id).unwrap_or_default();
        let (name, value, unit) = match Vleo::find(id) {
            Some(k) => {
                let v = &vars()[k as usize];
                (
                    v.label.to_string(),
                    num(si / v.unit.si_factor()),
                    v.unit.symbol(),
                )
            }
            None => (String::new(), num(si), "-"),
        };
        rows.push(Row {
            id: id.to_string(),
            name,
            value,
            unit: unit.to_string(),
            si: Some(si),
            note: "supplied".to_string(),
            ..Default::default()
        });
    }
    rows
}

/// Whether a result was made by the engine that answers here. A result from
/// another engine is shown as it was, and says so: its numbers came from
/// relations this tool may no longer hold, and running its inputs again gives
/// this engine's answer.
pub fn engine_current(s: &Saved) -> bool {
    let (kernel, graph) = engine();
    s.kernel == kernel && s.graph == graph
}

/// The engine that answers here, as a run's manifest and so a result records
/// it: `(kernel, graph)`.
pub fn engine() -> (String, String) {
    (
        crate::hex(Vleo::kernel_hash()),
        crate::hex(Vleo::graph_hash()),
    )
}

/// The question a result answers, as one key: which row, how much of the graph,
/// which engine and tree, which reference data, every input it ran at, and —
/// for a sweep — what was swept and how.
///
/// Two results with the same key are the same answer: the engine is
/// deterministic, and everything it reads is in the key. So a question already
/// answered is shown from its saved result rather than asked again, and saved
/// once rather than twice. The key is worked out from what a result records,
/// never read from the file, so a result saved before keys existed has one, and
/// a file cannot claim a key its contents do not produce.
pub fn question(
    target: &str,
    mode: &str,
    kernel: &str,
    graph: &str,
    data: &[String],
    inputs: &[Row],
    sweep: Option<(&str, f64, f64, usize)>,
) -> String {
    let mut h = vleo_core::hash::Hasher::new();
    for part in [target, mode, kernel, graph] {
        h.write_str(part);
    }
    let mut d: Vec<&str> = data.iter().map(String::as_str).collect();
    d.sort_unstable();
    for x in d {
        h.write_str(x);
    }
    let mut ins: Vec<(&str, f64)> = inputs
        .iter()
        .filter_map(|r| r.si.map(|v| (r.id.as_str(), v)))
        .collect();
    ins.sort_by(|a, b| a.0.cmp(b.0));
    for (id, v) in ins {
        h.write_str(id);
        h.write_f64(v);
    }
    match sweep {
        Some((over, from, to, points)) => {
            h.write_str("sweep");
            h.write_str(over);
            h.write_f64(from);
            h.write_f64(to);
            h.write_u64(points as u64);
        }
        None => h.write_str("point"),
    }
    format!("{:016x}", h.finish())
}

/// A result thinned to its summary: every input it ran on (the case, so it can
/// be run again), the answer, every row that could not run and why, and the
/// beliefs it rested on — everything but the other values of the run.
///
/// ALL VALUES FOR A TIME, THE SUMMARY FOR EVER (docs/ARCHITECTURE.html,
/// section 6). A run's full set of values is the heavy part of a result and the
/// part a person least often reads again; the engine gives it back on request,
/// because runs are deterministic and the inputs are kept. So an unpinned
/// result is thinned once it is older than the keep period, and says so.
pub fn thin(s: &Saved, today: &str) -> Saved {
    let mut t = s.clone();
    if !t.thinned.is_empty() {
        return t;
    }
    let before = t.outputs.len();
    t.outputs.retain(|o| o.id == t.target);
    t.thinned = format!("{today} {}", before - t.outputs.len());
    t
}

/// A behaviour sweep: `node` answered across the input `over`, from `from` to
/// `to` (SI) in `points` evenly spaced steps, with every other input as `base`
/// sets it. Each refused point is kept with why, never dropped — a sweep in
/// which some points quietly used a substituted value is a sweep whose
/// conclusion is unknown.
///
/// The one loop every face runs: the local engine for `/v1/sweep`, and the
/// engine compiled for the browser for a page read without it.
pub fn sweep(
    base: &vleo_bus::Case,
    node: &str,
    over: &str,
    from: f64,
    to: f64,
    points: usize,
) -> Result<Sweep, Error> {
    // The axis has to be a row a reader can actually move. Sweeping a computed
    // one drew a flat line and reported no refusals, which is the same silent
    // substitution as `set=` on one and reads as a real result.
    if let Some(why) = crate::why_not_suppliable(over) {
        return Err(Error::new(ErrorKind::Invalid, why));
    }
    let (ni, oi) = match (Vleo::find(node), Vleo::find(over)) {
        (Some(a), Some(b)) => (a as usize, b as usize),
        _ => {
            return Err(Error::new(
                ErrorKind::Invalid,
                "the sweep names a node that does not exist",
            ))
        }
    };
    if points < 2 {
        return Err(Error::new(
            ErrorKind::Invalid,
            "a sweep needs at least two points",
        ));
    }
    let mut w = Sweep {
        over: over.to_string(),
        over_name: vars()[oi].label.to_string(),
        x_unit: vars()[oi].unit.symbol().to_string(),
        x_factor: vars()[oi].unit.si_factor(),
        y_unit: vars()[ni].unit.symbol().to_string(),
        y_factor: vars()[ni].unit.si_factor(),
        from,
        to,
        points,
        ..Default::default()
    };
    let mut scratch = crate::Scratch::new();
    for i in 0..points {
        let t = i as f64 / (points - 1) as f64;
        let x = from + t * (to - from);
        let mut case = base.clone();
        case.target = node.to_string();
        case.supply.push((over.to_string(), x));
        match crate::evaluate(&case, &mut scratch) {
            Ok(r) => match r.values.iter().find(|v| v.id == node) {
                Some(v) => {
                    w.x.push(x);
                    w.y.push(v.value);
                }
                None => w.refused.push((x, "blocked".to_string())),
            },
            Err(f) => w.refused.push((x, format!("{f}"))),
        }
    }
    Ok(w)
}

/// What a saved result draws, described by the engine (see [`crate::figure`]):
/// for a sweep, the answer across what it moved, with the case it was saved at
/// called out. A single run draws nothing; its answer is a number.
pub fn figures(s: &Saved) -> Vec<crate::figure::Figure> {
    let Some(w) = &s.sweep else {
        return Vec::new();
    };
    let si = |rows: &[Row], id: &str| rows.iter().find(|r| r.id == id).and_then(|r| r.si);
    let here = si(&s.inputs, &w.over).zip(si(&s.outputs, &s.target));
    vec![crate::figure::from_sweep(&s.target, w, here)]
}

/// The question a case asks, as a saved result's key — worked out from the case
/// exactly as the run would apply it, so a question answered and saved by the
/// browser is found by the command line, and on another laptop.
pub fn question_for(case: &vleo_bus::Case, sweep: Option<(&str, f64, f64, usize)>) -> String {
    let (kernel, graph) = engine();
    question(
        &case.target,
        case.mode.name(),
        &kernel,
        &graph,
        &case.data_versions,
        &ran_inputs(&case.supply),
        sweep,
    )
}

/// A node's current recorded version and the release that carried it, as
/// the design the engine runs records it.
pub fn node_version(id: &str) -> Option<(u32, &'static str)> {
    crate::engine()
        .versions
        .iter()
        .find(|(n, _, _)| *n == id)
        .map(|(_, v, r)| (*v, *r))
}

/// The nodes whose record has moved past the version a result ran on:
/// `(node, then, now)`. Each is a belief the result rested on that has since
/// broken, as far as this build knows.
///
/// A row the run went through with no recorded version then rested on a belief
/// nobody had written down: its first version since is version 0 moving to 1.
/// That is only known for a result that says which versions it rested on.
pub fn moved_since(s: &Saved) -> Vec<(String, u32, u32)> {
    if !s.versions_known {
        return Vec::new();
    }
    let recorded = s.versions.iter().map(|(id, n, _)| (id.as_str(), *n));
    let unrecorded = s
        .outputs
        .iter()
        .filter(|o| !s.versions.iter().any(|(id, _, _)| *id == o.id))
        .map(|o| (o.id.as_str(), 0));
    recorded
        .chain(unrecorded)
        .filter_map(|(id, then)| {
            node_version(id)
                .filter(|(now, _)| *now > then)
                .map(|(now, _)| (id.to_string(), then, now))
        })
        .collect()
}

/// How a moved belief is said: `…had no recorded belief when this ran, and is
/// now at version 1` or `…was at version 2 when this ran, and is now at 3`.
pub fn moved_words(then: u32, now: u32) -> String {
    if then == 0 {
        format!("had no recorded belief when this ran, and is now at version {now}")
    } else {
        format!("was at version {then} when this ran, and is now at {now}")
    }
}

// ---------------------------------------------------------------------------
// the CSV

fn field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\"").replace(['\n', '\r'], " "))
    } else {
        s.to_string()
    }
}

fn meta(s: &str) -> String {
    s.replace(['\n', '\r'], " ")
}

/// A result as the CSV it is kept and sent as.
pub fn csv(s: &Saved) -> String {
    let mut o = String::new();
    o.push_str(
        "# VLEO multipayload — a saved result: what one run returned, and the inputs it ran on.\n",
    );
    o.push_str(
        "# Open it in any spreadsheet. Upload it on the Results page to see it again without\n",
    );
    o.push_str("# running anything; its input rows can be loaded back as the case.\n");
    o.push_str(
        "# `value` is in `unit`; `si` is the same number in SI, which is what the tool reads.\n",
    );
    o.push_str(&format!("#! result {FORMAT}\n"));
    for (k, v) in [
        ("target", s.target.as_str()),
        ("mode", s.mode.as_str()),
        ("saved", s.saved.as_str()),
        ("name", s.name.as_str()),
        ("chain", s.chain.as_str()),
        ("kernel", s.kernel.as_str()),
        ("graph", s.graph.as_str()),
        ("design", s.design.as_str()),
        ("case", s.case.as_str()),
        ("template", s.template.as_str()),
    ] {
        o.push_str(&format!("#! {k} {}\n", meta(v)));
    }
    o.push_str(&format!("#! data {}\n", meta(&s.data.join(" "))));
    o.push_str(&format!(
        "#! ran {}\n#! blocked {}\n",
        s.ran, s.blocked_count
    ));
    if s.versions_known {
        o.push_str(&format!(
            "#! versions {}\n",
            if s.versions.is_empty() {
                // Said, not left out: none recorded is itself a record, and a
                // row's first version after this is a belief that moved.
                "none".to_string()
            } else {
                s.versions
                    .iter()
                    .map(|(id, n, r)| format!("{id}=v{n}@{r}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        ));
    }
    if !s.thinned.is_empty() {
        o.push_str(&format!("#! thinned {}\n", meta(&s.thinned)));
    }
    // `cred` is last: every column is read by its name, so a tool from before
    // it was added still reads a result that has it.
    o.push_str("section,id,name,value,unit,si,credibility,governing,note,cred\n");
    for (section, rows) in [
        ("input", &s.inputs),
        ("output", &s.outputs),
        ("blocked", &s.blocked),
    ] {
        for r in rows {
            o.push_str(&format!(
                "{section},{},{},{},{},{},{},{},{},{}\n",
                field(&r.id),
                field(&r.name),
                field(&r.value),
                field(&r.unit),
                r.si.map(num).unwrap_or_default(),
                field(&r.credibility),
                field(&r.governing),
                field(&r.note),
                field(&r.cred)
            ));
        }
    }
    o
}

/// Read a result back. Refuses what is not one, by what is missing.
pub fn read(text: &str) -> Result<Saved, Error> {
    // Said first, before any row is looked at: a case CSV or a spreadsheet of
    // something else would otherwise be refused for its columns, which tells a
    // person nothing about what they uploaded.
    if !text.lines().any(|l| {
        l.trim_start()
            .trim_start_matches('\u{feff}')
            .starts_with("#! result")
    }) {
        return Err(Error::new(
            ErrorKind::Malformed,
            "this is not a saved result: it has no `#! result` line. Save one from a run, or \
             with `vleo run <node> --save <file.csv>`",
        ));
    }
    let mut s = Saved::default();
    let mut format_ok = false;
    let mut header: Option<Vec<String>> = None;
    for (n, raw) in text.lines().enumerate() {
        let t = raw.trim().trim_start_matches('\u{feff}');
        if let Some(m) = t.strip_prefix("#!") {
            let m = m.trim();
            let (k, v) = m.split_once(' ').unwrap_or((m, ""));
            let v = v.trim().to_string();
            match k {
                "result" => {
                    if v != FORMAT {
                        return Err(Error::new(
                            ErrorKind::Malformed,
                            format!("this is a `{v}` result, and this tool reads `{FORMAT}`"),
                        ));
                    }
                    format_ok = true;
                }
                "target" => s.target = v,
                "mode" => s.mode = v,
                "saved" => s.saved = v,
                "name" => s.name = v,
                "chain" => s.chain = v,
                "kernel" => s.kernel = v,
                "graph" => s.graph = v,
                "design" => s.design = v,
                "case" => s.case = v,
                "template" => s.template = v,
                "data" => s.data = v.split_whitespace().map(str::to_string).collect(),
                "ran" => s.ran = v.parse().unwrap_or(0),
                "blocked" => s.blocked_count = v.parse().unwrap_or(0),
                "thinned" => s.thinned = v.clone(),
                "versions" => {
                    s.versions_known = true;
                    s.versions = v
                        .split_whitespace()
                        .filter_map(|x| {
                            let (id, rest) = x.split_once("=v")?;
                            let (n, rel) = rest.split_once('@')?;
                            Some((id.to_string(), n.parse().ok()?, rel.to_string()))
                        })
                        .collect()
                }
                _ => {}
            }
            continue;
        }
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let cells = crate::inputs::split_csv(t);
        let Some(h) = &header else {
            let h: Vec<String> = cells
                .iter()
                .map(|c| c.trim().to_ascii_lowercase())
                .collect();
            if !h.iter().any(|c| c == "section") || !h.iter().any(|c| c == "id") {
                return Err(Error::new(
                    ErrorKind::Malformed,
                    format!(
                        "line {}: the first row that is not a comment must name the columns — \
                     section, id, name, value, unit, si",
                        n + 1
                    ),
                ));
            }
            header = Some(h);
            continue;
        };
        let col = |name: &str| {
            h.iter()
                .position(|c| c == name)
                .and_then(|k| cells.get(k))
                .map(|x| x.trim().to_string())
                .unwrap_or_default()
        };
        let si = col("si");
        let row = Row {
            id: col("id"),
            name: col("name"),
            value: col("value"),
            unit: col("unit"),
            si: if si.is_empty() {
                None
            } else {
                Some(si.parse::<f64>().map_err(|_| {
                    Error::new(
                        ErrorKind::Malformed,
                        format!("line {}: si '{si}' is not a number", n + 1),
                    )
                })?)
            },
            credibility: col("credibility"),
            governing: col("governing"),
            note: col("note"),
            cred: col("cred"),
        };
        match col("section").as_str() {
            "input" => s.inputs.push(row),
            "output" => s.outputs.push(row),
            "blocked" => s.blocked.push(row),
            other => {
                return Err(Error::new(
                    ErrorKind::Malformed,
                    format!(
                        "line {}: section '{other}' is not input, output or blocked",
                        n + 1
                    ),
                ))
            }
        }
    }
    if !format_ok {
        return Err(Error::new(
            ErrorKind::Malformed,
            "this is not a saved result: it has no `#! result` line. Save one from a run, or \
             with `vleo run <node> --save <file.csv>`",
        ));
    }
    if s.target.is_empty() || header.is_none() {
        return Err(Error::new(
            ErrorKind::Malformed,
            "this result names no target, or has no rows",
        ));
    }
    Ok(s)
}

/// A folder name for a result: when, what, and which question — so two saves of
/// different questions never collide, one question is kept once, and a listing
/// sorts by time.
///
/// Every part is reduced to letters, digits, `_` and `-`: the target comes from
/// the file, and an uploaded file can say anything there. A `..` or a slash in
/// it once built a name that wrote outside the results folder.
pub fn folder_name(s: &Saved) -> String {
    let plain = |t: &str, keep: char| -> String {
        let p: String = t
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == keep {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        p.trim_matches('-').to_string()
    };
    format!(
        "{}_{}_{}",
        plain(&s.saved, '-'),
        plain(&s.target, '_'),
        s.question()
    )
}

/// A sweep as the CSV it is kept beside its result as: one row per point, in
/// the units a person reads and in SI, refused points kept with why.
pub fn sweep_csv(w: &Sweep) -> String {
    let mut o = String::new();
    o.push_str("# VLEO multipayload — a saved sweep: one answer across a range of one input.\n");
    o.push_str(
        "# `x` and `y` are in the units named below; `x_si` and `y_si` are what the tool reads.\n",
    );
    o.push_str(
        "# A refused point has no y, and says why: it is a gap in the line, never joined across.\n",
    );
    o.push_str(&format!("#! sweep {SWEEP_FORMAT}\n"));
    for (k, v) in [
        ("over", w.over.clone()),
        ("over_name", w.over_name.clone()),
        ("x_unit", w.x_unit.clone()),
        ("x_factor", num(w.x_factor)),
        ("y_unit", w.y_unit.clone()),
        ("y_factor", num(w.y_factor)),
        ("from", num(w.from)),
        ("to", num(w.to)),
        ("points", w.points.to_string()),
    ] {
        o.push_str(&format!("#! {k} {}\n", meta(&v)));
    }
    o.push_str("x,y,x_si,y_si,refused\n");
    let f = |v: f64, k: f64| num(if k != 0.0 { v / k } else { v });
    let mut pts: Vec<(f64, Option<f64>, &str)> =
        w.x.iter()
            .zip(&w.y)
            .map(|(x, y)| (*x, Some(*y), ""))
            .chain(w.refused.iter().map(|(x, why)| (*x, None, why.as_str())))
            .collect();
    pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
    for (x, y, why) in pts {
        o.push_str(&format!(
            "{},{},{},{},{}\n",
            f(x, w.x_factor),
            y.map(|y| f(y, w.y_factor)).unwrap_or_default(),
            num(x),
            y.map(num).unwrap_or_default(),
            field(why)
        ));
    }
    o
}

/// Read a sweep back. Refuses what is not one, by what is missing.
pub fn read_sweep(text: &str) -> Result<Sweep, Error> {
    // Said first, before any row is read: a file of something else would
    // otherwise be refused for a column, which says nothing about what it is.
    if !text.lines().any(|l| {
        l.trim_start()
            .trim_start_matches('\u{feff}')
            .starts_with("#! sweep")
    }) {
        return Err(Error::new(
            ErrorKind::Malformed,
            "this is not a saved sweep: it has no `#! sweep` line",
        ));
    }
    let mut w = Sweep::default();
    let mut format_ok = false;
    let mut header: Option<Vec<String>> = None;
    let number = |k: &str, v: &str| {
        v.parse::<f64>().map_err(|_| {
            Error::new(
                ErrorKind::Malformed,
                format!("sweep: `{k}` is '{v}', which is not a number"),
            )
        })
    };
    for (n, raw) in text.lines().enumerate() {
        let t = raw.trim().trim_start_matches('\u{feff}');
        if let Some(m) = t.strip_prefix("#!") {
            let m = m.trim();
            let (k, v) = m.split_once(' ').unwrap_or((m, ""));
            let v = v.trim();
            match k {
                "sweep" => {
                    if v != SWEEP_FORMAT {
                        return Err(Error::new(
                            ErrorKind::Malformed,
                            format!("this is a `{v}` sweep, and this tool reads `{SWEEP_FORMAT}`"),
                        ));
                    }
                    format_ok = true;
                }
                "over" => w.over = v.to_string(),
                "over_name" => w.over_name = v.to_string(),
                "x_unit" => w.x_unit = v.to_string(),
                "y_unit" => w.y_unit = v.to_string(),
                "x_factor" => w.x_factor = number(k, v)?,
                "y_factor" => w.y_factor = number(k, v)?,
                "from" => w.from = number(k, v)?,
                "to" => w.to = number(k, v)?,
                "points" => {
                    w.points = v.parse().map_err(|_| {
                        Error::new(
                            ErrorKind::Malformed,
                            format!("sweep: `points` is '{v}', which is not a count"),
                        )
                    })?
                }
                _ => {}
            }
            continue;
        }
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let cells = crate::inputs::split_csv(t);
        let Some(h) = &header else {
            header = Some(
                cells
                    .iter()
                    .map(|c| c.trim().to_ascii_lowercase())
                    .collect(),
            );
            continue;
        };
        let col = |name: &str| {
            h.iter()
                .position(|c| c == name)
                .and_then(|k| cells.get(k))
                .map(|x| x.trim().to_string())
                .unwrap_or_default()
        };
        let x = number(&format!("line {} x_si", n + 1), &col("x_si"))?;
        let y = col("y_si");
        if y.is_empty() {
            w.refused.push((x, col("refused")));
        } else {
            w.x.push(x);
            w.y.push(number(&format!("line {} y_si", n + 1), &y)?);
        }
    }
    if !format_ok {
        return Err(Error::new(
            ErrorKind::Malformed,
            "this is not a saved sweep: it has no `#! sweep` line",
        ));
    }
    if w.over.is_empty() || w.points == 0 {
        return Err(Error::new(
            ErrorKind::Malformed,
            "this sweep names no input it moved, or no points",
        ));
    }
    Ok(w)
}

// ---------------------------------------------------------------------------
// the report

#[cfg(feature = "std")]
fn he(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(feature = "std")]
/// The same result as one HTML page that shows itself anywhere, for sending to
/// somebody who does not run the tool. The CSV rides inside it, so the page can
/// be uploaded in its place.
pub fn html(s: &Saved) -> String {
    let data = csv(s).replace("</", "<\\/");
    let answer = s
        .answer()
        .map(|a| {
            format!(
                "{} {}",
                he(&a.value),
                he(if a.unit == "-" { "" } else { &a.unit })
            )
        })
        .unwrap_or_else(|| t("answer-none").to_string());
    let mut o = String::with_capacity(64 * 1024);
    // ANSWER FIRST (docs/EXPLAINING.md E1): the number, then the three things
    // a reader needs before trusting it — how credible it is and what holds it
    // down, what could not run, and what it was run on. Then it said simply,
    // then where it breaks, then the reference tables.
    let target = s.answer();
    let label = target.map(|a| a.name.as_str()).unwrap_or(s.target.as_str());
    let changed_inputs: Vec<&Row> = s.inputs.iter().filter(|r| r.note == "changed").collect();
    let which = if s.blocked.is_empty() {
        String::new()
    } else {
        let ids = s
            .blocked
            .iter()
            .map(|b| b.id.as_str())
            .take(4)
            .collect::<Vec<_>>()
            .join(", ");
        f("ran-which", &[("ids", &he(&ids))])
    };
    let ran_line = f(
        "ran",
        &[
            ("ran", &s.ran.to_string()),
            ("blocked", &s.blocked_count.to_string()),
            ("which", &which),
        ],
    );
    let moved = moved_since(s);
    let cred = match target {
        Some(a) => f(
            "cred",
            &[
                ("cred", &he(&a.credibility)),
                ("governing", &he(&a.governing)),
            ],
        ),
        None => t("cred-none").to_string(),
    };
    let inputs = if changed_inputs.is_empty() {
        f("inputs-default", &[("n", &s.inputs.len().to_string())])
    } else {
        let list = changed_inputs
            .iter()
            .take(4)
            .map(|r| {
                format!(
                    "{} = {} {}",
                    r.id,
                    r.value,
                    if r.unit == "-" { "" } else { &r.unit }
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        f(
            "inputs-changed",
            &[
                ("k", &changed_inputs.len().to_string()),
                ("n", &s.inputs.len().to_string()),
                ("list", &he(&list)),
            ],
        )
    };
    let rests = if !moved.is_empty() {
        if moved.len() == 1 {
            t("rests-moved-1").to_string()
        } else {
            f("rests-moved-n", &[("n", &moved.len().to_string())])
        }
    } else if s.versions.is_empty() {
        t("rests-none").to_string()
    } else if s.versions.len() == 1 {
        t("rests-versions-1").to_string()
    } else {
        f("rests-versions-n", &[("n", &s.versions.len().to_string())])
    };
    let name = if s.name.is_empty() {
        String::new()
    } else {
        f("name", &[("name", &he(&s.name))])
    };
    let data_line = if s.data.is_empty() {
        String::new()
    } else {
        f("data", &[("data", &he(&s.data.join(", ")))])
    };
    o.push_str(&f(
        "header",
        &[
            ("target", &he(&s.target)),
            ("answer", &answer),
            ("cred", &cred),
            ("ran", &ran_line),
            ("inputs", &inputs),
            ("rests", &rests),
            ("name", &name),
            ("saved", &he(&s.saved)),
            ("mode", &he(&s.mode)),
            ("chain", &he(&s.chain)),
            ("kernel", &he(&s.kernel)),
            ("graph", &he(&s.graph)),
            ("data", &data_line),
        ],
    ));
    // Thinned, it says so before anything else is read from it: the values
    // not shown were let go, not zero and not lost by accident.
    if let Some((on, n)) = s.thinned.split_once(' ') {
        o.push_str(&if n == "1" {
            f("thinned-1", &[("on", &he(on))])
        } else {
            f("thinned-n", &[("on", &he(on)), ("n", &he(n))])
        });
    }
    if let Some(w) = &s.sweep {
        o.push_str(&sweep_section(s, w));
    }
    let whose = if changed_inputs.is_empty() {
        "simply-declared"
    } else {
        "simply-case"
    };
    o.push_str(&f(
        "simply",
        &[
            ("which", t(whose)),
            ("label", &he(label)),
            ("answer", &answer),
        ],
    ));
    o.push_str(t("breaks-open"));
    if let Some(a) = target {
        o.push_str(&f("breaks-weakest", &[("governing", &he(&a.governing))]));
    }
    if s.blocked.is_empty() {
        o.push_str(t("breaks-none-blocked"));
    } else if s.blocked.len() == 1 {
        o.push_str(t("breaks-blocked-1"));
    } else {
        o.push_str(&f(
            "breaks-blocked-n",
            &[("n", &s.blocked.len().to_string())],
        ));
    }
    for (id, then, now) in &moved {
        o.push_str(&f(
            "breaks-moved",
            &[("id", &he(id)), ("words", &moved_words(*then, *now))],
        ));
    }
    if !s.versions.is_empty() {
        let list = s
            .versions
            .iter()
            .map(|(id, n, r)| format!("{id} v{n} ({r})"))
            .collect::<Vec<_>>()
            .join(", ");
        o.push_str(&f("breaks-versions", &[("list", &he(&list))]));
    }
    o.push_str(t("breaks-close"));
    // A reference table: its title and columns are parts named for it.
    let table = |which: &str, rows: &[&Row], cells: &dyn Fn(&Row) -> Vec<String>| {
        let mut o = f("table-open", &[("title", t(&format!("title-{which}")))]);
        if rows.is_empty() {
            o.push_str(t("table-none"));
            return o;
        }
        o.push_str(t("table-head-open"));
        for c in t(&format!("cols-{which}")).split('|') {
            o.push_str(&f("th", &[("text", c)]));
        }
        o.push_str(t("table-head-close"));
        for r in rows {
            o.push_str(t("tr-open"));
            for c in cells(r) {
                o.push_str(&f("td", &[("text", &c)]));
            }
            o.push_str(t("tr-close"));
        }
        o.push_str(t("table-close"));
        o
    };
    let unit = |u: &str| he(if u == "-" { "" } else { u });
    let code = |s: &str| f("code", &[("text", &he(s))]);
    let value_cells = |r: &Row| {
        vec![
            code(&r.id),
            he(&r.name),
            format!("{} {}", he(&r.value), unit(&r.unit)),
            he(&r.credibility),
            he(&r.governing),
        ]
    };
    o.push_str(&table("changed", &changed_inputs, &|r| {
        vec![
            code(&r.id),
            he(&r.name),
            format!("{} {}", he(&r.value), unit(&r.unit)),
        ]
    }));
    let outs: Vec<&Row> = s.outputs.iter().collect();
    o.push_str(&table("values", &outs, &value_cells));
    let blocked: Vec<&Row> = s.blocked.iter().collect();
    o.push_str(&table("blocked", &blocked, &|r| {
        vec![code(&r.id), he(&r.name), he(&r.note)]
    }));
    let all: Vec<&Row> = s.inputs.iter().collect();
    o.push_str(t("details-open"));
    o.push_str(&table("all", &all, &|r| {
        vec![
            code(&r.id),
            he(&r.name),
            format!("{} {}", he(&r.value), unit(&r.unit)),
            he(&r.note),
        ]
    }));
    o.push_str(t("details-close"));
    o.push_str(&f("result-block", &[("csv", &data)]));
    if let Some(w) = &s.sweep {
        o.push_str(&f(
            "sweep-block",
            &[("csv", &sweep_csv(w).replace("</", "<\\/"))],
        ));
    }
    vleo_sheet::shell::fill(&vleo_sheet::shell::Page {
        title: &f("title", &[("target", &s.target)]),
        head: &f("head", &[("css", REPORT_CSS)]),
        body: o.trim_end(),
        ..Default::default()
    })
}

#[cfg(feature = "std")]
/// The report's parts (`web/pages/report.html`), read once.
fn parts() -> &'static vleo_sheet::shell::Parts {
    static P: std::sync::OnceLock<vleo_sheet::shell::Parts> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        vleo_sheet::shell::Parts::parse(
            "web/pages/report.html",
            include_str!("../../../web/pages/report.html"),
        )
        .unwrap_or_else(|e| panic!("{e}"))
    })
}

#[cfg(feature = "std")]
/// A part as written.
fn t(name: &str) -> &'static str {
    parts().text(name)
}

#[cfg(feature = "std")]
/// A part with its slots filled.
fn f(name: &str, slots: &[(&str, &str)]) -> String {
    parts().fill(name, slots)
}

#[cfg(feature = "std")]
/// The sweep, drawn: the engine's points and nothing between them, broken
/// wherever a point was refused.
fn sweep_section(s: &Saved, w: &Sweep) -> String {
    let fx = if w.x_factor != 0.0 { w.x_factor } else { 1.0 };
    let fy = if w.y_factor != 0.0 { w.y_factor } else { 1.0 };
    let xs: Vec<f64> = w.x.iter().map(|x| x / fx).collect();
    let ys: Vec<f64> = w.y.iter().map(|y| y / fy).collect();
    let unit = |u: &str| {
        if u == "-" || u.is_empty() {
            String::new()
        } else {
            format!(" {u}")
        }
    };
    let over_name = if w.over_name.is_empty() {
        w.over.as_str()
    } else {
        w.over_name.as_str()
    };
    let refused = if w.refused.is_empty() {
        String::new()
    } else {
        f(
            "sweep-refused",
            &[
                ("n", &w.refused.len().to_string()),
                ("why", &he(&w.refused[0].1)),
            ],
        )
    };
    let mut o = f(
        "sweep-open",
        &[
            ("target", &he(&s.target)),
            ("over", &he(over_name)),
            ("from", &num(w.from / fx)),
            ("xu", &he(&unit(&w.x_unit))),
            ("to", &num(w.to / fx)),
            ("points", &w.points.to_string()),
            ("answered", &w.x.len().to_string()),
            ("refused", &refused),
        ],
    );
    if xs.is_empty() {
        o.push_str(t("sweep-empty"));
        return o;
    }
    let span = |v: &[f64]| {
        let lo = v.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if hi > lo {
            (lo, hi)
        } else {
            (lo - 0.5 * lo.abs().max(1.0), hi + 0.5 * hi.abs().max(1.0))
        }
    };
    let xr = {
        let (a, b) = (w.from / fx, w.to / fx);
        if a < b {
            (a, b)
        } else if b < a {
            (b, a)
        } else {
            span(&xs)
        }
    };
    let (y0, y1) = span(&ys);
    let pad = (y1 - y0) * 0.06;
    let yr = (y0 - pad, y1 + pad);
    let (w_px, h_px, l, r, top, b) = (720.0, 320.0, 72.0, 16.0, 12.0, 44.0);
    let px = |x: f64| l + (x - xr.0) / (xr.1 - xr.0) * (w_px - l - r);
    let py = |y: f64| h_px - b - (y - yr.0) / (yr.1 - yr.0) * (h_px - top - b);
    let d1 = |v: f64| format!("{v:.1}");
    o.push_str(&f(
        "svg-open",
        &[
            ("w", &w_px.to_string()),
            ("h", &h_px.to_string()),
            ("target", &he(&s.target)),
            ("over", &he(&w.over)),
        ],
    ));
    for k in 0..=4 {
        let fy_ = yr.0 + (yr.1 - yr.0) * k as f64 / 4.0;
        let fx_ = xr.0 + (xr.1 - xr.0) * k as f64 / 4.0;
        o.push_str(&f(
            "svg-grid",
            &[
                ("l", &l.to_string()),
                ("x2", &(w_px - r).to_string()),
                ("y", &d1(py(fy_))),
                ("tx", &(l - 6.0).to_string()),
                ("ty", &d1(py(fy_) + 4.0)),
                ("yl", &num(round4(fy_))),
                ("x", &d1(px(fx_))),
                ("by", &(h_px - b + 16.0).to_string()),
                ("xl", &num(round4(fx_))),
            ],
        ));
    }
    // One path, broken wherever a point was refused.
    let mut pts: Vec<(f64, Option<f64>)> =
        xs.iter().zip(&ys).map(|(x, y)| (*x, Some(*y))).collect();
    pts.extend(w.refused.iter().map(|(x, _)| (x / fx, None)));
    pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
    let mut d = String::new();
    let mut pen = false;
    for (x, y) in pts {
        match y {
            Some(y) => {
                d.push_str(&format!(
                    "{}{:.1},{:.1} ",
                    if pen { "L" } else { "M" },
                    px(x),
                    py(y)
                ));
                pen = true;
            }
            None => pen = false,
        }
    }
    o.push_str(&f("svg-path", &[("d", d.trim_end())]));
    // Where this case sits on the line: the swept input's own value, and the
    // answer the run returned there — not read off the curve.
    let at = s.inputs.iter().find(|i| i.id == w.over).and_then(|i| i.si);
    if let (Some(x), Some(y)) = (at, s.answer().and_then(|a| a.si)) {
        let (x, y) = (x / fx, y / fy);
        if x >= xr.0 && x <= xr.1 && y >= yr.0 && y <= yr.1 {
            o.push_str(&f(
                "svg-here",
                &[
                    ("x", &d1(px(x))),
                    ("y", &d1(py(y))),
                    ("tx", &d1(px(x) + 8.0)),
                    ("ty", &d1(py(y) - 8.0)),
                ],
            ));
        }
    }
    let axis = |u: &str| {
        if u.is_empty() || u == "-" {
            t("dimensionless").to_string()
        } else {
            he(u)
        }
    };
    o.push_str(&f(
        "svg-axes",
        &[
            ("x", &((l + w_px - r) / 2.0).to_string()),
            ("y", &(h_px - 6.0).to_string()),
            ("over", &he(&w.over)),
            ("xu", &axis(&w.x_unit)),
            ("my", &((top + h_px - b) / 2.0).to_string()),
            ("target", &he(&s.target)),
            ("yu", &axis(&w.y_unit)),
        ],
    ));
    o.push_str(t("sweep-close"));
    o
}

#[cfg(feature = "std")]
/// A tick label, to four significant figures.
fn round4(v: f64) -> f64 {
    if v == 0.0 || !v.is_finite() {
        return v;
    }
    use vleo_units::pmath;
    let m = pmath::powi(10.0, 3 - pmath::floor(pmath::log10(v.abs())) as i32);
    pmath::round(v * m) / m
}

/// The sweep inside a report page, when it carries one.
pub fn unwrap_sweep(text: &str) -> Option<String> {
    let at = text.find("id=\"vleo-sweep\"")?;
    let open = text[at..].find('>').map(|k| at + k + 1)?;
    let close = text[open..].find("</script>").map(|k| open + k)?;
    Some(text[open..close].replace("<\\/", "</"))
}

/// The CSV inside a report page, or the text itself when it is not one.
pub fn unwrap_report(text: &str) -> String {
    let Some(at) = text.find("id=\"vleo-result\"") else {
        return text.to_string();
    };
    let Some(open) = text[at..].find('>').map(|k| at + k + 1) else {
        return text.to_string();
    };
    let close = text[open..]
        .find("</script>")
        .map(|k| open + k)
        .unwrap_or(text.len());
    text[open..close].replace("<\\/", "</")
}

#[cfg(feature = "std")]
const REPORT_CSS: &str = include_str!("../../../web/pages/report.css");

// ---------------------------------------------------------------------------
// kept on disk

/// Results kept in a directory outside the repository.
///
/// A result is a folder: `result.csv` (the values and the inputs they ran
/// on), `sweep.csv` when it is a sweep, and `report.html` — the page to
/// send, which carries both and uploads back whole. A folder is written
/// aside and renamed into place, so another laptop or a sync client never
/// lists a result half made. A result saved as one `.csv` before results
/// were folders is still listed, opened and removed.
#[cfg(feature = "std")]
pub mod store {
    use super::*;
    use std::path::Path;

    /// The files in a result's folder.
    pub const RESULT: &str = "result.csv";
    pub const SWEEP: &str = "sweep.csv";
    pub const REPORT: &str = "report.html";
    /// Present when the result is pinned: kept whole, and never thinned.
    pub const PINNED: &str = "pinned";
    /// The index of a results folder: one line per result, rebuilt from the
    /// folders whenever it is out of date, so it is never the only copy of
    /// anything and another laptop's results are in it as soon as they land.
    pub const INDEX: &str = ".index.tsv";

    /// What a directory holds: each result by its name, and each entry that
    /// does not read as one, with why.
    pub type Listing = (Vec<(String, Saved)>, Vec<(String, String)>);

    /// Every result in the directory, newest first, with any that no longer
    /// read named rather than skipped.
    pub fn list(dir: &Path) -> Listing {
        let mut good = Vec::new();
        let mut bad = Vec::new();
        let Ok(rd) = std::fs::read_dir(dir) else {
            return (good, bad);
        };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            // A folder still being written, or anything else hidden.
            if name.starts_with('.') {
                continue;
            }
            let is_dir = e.path().is_dir();
            if !is_dir && !name.ends_with(".csv") {
                continue;
            }
            match open_path(&e.path(), is_dir) {
                Ok(s) => good.push((name, s)),
                Err(why) => bad.push((name, why.into())),
            }
        }
        good.sort_by(|a, b| b.0.cmp(&a.0));
        (good, bad)
    }

    /// One result, as the index holds it.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Entry {
        pub name: String,
        pub question: String,
        pub saved: String,
        pub target: String,
        pub pinned: bool,
        pub thinned: bool,
    }

    /// Every result in the folder, from the index — rebuilt first when the
    /// folder holds anything the index does not, lacks anything it names, or
    /// has changed since it was written. Reading every result is what the
    /// index saves: finding a question reads one small file.
    pub fn index(dir: &Path) -> Vec<Entry> {
        let now = entries(dir);
        let at = dir.join(INDEX);
        if let (Ok(text), Ok(written)) = (
            std::fs::read_to_string(&at),
            std::fs::metadata(&at).and_then(|m| m.modified()),
        ) {
            let held: Vec<Entry> = text.lines().filter_map(entry_from_line).collect();
            let same_names = held.len() == now.len()
                && held.iter().all(|e| now.iter().any(|(n, _)| *n == e.name));
            // Strictly earlier: a change in the same tick as the index is a
            // change, on a filesystem whose clock is coarse.
            let unchanged = now.iter().all(|(_, m)| *m < written);
            if same_names && unchanged {
                // A pin is one file's presence; read live, never from memory.
                return held
                    .into_iter()
                    .map(|e| Entry {
                        pinned: is_pinned(dir, &e.name),
                        ..e
                    })
                    .collect();
            }
        }
        let fresh: Vec<Entry> = list(dir)
            .0
            .into_iter()
            .map(|(name, s)| Entry {
                pinned: dir.join(&name).join(PINNED).is_file(),
                thinned: !s.thinned.is_empty(),
                question: s.question(),
                saved: s.saved.clone(),
                target: s.target.clone(),
                name,
            })
            .collect();
        let mut text = String::from("# name\tquestion\tsaved\ttarget\tpinned\tthinned — rebuilt from the folders; safe to delete\n");
        for e in &fresh {
            text.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\n",
                e.name, e.question, e.saved, e.target, e.pinned as u8, e.thinned as u8
            ));
        }
        // An index that cannot be written is only slower, never wrong.
        let _ = vleo_data::write_whole(&at, text);
        fresh
    }

    fn entry_from_line(l: &str) -> Option<Entry> {
        if l.starts_with('#') {
            return None;
        }
        let c: Vec<&str> = l.split('\t').collect();
        (c.len() == 6).then(|| Entry {
            name: c[0].into(),
            question: c[1].into(),
            saved: c[2].into(),
            target: c[3].into(),
            pinned: c[4] == "1",
            thinned: c[5] == "1",
        })
    }

    /// The results in a folder, each with when it last changed.
    fn entries(dir: &Path) -> Vec<(String, std::time::SystemTime)> {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        rd.flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let p = e.path();
                if name.starts_with('.') || !(p.is_dir() || name.ends_with(".csv")) {
                    return None;
                }
                // A folder's time moves when a file is added to it or removed
                // (a pin); its result.csv's when the result is rewritten (a
                // thinning). The later of the two.
                let m = |q: &Path| std::fs::metadata(q).and_then(|m| m.modified()).ok();
                let t = [m(&p), m(&p.join(RESULT))].into_iter().flatten().max()?;
                Some((name, t))
            })
            .collect()
    }

    /// The result that already answers this question, if one is kept whole —
    /// the newest, when more than one is. A thinned result is a summary, not
    /// the answer, so the question is asked again.
    pub fn find(dir: &Path, question: &str) -> Option<(String, Saved)> {
        let mut hits: Vec<Entry> = index(dir)
            .into_iter()
            .filter(|e| e.question == question && !e.thinned)
            .collect();
        hits.sort_by(|a, b| b.name.cmp(&a.name));
        hits.into_iter()
            .find_map(|e| open(dir, &e.name).ok().map(|s| (e.name, s)))
    }

    /// Pin a result, or unpin it: a pinned result is kept whole for good.
    pub fn pin(dir: &Path, name: &str, on: bool) -> Result<(), Error> {
        if !is_plain(name) || !dir.join(name).join(RESULT).is_file() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "'{name}' is not a result's folder; a result kept as one .csv before \
                 results were folders cannot be pinned — upload it again to make it one"
                ),
            ));
        }
        let p = dir.join(name).join(PINNED);
        let _ = std::fs::remove_file(dir.join(INDEX));
        if on {
            vleo_data::write_whole(&p, "kept whole: never thinned\n")
                .map_err(|e| Error::io(name, e))
        } else if p.exists() {
            std::fs::remove_file(&p).map_err(|e| Error::io(name, e))
        } else {
            Ok(())
        }
    }

    /// Whether a result is pinned.
    pub fn is_pinned(dir: &Path, name: &str) -> bool {
        is_plain(name) && dir.join(name).join(PINNED).is_file()
    }

    /// Thin every unpinned result saved more than `keep_days` before `now`
    /// (Unix seconds) to its summary — see [`super::thin`]. Returns the names
    /// thinned. A pinned result, one already thinned, one kept as one .csv, and
    /// one whose date does not read are left as they are.
    pub fn thin_old(dir: &Path, now: i64, keep_days: u32) -> Result<Vec<String>, Error> {
        let today = vleo_units::calendar::Civil::from_unix(now)
            .date()
            .to_string();
        let mut done = Vec::new();
        for e in index(dir) {
            if e.pinned || e.thinned || !dir.join(&e.name).is_dir() {
                continue;
            }
            let Some(saved) = vleo_units::calendar::Civil::parse(&e.saved) else {
                continue;
            };
            if now - saved.to_unix() <= i64::from(keep_days) * 86_400 {
                continue;
            }
            let s = open(dir, &e.name)?;
            let t = super::thin(&s, &today);
            let folder = dir.join(&e.name);
            vleo_data::write_whole(&folder.join(RESULT), csv(&t))
                .map_err(|x| Error::io(&e.name, x))?;
            vleo_data::write_whole(&folder.join(REPORT), html(&t))
                .map_err(|x| Error::io(&e.name, x))?;
            done.push(e.name);
        }
        if !done.is_empty() {
            let _ = std::fs::remove_file(dir.join(INDEX));
        }
        Ok(done)
    }

    /// Keep a result, once. Returns the name it is kept under and whether it
    /// was already kept: the same question saved twice is one result, because
    /// the same engine on the same inputs gives the same answer.
    pub fn save(dir: &Path, s: &Saved) -> Result<(String, bool), Error> {
        if let Some((name, _)) = find(dir, &s.question()) {
            return Ok((name, true));
        }
        std::fs::create_dir_all(dir).map_err(|e| Error::io(dir.display(), e))?;
        let name = folder_name(s);
        if !is_plain(&name) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("'{name}' is not a result's name"),
            ));
        }
        let at = dir.join(&name);
        if at.exists() {
            return Ok((name, true));
        }
        // Whole: made aside under a hidden name, then renamed into place in
        // one step. A results folder may be shared or synced.
        let aside = dir.join(format!(".{name}.part-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&aside);
        let made = (|| -> Result<(), Error> {
            std::fs::create_dir_all(&aside).map_err(|e| Error::io(aside.display(), e))?;
            vleo_data::write_whole(&aside.join(RESULT), csv(s))
                .map_err(|e| Error::io(RESULT, e))?;
            if let Some(w) = &s.sweep {
                vleo_data::write_whole(&aside.join(SWEEP), sweep_csv(w))
                    .map_err(|e| Error::io(SWEEP, e))?;
            }
            vleo_data::write_whole(&aside.join(REPORT), html(s))
                .map_err(|e| Error::io(REPORT, e))?;
            std::fs::rename(&aside, &at).map_err(|e| Error::io(&name, e))
        })();
        if let Err(e) = made {
            let _ = std::fs::remove_dir_all(&aside);
            return Err(e);
        }
        Ok((name, false))
    }

    /// One result by its name. The name must be a plain name in the directory:
    /// a path read from a request never reaches anywhere else.
    pub fn open(dir: &Path, name: &str) -> Result<Saved, Error> {
        if !is_plain(name) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("'{name}' is not a result's name"),
            ));
        }
        let p = dir.join(name);
        open_path(&p, p.is_dir()).map_err(|e| e.within(name))
    }

    /// Remove one result — its folder, or the one file an older result is.
    pub fn remove(dir: &Path, name: &str) -> Result<(), Error> {
        if !is_plain(name) {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("'{name}' is not a result's name"),
            ));
        }
        let p = dir.join(name);
        if p.is_dir() {
            // Only a folder that is a result: the name came from a request.
            if !p.join(RESULT).is_file() {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("{name}: not a result's folder, so it was left alone"),
                ));
            }
            std::fs::remove_dir_all(&p).map_err(|e| Error::io(name, e))
        } else {
            std::fs::remove_file(&p).map_err(|e| Error::io(name, e))
        }
    }

    fn open_path(p: &Path, is_dir: bool) -> Result<Saved, Error> {
        if !is_dir {
            let text = std::fs::read_to_string(p).map_err(Error::from_io)?;
            return read(&text);
        }
        let text = std::fs::read_to_string(p.join(RESULT)).map_err(|e| Error::io(RESULT, e))?;
        let mut s = read(&text)?;
        if let Ok(t) = std::fs::read_to_string(p.join(SWEEP)) {
            s.sweep = Some(read_sweep(&t).map_err(|e| e.within(SWEEP))?);
        }
        Ok(s)
    }

    fn is_plain(name: &str) -> bool {
        !name.is_empty()
            && !name.starts_with('.')
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
            && !name.contains("..")
    }
}
