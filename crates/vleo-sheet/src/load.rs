//! Reading the tree off disk.

use crate::fnv1a;
use crate::model::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The whole repository, loaded. Nodes are held in a sorted map so that every
/// generator iterates in the same order on every machine — a generator whose
/// output depends on directory order fails its own regeneration check at
/// random, and within a fortnight nobody reads that check.
#[derive(Default)]
pub struct Tree {
    pub root: PathBuf,
    pub sheets: BTreeMap<String, Sheet>,
    pub groups: BTreeMap<String, Group>,
    pub relations: Vec<Relation>,
    pub cases: BTreeMap<String, Case>,
    pub sources: BTreeMap<String, Source>,
    /// The cycles the ARCHITECTURE declares, read once from `layers/`. Every
    /// case inherits these; a case's own `[[iterate]]` is added to them.
    pub cycles: Vec<CycleSpec>,
}

// A VALUE OF THE WRONG TYPE IS REFUSED, NOT READ AS EMPTY.
//
// These getters used to turn a number typed as "12,5", or a string where a
// number belongs, into 0.0 or "" — a sheet that loaded, generated and ran on a
// value its author never wrote. An absent key is still empty (most keys are
// optional); a PRESENT key of the wrong type is recorded here, and the loader
// refuses the file with every such value named.
thread_local! {
    static WRONG: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn wrong(expected: &str, v: &toml::Value) {
    let shown: String = v.to_string().chars().take(60).collect();
    WRONG.with(|w| {
        w.borrow_mut()
            .push(format!("expected {expected}, found {shown}"))
    });
}

/// Start collecting for one file.
fn begin() {
    WRONG.with(|w| w.borrow_mut().clear());
}

/// Stop collecting; any value of the wrong type refuses the file.
fn finish(path: &Path) -> Result<(), String> {
    let found = WRONG.with(|w| std::mem::take(&mut *w.borrow_mut()));
    if found.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{}: {} value(s) of the wrong type: {}",
            path.display(),
            found.len(),
            found.join("; ")
        ))
    }
}

// A KEY NOBODY READS IS REFUSED, NOT IGNORED.
//
// A misspelt key (`quesiton`, `tolerence`) was skipped without a word, and the
// field it meant to set kept its default. The keys this loader reads are the
// sheet's schema, taken from this file itself: every `.get("…")` below is one,
// so a key added to the loader is known the moment it is read and never has to
// be listed a second time. Tables whose keys are DATA (a fixture's or a case's
// input values, keyed by binding) are not checked.
fn known_keys() -> &'static std::collections::BTreeSet<&'static str> {
    static KNOWN: std::sync::OnceLock<std::collections::BTreeSet<&'static str>> =
        std::sync::OnceLock::new();
    KNOWN.get_or_init(|| {
        let src: &'static str = include_str!("load.rs");
        let mut k = std::collections::BTreeSet::new();
        let mut rest = src;
        while let Some(i) = rest.find(".get(\"") {
            rest = &rest[i + 6..];
            if let Some(j) = rest.find('"') {
                let key = &rest[..j];
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    k.insert(key);
                }
            }
        }
        k
    })
}

const DATA_TABLES: &[&str] = &["inputs", "supply"];

fn unknown_keys(v: &toml::Value, at: &str, out: &mut Vec<String>) {
    match v {
        toml::Value::Table(t) => {
            for (k, x) in t {
                let here = if at.is_empty() {
                    k.clone()
                } else {
                    format!("{at}.{k}")
                };
                if !known_keys().contains(k.as_str()) {
                    out.push(here.clone());
                }
                if !DATA_TABLES.contains(&k.as_str()) {
                    unknown_keys(x, &here, out);
                }
            }
        }
        toml::Value::Array(a) => {
            for x in a {
                unknown_keys(x, at, out);
            }
        }
        _ => {}
    }
}

/// Refuse a file carrying a key the loader never reads.
fn refuse_unknown(v: &toml::Value, path: &Path) -> Result<(), String> {
    let mut out = Vec::new();
    unknown_keys(v, "", &mut out);
    if out.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{}: key(s) this tool does not read, probably misspelt: {}",
            path.display(),
            out.join(", ")
        ))
    }
}

fn s(v: Option<&toml::Value>) -> String {
    match v {
        None => String::new(),
        Some(toml::Value::String(x)) => x.clone(),
        Some(other) => {
            wrong("text", other);
            String::new()
        }
    }
}

/// Code as the sheet holds it: exactly as written, less the line break a
/// multi-line string ends with.
fn code(v: Option<&toml::Value>) -> String {
    s(v).trim_end_matches(['\n', '\r']).to_string()
}

fn f(v: Option<&toml::Value>) -> f64 {
    match v {
        None => 0.0,
        Some(toml::Value::Float(x)) => *x,
        Some(toml::Value::Integer(i)) => *i as f64,
        Some(other) => {
            wrong("a number", other);
            0.0
        }
    }
}

fn u(v: Option<&toml::Value>) -> u32 {
    match v {
        None => 0,
        Some(toml::Value::Integer(i)) if *i >= 0 && *i <= u32::MAX as i64 => *i as u32,
        Some(other) => {
            wrong("a whole number from 0", other);
            0
        }
    }
}

pub fn load_all(root: &Path) -> Result<Tree, String> {
    let mut tree = Tree {
        root: root.to_path_buf(),
        ..Default::default()
    };
    let crates_dir = root.join("crates");
    let mut crate_dirs: Vec<PathBuf> = fs::read_dir(&crates_dir)
        .map_err(|e| format!("crates/: {e}"))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("vleo-mod-"))
                .unwrap_or(false)
        })
        .collect();
    crate_dirs.sort();
    for cd in crate_dirs {
        let crate_name = cd.file_name().unwrap().to_str().unwrap().to_string();
        let nodes_dir = cd.join("nodes");
        if !nodes_dir.is_dir() {
            continue;
        }
        let mut node_dirs: Vec<PathBuf> = fs::read_dir(&nodes_dir)
            .map_err(|e| format!("{}: {e}", nodes_dir.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        node_dirs.sort();
        for nd in node_dirs {
            let sheet = load_sheet(&nd, &crate_name)?;
            if tree.sheets.contains_key(&sheet.id) {
                return Err(format!(
                    "two nodes claim the identifier '{}' — the second is {}",
                    sheet.id,
                    nd.display()
                ));
            }
            tree.sheets.insert(sheet.id.clone(), sheet);
        }
    }
    load_layers(&mut tree)?;
    load_cases(&mut tree)?;
    load_sources(&mut tree)?;
    Ok(tree)
}

fn load_sheet(dir: &Path, crate_name: &str) -> Result<Sheet, String> {
    let path = dir.join("node.toml");
    begin();
    let sh = load_sheet_values(dir, crate_name, &path)?;
    finish(&path)?;
    Ok(sh)
}

fn load_sheet_values(dir: &Path, crate_name: &str, path: &Path) -> Result<Sheet, String> {
    let path = path.to_path_buf();
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: toml::Value = text
        .parse()
        .map_err(|e| format!("{}: malformed sheet: {e}", path.display()))?;
    refuse_unknown(&v, &path)?;
    let t = v
        .as_table()
        .ok_or_else(|| format!("{}: not a table", path.display()))?;

    let mut sh = Sheet {
        id: s(t.get("id")),
        label: s(t.get("label")),
        folder: s(t.get("folder")),
        subsystem: s(t.get("subsystem")),
        parent: s(t.get("parent")),
        kind: s(t.get("kind")),
        owner: s(t.get("owner")),
        tier: s(t.get("tier")),
        // Minor unless somebody said otherwise. A default of "significant"
        // would put every node through two reviewers and a differential fill,
        // which is how a gate becomes a queue.
        criticality: {
            let c = s(t.get("criticality"));
            if c.is_empty() {
                "minor".to_string()
            } else {
                c
            }
        },
        migrated_from: s(t.get("migrated_from")),
        // Never defaulted. A default here picks a direction for a bound whose
        // direction nobody stated, which is the one failure this field exists
        // to prevent.
        sense: s(t.get("sense")),
        // Five or six significant figures is what a MATLAB export prints, so
        // this is the floor set by the format rather than by the physics.
        parity_tolerance: t
            .get("parity_tolerance")
            .and_then(|v| v.as_float())
            .unwrap_or(1e-4),
        state: s(t.get("state")),
        layer: u(t.get("layer")) as u8,
        order: u(t.get("order")),
        crosses_to: s(t.get("crosses_to")),
        crate_name: crate_name.to_string(),
        dir: dir.to_path_buf(),
        ..Default::default()
    };
    if let Some(q) = t.get("question").and_then(|q| q.as_table()) {
        sh.question = s(q.get("text"));
        sh.note = s(q.get("note"));
    }
    if let Some(m) = t.get("maths").and_then(|m| m.as_table()) {
        sh.expression = s(m.get("expression"));
        sh.source = s(m.get("source"));
        sh.relation_by = s(m.get("confirmed_by"));
    }
    // Prose, and read the same way whether it is one paragraph or twenty lines.
    // Nothing below feeds the generators: a missing [theory] is an open gap on a
    // published node, never a failure to build one.
    if let Some(th) = t.get("theory").and_then(|th| th.as_table()) {
        sh.theory.why = reflow(&s(th.get("why")));
        sh.theory.reading = reflow(&s(th.get("reading")));
        for st in th.get("step").and_then(|s| s.as_array()).unwrap_or(&vec![]) {
            let st = st.as_table().unwrap();
            sh.theory.steps.push(TheoryStep {
                text: reflow(&s(st.get("text"))),
                math: reflow(&s(st.get("math"))),
            });
        }
    }
    if let Some(ex) = t.get("explain").and_then(|x| x.as_table()) {
        sh.explain.simply = reflow(&s(ex.get("simply")));
        sh.explain.breaks = reflow(&s(ex.get("breaks")));
        sh.explain.wrong = reflow(&s(ex.get("wrong")));
        sh.explain.by = s(ex.get("by")).trim().to_string();
    }
    if let Some(m) = t.get("method").and_then(|x| x.as_table()) {
        // Kept exactly as written: a method's lines and indentation are its
        // meaning, and a reflow would join them.
        sh.method.text = code(m.get("text"));
        sh.method.by = s(m.get("by")).trim().to_string();
    }
    if let Some(a) = t.get("author").and_then(|x| x.as_table()) {
        sh.author = crate::model::AuthorCode {
            name: s(a.get("name")),
            language: s(a.get("language")),
            entry: s(a.get("entry")),
            code: code(a.get("code")),
            test_code: code(a.get("test_code")),
            how_run: reflow(&s(a.get("how_run"))),
        };
    }
    sh.cases = crate::method::cases_of(&v).map_err(|e| format!("{}: {e}", path.display()))?;
    for fl in t
        .get("flight")
        .and_then(|x| x.as_array())
        .unwrap_or(&vec![])
    {
        let Some(fl) = fl.as_table() else { continue };
        sh.flight.push(crate::model::Flight {
            name: s(fl.get("name")),
            language: s(fl.get("language")),
            purpose: reflow(&s(fl.get("purpose"))),
            code: code(fl.get("code")),
            test_code: code(fl.get("test_code")),
            test_result: reflow(&s(fl.get("test_result"))),
        });
    }
    let strings = |v: Option<&toml::Value>| -> Vec<String> {
        v.and_then(|x| x.as_array())
            .map(|xs| {
                xs.iter()
                    .filter_map(|x| x.as_str())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    for v in t
        .get("version")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
    {
        let Some(v) = v.as_table() else { continue };
        sh.versions.push(Version {
            n: u(v.get("n")),
            release: s(v.get("release")),
            date: s(v.get("date")),
            by: s(v.get("by")),
            about: strings(v.get("about")),
            believed: reflow(&s(v.get("believed"))),
            tested: reflow(&s(v.get("tested"))),
            learned: reflow(&s(v.get("learned"))),
            cost: s(v.get("cost")),
            changed: reflow(&s(v.get("changed"))),
            risks: strings(v.get("risks")),
            rests_on: reflow(&s(v.get("rests_on"))),
            breaks_if: reflow(&s(v.get("breaks_if"))),
            relation: s(v.get("relation")),
            source: s(v.get("source")),
        });
    }
    for r in t.get("risk").and_then(|r| r.as_array()).unwrap_or(&vec![]) {
        let Some(r) = r.as_table() else { continue };
        sh.risks.push(Risk {
            id: s(r.get("id")),
            title: s(r.get("title")),
            level: s(r.get("level")),
            owner: s(r.get("owner")),
            since: s(r.get("since")),
            why: reflow(&s(r.get("why"))),
        });
    }
    for a in t
        .get("assumption")
        .and_then(|a| a.as_array())
        .unwrap_or(&vec![])
    {
        let a = a.as_table().unwrap();
        sh.assumptions.push(Assumption {
            text: s(a.get("text")),
            fails_when: s(a.get("fails_when")),
        });
    }
    if let Some(o) = t.get("output").and_then(|o| o.as_table()) {
        sh.symbol = s(o.get("symbol"));
        sh.ty = s(o.get("type"));
        sh.unit = s(o.get("unit"));
        sh.lower = f(o.get("lower"));
        sh.upper = f(o.get("upper"));
        sh.reason_lower = s(o.get("reason_lower"));
        sh.reason_upper = s(o.get("reason_upper"));
    }
    for pb in t
        .get("publishes")
        .and_then(|p| p.as_array())
        .unwrap_or(&vec![])
    {
        let pb = match pb.as_table() {
            Some(x) => x,
            None => continue,
        };
        sh.publishes.push(crate::model::Publish {
            id: s(pb.get("id")),
            symbol: s(pb.get("symbol")),
            label: s(pb.get("label")),
            ty: s(pb.get("type")),
            unit: s(pb.get("unit")),
            lower: f(pb.get("lower")),
            upper: f(pb.get("upper")),
            reason_lower: s(pb.get("reason_lower")),
            reason_upper: s(pb.get("reason_upper")),
        });
    }
    if let Some(val) = t.get("value").and_then(|v| v.as_table()) {
        sh.value = Some(f(val.get("number")));
        sh.confirmed_by = s(val.get("confirmed_by"));
    }
    for i in t.get("input").and_then(|i| i.as_array()).unwrap_or(&vec![]) {
        let i = i.as_table().unwrap();
        sh.inputs.push(Input {
            binding: s(i.get("binding")),
            var: s(i.get("var")),
            ty: s(i.get("type")),
        });
    }
    if let Some(alg) = t.get("algorithm").and_then(|a| a.as_table()) {
        for st in alg
            .get("step")
            .and_then(|s| s.as_array())
            .unwrap_or(&vec![])
        {
            let st = st.as_table().unwrap();
            sh.steps.push(Step {
                number: u(st.get("number")),
                text: s(st.get("text")),
                binds: s(st.get("binds")),
                ty: s(st.get("type")),
            });
        }
    }
    if let Some(c) = t.get("contributes").and_then(|c| c.as_table()) {
        for k in c.get("kpis").and_then(|k| k.as_array()).unwrap_or(&vec![]) {
            sh.kpis.push(k.as_str().unwrap_or("").to_string());
        }
    }
    if let Some(d) = t.get("data").and_then(|d| d.as_table()) {
        for b in d
            .get("bundles")
            .and_then(|b| b.as_array())
            .unwrap_or(&vec![])
        {
            sh.bundles.push(b.as_str().unwrap_or("").to_string());
        }
    }
    if let Some(vw) = t.get("view").and_then(|v| v.as_table()) {
        sh.view = match s(vw.get("kind")).as_str() {
            "line" => View::Line {
                over: s(vw.get("over")),
                points: u(vw.get("points")),
            },
            "heatmap" => View::Heatmap {
                over_x: s(vw.get("over_x")),
                over_y: s(vw.get("over_y")),
                points: u(vw.get("points")),
            },
            "bar" => View::Bar { y: s(vw.get("y")) },
            _ => View::Number,
        };
    }

    let fx = dir.join("fixtures.toml");
    if fx.is_file() {
        let ftext = fs::read_to_string(&fx).map_err(|e| format!("{}: {e}", fx.display()))?;
        let fv: toml::Value = ftext
            .parse()
            .map_err(|e| format!("{}: malformed fixtures: {e}", fx.display()))?;
        refuse_unknown(&fv, &fx)?;
        for r in fv
            .get("fixture")
            .and_then(|r| r.as_array())
            .unwrap_or(&vec![])
        {
            let r = r.as_table().unwrap();
            let mut inputs = Vec::new();
            if let Some(m) = r.get("inputs").and_then(|m| m.as_table()) {
                for (k, val) in m {
                    inputs.push((k.clone(), f(Some(val))));
                }
            }
            inputs.sort_by(|a, b| a.0.cmp(&b.0));
            sh.fixtures.push(Fixture {
                label: s(r.get("label")),
                expect: f(r.get("expect")),
                tolerance: f(r.get("tolerance")),
                provenance: s(r.get("provenance")),
                source: s(r.get("source")),
                inputs,
                variable: s(r.get("variable")),
            });
        }
    }

    // The sheet hash covers what a reader would call the node's meaning: the
    // question, the relation, the interface, the domain and the algorithm.
    // Formatting, comments and notes are deliberately outside it, so tidying a
    // sentence does not invalidate every artefact downstream.
    let mut canon = String::new();
    canon.push_str(&sh.id);
    canon.push_str(&sh.kind);
    canon.push_str(&sh.question);
    canon.push_str(&sh.expression);
    canon.push_str(&sh.source);
    canon.push_str(&sh.ty);
    canon.push_str(&sh.unit);
    canon.push_str(&format!("{:?}{:?}{:?}", sh.lower, sh.upper, sh.value));
    for i in &sh.inputs {
        canon.push_str(&i.binding);
        canon.push_str(&i.var);
        canon.push_str(&i.ty);
    }
    for st in &sh.steps {
        canon.push_str(&st.text);
        canon.push_str(&st.binds);
        canon.push_str(&st.ty);
    }
    // The extra variables a set row publishes are part of its meaning, and the
    // ORDER of them is part of its interface: `OUTPUT_VARS` and the slots the
    // bus writes are positional. A row that added, removed or reordered one
    // while keeping its hash would hand a face a page that names one variable
    // and a run that filled another. Every row with one answer publishes none
    // of these, so no existing hash moves.
    for pb in &sh.publishes {
        canon.push_str(&pb.id);
        canon.push_str(&pb.symbol);
        canon.push_str(&pb.ty);
        canon.push_str(&pb.unit);
        canon.push_str(&format!("{:?}{:?}", pb.lower, pb.upper));
    }
    // The method is what the generated code is translated from, so it is part
    // of what the node computes. Absent on every sheet that has not got one, so
    // no existing hash moves.
    if !sh.method.text.trim().is_empty() {
        canon.push_str("method:");
        canon.push_str(&sh.method.text.replace("\r\n", "\n"));
    }
    sh.sheet_hash = fnv1a(&canon);
    sh.impl_hash = fnv1a(&read_holes_raw(dir));
    Ok(sh)
}

/// Prose, unwrapped — but not unparagraphed.
///
/// A sheet wraps its paragraphs at a terminal width so it is readable as a file,
/// and those line breaks are a property of the file rather than of the sentence:
/// carried into a page they become breaks in the middle of clauses. A blank line
/// IS content, though, so it survives as a paragraph break and everything else
/// collapses to one space.
fn reflow(s: &str) -> String {
    s.split("\n\n")
        .map(|para| para.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// The hole bodies, as one string, for the implementation hash.
fn read_holes_raw(dir: &Path) -> String {
    let p = dir.join("model.rs");
    let text = match fs::read_to_string(&p) {
        Ok(t) => t,
        Err(_) => return String::new(),
    };
    let mut out = String::new();
    let mut inside = false;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("// ---- HOLE ") {
            inside = true;
            continue;
        }
        if l.starts_with("// ---- end HOLE") {
            inside = false;
            continue;
        }
        if inside {
            out.push_str(l);
            out.push('\n');
        }
    }
    out
}

/// The bodies of the filled holes, keyed by hole number.
///
/// This is the generation gap: the scaffold is emitted from the sheet every
/// time, and the few typed lines inside each marker are carried across
/// unchanged. A hand edit anywhere outside a marker is lost by the
/// regeneration and therefore caught by the regeneration diff, which is what
/// makes the generated region genuinely owned by the generator rather than
/// merely labelled that way.
pub fn read_holes(dir: &Path) -> BTreeMap<u32, String> {
    let mut map = BTreeMap::new();
    let p = dir.join("model.rs");
    let text = match fs::read_to_string(&p) {
        Ok(t) => t,
        Err(_) => return map,
    };
    let mut current: Option<u32> = None;
    let mut buf = String::new();
    for line in text.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("// ---- HOLE ") {
            let n: u32 = rest
                .split_whitespace()
                .next()
                .and_then(|x| x.parse().ok())
                .unwrap_or(0);
            current = Some(n);
            buf.clear();
            continue;
        }
        if l.starts_with("// ---- end HOLE") {
            if let Some(n) = current.take() {
                map.insert(n, buf.trim_end().to_string());
            }
            continue;
        }
        if current.is_some() {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    map
}

fn load_layers(tree: &mut Tree) -> Result<(), String> {
    begin();
    load_layers_values(tree)?;
    finish(&tree.root.join("load_layers"))
}

fn load_layers_values(tree: &mut Tree) -> Result<(), String> {
    let dir = tree.root.join("layers");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("layers/: {e}"))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "toml").unwrap_or(false))
        .collect();
    files.sort();
    for p in files {
        let text = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let v: toml::Value = text.parse().map_err(|e| format!("{}: {e}", p.display()))?;
        // A layer file may declare the architecture's own cycles. They belong
        // here rather than in a case because a loop is a property of the
        // design, not of who bought it.
        tree.cycles.extend(parse_cycles(&v));
        for g in v.get("group").and_then(|g| g.as_array()).unwrap_or(&vec![]) {
            let g = g.as_table().unwrap();
            let grp = Group {
                id: s(g.get("id")),
                label: s(g.get("label")),
                parent: s(g.get("parent")),
                owner: s(g.get("owner")),
                layer: u(g.get("layer")) as u8,
                order: u(g.get("order")),
                is_box: g.get("box").and_then(|b| b.as_bool()).unwrap_or(false),
                tone: s(g.get("tone")),
                cases: g
                    .get("cases")
                    .and_then(|c| c.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(|x| x.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
            };
            tree.groups.insert(grp.id.clone(), grp);
        }
        for r in v
            .get("relates")
            .and_then(|r| r.as_array())
            .unwrap_or(&vec![])
        {
            let r = r.as_table().unwrap();
            tree.relations.push(Relation {
                from: s(r.get("from")),
                to: s(r.get("to")),
                why: s(r.get("why")),
            });
        }
    }
    Ok(())
}

/// Parse an `[[iterate]]` array. The same shape appears in `layers/cycles.toml`
/// for the architecture's own loops and in a case file for one a customer adds,
/// so it is read in one place rather than twice.
fn parse_cycles(v: &toml::Value) -> Vec<CycleSpec> {
    let mut out = Vec::new();
    for it in v
        .get("iterate")
        .and_then(|i| i.as_array())
        .unwrap_or(&vec![])
    {
        let Some(it) = it.as_table() else { continue };
        let mut cy = CycleSpec {
            converge_on: s(it.get("converge_on")),
            tolerance: f(it.get("tolerance")),
            max_iter: u(it.get("max_iter")),
            ..Default::default()
        };
        for n in it
            .get("nodes")
            .and_then(|n| n.as_array())
            .unwrap_or(&vec![])
        {
            cy.nodes.push(n.as_str().unwrap_or("").to_string());
        }
        for sd in it.get("seed").and_then(|s| s.as_array()).unwrap_or(&vec![]) {
            let Some(sd) = sd.as_table() else { continue };
            cy.seeds.push((s(sd.get("var")), f(sd.get("value"))));
        }
        out.push(cy);
    }
    out
}

fn load_cases(tree: &mut Tree) -> Result<(), String> {
    begin();
    load_cases_values(tree)?;
    finish(&tree.root.join("load_cases"))
}

fn load_cases_values(tree: &mut Tree) -> Result<(), String> {
    let dir = tree.root.join("cases");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("cases/: {e}"))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "toml").unwrap_or(false))
        .collect();
    files.sort();
    for p in files {
        let text = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let v: toml::Value = text.parse().map_err(|e| format!("{}: {e}", p.display()))?;
        let mut c = Case {
            id: s(v.get("id")),
            label: s(v.get("label")),
            note: s(v.get("note")),
            ..Default::default()
        };
        if let Some(cond) = v
            .get("groups")
            .and_then(|g| g.get("condition"))
            .and_then(|c| c.as_array())
        {
            c.conditions = cond
                .iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect();
        }
        if let Some(sup) = v.get("supply").and_then(|s| s.as_table()) {
            for (k, val) in sup {
                c.supply.push((k.clone(), f(Some(val))));
            }
            c.supply.sort_by(|a, b| a.0.cmp(&b.0));
        }
        // Every case runs the architecture's cycles. A case may add one of its
        // own; it may not drop one the design has.
        c.cycles = tree.cycles.clone();
        c.cycles.extend(parse_cycles(&v));
        tree.cases.insert(c.id.clone(), c);
    }
    Ok(())
}

fn load_sources(tree: &mut Tree) -> Result<(), String> {
    begin();
    load_sources_values(tree)?;
    finish(&tree.root.join("load_sources"))
}

fn load_sources_values(tree: &mut Tree) -> Result<(), String> {
    let p = tree.root.join("sources").join("sources.toml");
    let text = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let v: toml::Value = text.parse().map_err(|e| format!("{}: {e}", p.display()))?;
    for r in v
        .get("source")
        .and_then(|r| r.as_array())
        .unwrap_or(&vec![])
    {
        let r = r.as_table().unwrap();
        let src = Source {
            id: s(r.get("id")),
            title: s(r.get("title")),
            where_: s(r.get("where")),
            status: s(r.get("status")),
            used_for: s(r.get("used_for")),
        };
        tree.sources.insert(src.id.clone(), src);
    }
    Ok(())
}

impl Tree {
    /// Global index of a node, by identifier. Indices are assigned in sorted
    /// order, so they are stable between machines and between runs.
    pub fn index_of(&self, id: &str) -> Option<u16> {
        self.sheets.keys().position(|k| k == id).map(|i| i as u16)
    }
    pub fn ordered(&self) -> Vec<&Sheet> {
        self.sheets.values().collect()
    }
}
