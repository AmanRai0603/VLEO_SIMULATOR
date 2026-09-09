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
}

fn s(v: Option<&toml::Value>) -> String {
    v.and_then(|v| v.as_str()).unwrap_or("").to_string()
}

fn f(v: Option<&toml::Value>) -> f64 {
    v.and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
        .unwrap_or(0.0)
}

fn u(v: Option<&toml::Value>) -> u32 {
    v.and_then(|v| v.as_integer()).unwrap_or(0) as u32
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
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: toml::Value = text
        .parse()
        .map_err(|e| format!("{}: malformed sheet: {e}", path.display()))?;
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
        state: s(t.get("state")),
        layer: u(t.get("layer")) as u8,
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
    sh.sheet_hash = fnv1a(&canon);
    sh.impl_hash = fnv1a(&read_holes_raw(dir));
    Ok(sh)
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
        for g in v.get("group").and_then(|g| g.as_array()).unwrap_or(&vec![]) {
            let g = g.as_table().unwrap();
            let grp = Group {
                id: s(g.get("id")),
                label: s(g.get("label")),
                parent: s(g.get("parent")),
                owner: s(g.get("owner")),
                layer: u(g.get("layer")) as u8,
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

fn load_cases(tree: &mut Tree) -> Result<(), String> {
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
        if let Some(sup) = v.get("supply").and_then(|s| s.as_table()) {
            for (k, val) in sup {
                c.supply.push((k.clone(), f(Some(val))));
            }
            c.supply.sort_by(|a, b| a.0.cmp(&b.0));
        }
        for it in v
            .get("iterate")
            .and_then(|i| i.as_array())
            .unwrap_or(&vec![])
        {
            let it = it.as_table().unwrap();
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
                let sd = sd.as_table().unwrap();
                cy.seeds.push((s(sd.get("var")), f(sd.get("value"))));
            }
            c.cycles.push(cy);
        }
        tree.cases.insert(c.id.clone(), c);
    }
    Ok(())
}

fn load_sources(tree: &mut Tree) -> Result<(), String> {
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
