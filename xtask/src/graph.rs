//! The crate graph, and the rings it must respect.

use super::*;

/// Which ring a crate sits in. Lower depends on nothing higher.
///
/// The four rings are the repository's one architectural rule, and until now
/// the only thing enforcing them was the compiler refusing a cycle — which
/// permits every wrong-direction edge that is not also circular. `vleo-core`
/// gaining a dependency on `vleo-bus` would compile, and would quietly make the
/// kernel depend on transport.
///
/// `vleo-sheet` sits beside the bus rather than in the chain: it is what a
/// sheet MEANS, and both the generators and the daemon read it. It reads the
/// kernel because a method may call a kernel function by name, and the method
/// checker runs that same function — so it sits above the kernel, never in
/// it. `vleo-data` is reference data and sits at the bus's level.
pub(super) fn ring(crate_name: &str) -> Option<(u8, &'static str)> {
    Some(match crate_name {
        "vleo-units" => (0, "RING 0 — quantities and portable maths"),
        "vleo-core" => (1, "RING 1 — the kernel: physics and the relations"),
        "vleo-sheet" => (2, "beside the bus — what a sheet means"),
        "vleo-files" => (
            2,
            "beside the bus — every design file, read, written and checked",
        ),
        "vleo-bus" => (2, "RING 2 — transport"),
        "vleo-data" => (2, "reference data"),
        "vleo-modules" => (4, "the facade over every node crate"),
        "vleo-design" => (
            4,
            "the design as one file — the tree, read as the folders are",
        ),
        "vleo-server" => (5, "the server both the daemon and the Python package start"),
        "vleo-cli" | "vleo-daemon" | "vleo-ffi" | "vleo-py" | "vleo-wasm" | "vleo-method-wasm"
        | "vleo-kernel-wasm" => (6, "a face"),
        "xtask" => (6, "the task runner"),
        n if n.starts_with("vleo-mod-") => (3, "RING 3 — the nodes"),
        _ => return None,
    })
}

/// Every wrong-direction dependency between the workspace's own crates.
///
/// Build dependencies count. `vleo-modules` reads `vleo-sheet` in its build
/// script to emit the tables, and a build-time edge in the wrong direction is
/// the same defect as a runtime one — it just fails later and more confusingly.
pub(super) fn crate_direction(root: &Path) -> Result<Vec<String>, String> {
    let mut bad = Vec::new();
    let mut seen = 0usize;
    let mut dirs: Vec<PathBuf> = fs::read_dir(root.join("crates"))
        .map_err(|e| format!("crates/: {e}"))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    dirs.push(root.join("xtask"));
    dirs.sort();
    for d in dirs {
        let ct = d.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&ct) else {
            continue;
        };
        let Ok(v) = text.parse::<toml::Value>() else {
            continue;
        };
        let Some(name) = v
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
        else {
            continue;
        };
        let Some((mine, what)) = ring(name) else {
            bad.push(format!(
                "{name} is in no ring — add it to `ring` and say where it belongs, or the \
                 direction check silently stops covering it"
            ));
            continue;
        };
        seen += 1;
        for table in ["dependencies", "build-dependencies"] {
            let Some(deps) = v.get(table).and_then(|d| d.as_table()) else {
                continue;
            };
            for dep in deps.keys() {
                if !dep.starts_with("vleo") {
                    continue;
                }
                let Some((theirs, their_what)) = ring(dep) else {
                    bad.push(format!("{name} depends on {dep}, which is in no ring"));
                    continue;
                };
                if theirs >= mine {
                    bad.push(format!(
                        "{name} ({what}) depends on {dep} ({their_what}) — \
                         {} and the rings depend inward only{}",
                        if theirs == mine {
                            "same ring"
                        } else {
                            "that is outward"
                        },
                        if table == "build-dependencies" {
                            ", and a build-time edge is the same defect as a runtime one"
                        } else {
                            ""
                        }
                    ));
                }
            }
        }
    }
    // A check that examined nothing must not report success.
    if seen < 20 {
        bad.push(format!(
            "only {seen} crate(s) were checked, which is fewer than this workspace has — \
             the direction check is not reading what it claims to"
        ));
    }
    Ok(bad)
}

pub(super) fn cmd_graph(root: &Path) -> Result<(), String> {
    let tree = load(root)?;
    let derivation: usize = tree.ordered().iter().map(|s| s.inputs.len()).sum();
    let contribution: usize = tree.ordered().iter().map(|s| s.kpis.len()).sum();
    println!("three graphs, never merged:");
    println!("  derivation   {derivation:>5} edges   variable -> node      execution");
    println!("  contribution {contribution:>5} edges   variable -> KPI       coverage");
    println!(
        "  relation     {:>5} edges   group -> group        navigation",
        tree.relations.len()
    );
    println!();
    let deepest = deepest_chain(&tree);
    println!(
        "deepest declared chain: {} nodes — {}",
        deepest.len(),
        deepest.join(" -> ")
    );
    let unread: Vec<&str> = tree
        .ordered()
        .iter()
        .filter(|s| {
            s.kind != "kpi"
                && !tree
                    .sheets
                    .values()
                    .any(|c| c.inputs.iter().any(|i| i.var == s.id))
        })
        .map(|s| s.id.as_str())
        .collect();
    println!(
        "{} node(s) nothing reads — every one is a leaf of the design, or an oversight:",
        unread.len()
    );
    for u in unread.iter().take(20) {
        println!("    {u}");
    }

    // THE CRATE DIRECTION CHECK. The help has named this for as long as the
    // command has existed and nothing implemented it, so the rings were held up
    // by the compiler refusing cycles — which allows every wrong-direction edge
    // that is not also circular.
    println!();
    let bad = crate_direction(root)?;
    if bad.is_empty() {
        println!("crate direction: every dependency points inward");
    } else {
        for b in &bad {
            println!("  \x1b[31mFAIL\x1b[0m {b}");
        }
        return Err(format!(
            "{} wrong-direction crate dependency(ies)",
            bad.len()
        ));
    }
    Ok(())
}

pub(super) fn deepest_chain(tree: &Tree) -> Vec<String> {
    let mut memo: BTreeMap<String, Vec<String>> = BTreeMap::new();
    fn depth(
        id: &str,
        tree: &Tree,
        memo: &mut BTreeMap<String, Vec<String>>,
        seen: &mut Vec<String>,
    ) -> Vec<String> {
        if let Some(v) = memo.get(id) {
            return v.clone();
        }
        if seen.iter().any(|s| s == id) {
            return vec![id.to_string()];
        }
        seen.push(id.to_string());
        let mut best: Vec<String> = Vec::new();
        if let Some(sh) = tree.sheets.get(id) {
            for i in &sh.inputs {
                let c = depth(&i.var, tree, memo, seen);
                if c.len() > best.len() {
                    best = c;
                }
            }
        }
        seen.pop();
        let mut out = best;
        out.push(id.to_string());
        memo.insert(id.to_string(), out.clone());
        out
    }
    let mut best = Vec::new();
    for id in tree.sheets.keys() {
        let c = depth(id, tree, &mut memo, &mut Vec::new());
        if c.len() > best.len() {
            best = c;
        }
    }
    best
}
