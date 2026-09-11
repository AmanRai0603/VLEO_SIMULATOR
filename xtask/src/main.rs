//! `cargo xtask` — the one gate binary.
//!
//! Rust rather than shell scripts or a pipeline-only step: cross-platform,
//! identical on a laptop and in continuous integration. A rule that lives only
//! in the pipeline is a rule half the team never sees.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vleo_sheet::{emit, gate, load_all, page, Tree};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");
    let root = repo_root();
    let rest: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    let r = match cmd {
        "docs" => cmd_docs(&root, &rest),
        "assemble" => cmd_assemble(&root, &rest),
        "gate" => cmd_gate(&root, &rest),
        "status" => cmd_status(&root),
        "gap" => cmd_gap(&root),
        "graph" => cmd_graph(&root),
        "new" => cmd_new(&root, &rest),
        "fill" => cmd_fill(&root, &rest),
        "codeowners" => cmd_codeowners(&root),
        "bundle" => cmd_bundle(&root, &rest),
        "variables" => cmd_variables(&root),
        "help" | "--help" | "-h" => {
            help();
            Ok(())
        }
        other => Err(format!(
            "unknown command '{other}'. Try `cargo xtask help`."
        )),
    };

    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("\x1b[31mxtask: {e}\x1b[0m");
            ExitCode::FAILURE
        }
    }
}

fn help() {
    println!(
        "\
cargo xtask <command>

  docs [<node>]      the six per-node generators — model, contract, module,
                     evidence, page fragment and metadata. They never read
                     another node, which is what makes 250 nodes 250
                     independent acts.
  assemble           the three assembly generators — the index, the document
                     and the graph tables. They combine and refuse; they never
                     decide, because a decision taken during assembly is a
                     decision nobody reviewed.
  gate [<node>]      the checks, in order, stopping at the first failure.
                     Called by the authoring hook, by the pipeline and by hand.
  status             counts by state and by subsystem, and what is blocking.
  gap                what every sheet promised and nothing yet covers.
  graph              the three graphs, their sizes, and the crate direction check.
  new <id> --like <sibling>
                     clone the shape of a sibling and blank what must be
                     re-decided. Not a copy: a real copy drags a stale source
                     citation through thirty nodes.
  fill <node> --hole <n> --body <file|->
                     splice one hole body into a generated model.rs. The hole
                     filler is never handed the file: it returns the few typed
                     lines as text and this puts them where they go. An agent
                     given the file and told not to stray is not constrained.
  codeowners         regenerate CODEOWNERS from the layer files.
  bundle publish <dir>
                     hash every payload file and write the result into the
                     manifest. Publishing twice from the same input gives the
                     same hash, which is what makes verification mean anything.
                     Publication is irreversible by design.
  bundle verify      re-check every hash in bundles/.
  variables          write docs/VARIABLES.md — every variable in the tree, its
                     unit, its range, the reason for each bound, and what reads
                     it. Generated, because a register maintained by hand is a
                     register that is wrong.

The tree is seeded once, ever, by tools/seed_tree.py."
    );
}

fn repo_root() -> PathBuf {
    let mut p = std::env::current_dir().expect("a working directory");
    loop {
        if p.join("Cargo.toml").is_file() && p.join("crates").is_dir() && p.join("layers").is_dir()
        {
            return p;
        }
        if !p.pop() {
            panic!("not inside the VLEO repository");
        }
    }
}

fn load(root: &Path) -> Result<Tree, String> {
    load_all(root)
}

fn write_if_changed(path: &Path, text: &str) -> Result<bool, String> {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == text {
            return Ok(false);
        }
    }
    if let Some(d) = path.parent() {
        fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(true)
}

// ---------------------------------------------------------------------------

fn cmd_docs(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
    let only = args.first().copied();
    let mut written = 0usize;
    let mut touched = 0usize;
    for sh in tree.ordered() {
        if let Some(o) = only {
            if sh.id != o {
                continue;
            }
        }
        touched += 1;
        let holes = vleo_sheet::load::read_holes(&sh.dir);
        let gaps = emit::gap_pass(sh, &holes);
        // A seeded row gets its page and its metadata and no code at all.
        // Everything is ready for it; the content is what is missing, and
        // generating a file full of `todo!()` would hide that behind something
        // that looks like work.
        let artefacts: Vec<(&str, String)> = if sh.is_seeded() {
            vec![
                ("page.html", page::fragment(sh, &holes, &tree)),
                ("meta.json", emit::meta_json(sh, &gaps)),
            ]
        } else {
            vec![
                ("model.rs", emit::model_rs(sh, &holes)),
                ("contract.rs", emit::contract_rs(sh)),
                ("mod.rs", emit::mod_rs(sh)),
                ("evidence.rs", emit::evidence_rs(sh)),
                ("page.html", page::fragment(sh, &holes, &tree)),
                ("meta.json", emit::meta_json(sh, &gaps)),
            ]
        };
        for (name, text) in artefacts {
            // Format the candidate before comparing, so the generator is a
            // function of its input: writing unformatted text and formatting it
            // afterwards makes every run report a change and the
            // regenerate-and-compare check stops meaning anything.
            let text = if name.ends_with(".rs") {
                gate::formatted(&text)
            } else {
                text
            };
            if write_if_changed(&sh.dir.join(name), &text)? {
                written += 1;
            }
        }
    }
    if touched == 0 {
        return Err(format!("no node matched '{}'", only.unwrap_or("")));
    }
    println!("docs: {touched} node(s), {written} artefact(s) written");
    Ok(())
}

fn cmd_assemble(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
    let checks = gate::validate_tree(&tree);
    let failed: Vec<&gate::Check> = checks.iter().filter(|c| c.failed()).collect();
    for c in &checks {
        match &c.verdict {
            gate::Verdict::Pass => println!("  \x1b[32mok\x1b[0m   {}", c.name),
            gate::Verdict::Note(w) => println!("  \x1b[33mnote\x1b[0m {} — {w}", c.name),
            gate::Verdict::Fail(w) => println!("  \x1b[31mFAIL\x1b[0m {} — {w}", c.name),
        }
    }
    if !failed.is_empty() && !args.contains(&"--force") {
        return Err(format!(
            "{} assembly validation(s) failed. Assembly combines and refuses; it never decides.",
            failed.len()
        ));
    }

    let out = root.join("generated");
    fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    fs::write(out.join("index.json"), page::index_json(&tree))
        .map_err(|e| format!("index.json: {e}"))?;

    // The document: the shell plus every fragment, served from one directory.
    let frag_dir = out.join("fragments");
    fs::create_dir_all(&frag_dir).map_err(|e| format!("{}: {e}", frag_dir.display()))?;
    let mut bytes = 0usize;
    for sh in tree.ordered() {
        let src = sh.dir.join("page.html");
        if let Ok(t) = fs::read_to_string(&src) {
            bytes += t.len();
            fs::write(frag_dir.join(format!("{}.html", sh.id)), t)
                .map_err(|e| format!("fragment {}: {e}", sh.id))?;
        }
    }
    println!(
        "assemble: index {} KB, {} fragments totalling {} KB — nothing here is committed",
        page::index_json(&tree).len() / 1024,
        tree.sheets.len(),
        bytes / 1024
    );
    Ok(())
}

fn cmd_gate(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
    let only = args.first().copied();
    let mut failures = 0usize;
    let mut nodes = 0usize;
    for sh in tree.ordered() {
        if let Some(o) = only {
            if sh.id != o {
                continue;
            }
        }
        nodes += 1;
        let checks = gate::gate_node(sh, &tree);
        let bad: Vec<&gate::Check> = checks.iter().filter(|c| c.failed()).collect();
        if only.is_some() {
            for c in &checks {
                match &c.verdict {
                    gate::Verdict::Pass => println!("  \x1b[32mok\x1b[0m   {}", c.name),
                    gate::Verdict::Note(w) => println!("  \x1b[33mnote\x1b[0m {} — {w}", c.name),
                    gate::Verdict::Fail(w) => println!("  \x1b[31mFAIL\x1b[0m {} — {w}", c.name),
                }
            }
        } else if !bad.is_empty() {
            println!("\x1b[31m{}\x1b[0m", sh.id);
            for c in bad.iter() {
                if let gate::Verdict::Fail(w) = &c.verdict {
                    println!("    {} — {w}", c.name);
                }
            }
        }
        failures += bad.len();
    }
    let tree_checks = gate::validate_tree(&tree);
    let tree_bad: Vec<&gate::Check> = tree_checks.iter().filter(|c| c.failed()).collect();
    for c in &tree_bad {
        if let gate::Verdict::Fail(w) = &c.verdict {
            println!("\x1b[31massembly\x1b[0m {} — {w}", c.name);
        }
    }
    println!(
        "gate: {nodes} node(s), {failures} node check failure(s), {} assembly failure(s)",
        tree_bad.len()
    );
    if failures + tree_bad.len() > 0 {
        return Err("the gate refused. A fixture disagreement is a physics disagreement, not a build failure — take it to the node owner, and do not widen the tolerance.".into());
    }
    Ok(())
}

fn cmd_status(root: &Path) -> Result<(), String> {
    let tree = load(root)?;
    let names = ["", "management", "the system", "subsystem", "the run"];
    let mut by_layer: BTreeMap<u8, [usize; 2]> = BTreeMap::new();
    for sh in tree.ordered() {
        let e = by_layer.entry(sh.layer).or_default();
        e[0] += 1;
        if !sh.is_seeded() {
            e[1] += 1;
        }
    }
    println!(
        "{:<4} {:<14} {:>7} {:>11} {:>9}",
        "", "layer", "rows", "specified", "seeded"
    );
    for (l, c) in &by_layer {
        println!(
            "{:<4} {:<14} {:>7} {:>11} {:>9}",
            l,
            names.get(*l as usize).copied().unwrap_or("?"),
            c[0],
            c[1],
            c[0] - c[1]
        );
    }
    println!(
        "{:<4} {:<14} {:>7} {:>11} {:>9}\n",
        "",
        "total",
        tree.sheets.len(),
        tree.ordered().iter().filter(|s| !s.is_seeded()).count(),
        tree.ordered().iter().filter(|s| s.is_seeded()).count()
    );
    let mut by_sub: BTreeMap<&str, [usize; 3]> = BTreeMap::new();
    let mut gaps_total = 0usize;
    for sh in tree.ordered() {
        let e = by_sub.entry(sh.subsystem.as_str()).or_default();
        e[0] += 1;
        if sh.is_declared() {
            e[1] += 1;
        } else {
            e[2] += 1;
        }
        gaps_total += emit::gap_pass(sh, &vleo_sheet::load::read_holes(&sh.dir)).len();
    }
    println!(
        "{:<12} {:>6} {:>10} {:>10}",
        "subsystem", "rows", "declared", "computed"
    );
    for (s, c) in &by_sub {
        println!("{:<12} {:>6} {:>10} {:>10}", s, c[0], c[1], c[2]);
    }
    println!(
        "{:<12} {:>6} {:>10} {:>10}",
        "total",
        tree.sheets.len(),
        by_sub.values().map(|c| c[1]).sum::<usize>(),
        by_sub.values().map(|c| c[2]).sum::<usize>()
    );
    println!();
    println!(
        "{} of {} rows carry no open gap. Always n of {} — the denominator was always there.",
        tree.sheets.len()
            - tree
                .ordered()
                .iter()
                .filter(|s| !emit::gap_pass(s, &vleo_sheet::load::read_holes(&s.dir)).is_empty())
                .count(),
        tree.sheets.len(),
        tree.sheets.len()
    );
    println!("{gaps_total} open gap(s) across the tree.");
    let kpis: Vec<&str> = tree
        .ordered()
        .iter()
        .filter(|s| s.kind == "kpi")
        .map(|s| s.id.as_str())
        .collect();
    println!("{} KPI closures declared: {}", kpis.len(), kpis.join(", "));
    Ok(())
}

fn cmd_gap(root: &Path) -> Result<(), String> {
    let tree = load(root)?;
    let report = emit::gap_report(&tree);
    if report.is_empty() {
        println!("gap pass: nothing the sheets promised is uncovered.");
        return Ok(());
    }
    for (id, gaps) in &report {
        println!("\x1b[33m{id}\x1b[0m");
        for g in gaps {
            println!("    {g}");
        }
    }
    println!(
        "\n{} node(s) with an open gap. This is a list, not a verdict — but a node with an open gap cannot enter the second review.",
        report.len()
    );
    Ok(())
}

fn cmd_graph(root: &Path) -> Result<(), String> {
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
    Ok(())
}

fn deepest_chain(tree: &Tree) -> Vec<String> {
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

/// Splice one hole body into a node's `model.rs`.
///
/// This exists so that the hole filler never touches the file. The working
/// model is explicit that its prohibition holds *only* in that form: an agent
/// given write access to a file and told not to stray is not constrained, it is
/// asked. So the agent returns the body of one numbered step as text, and this
/// is the only thing that puts text into a generated file.
///
/// Four refusals, all before anything is written:
///
///   * a hole number the sheet does not declare
///   * a body carrying a `HOLE` marker of its own, which would let one body
///     claim the block after it as well
///   * a guard. Guards are generated from the declared domain and travel with
///     their reason; one added here has no reason attached and is deleted by
///     the next person who finds it awkward
///   * a platform maths call. The gate catches this later, but later means
///     after it is committed, and the message is more useful at the moment
///     somebody is holding the two lines in their head
fn cmd_fill(root: &Path, args: &[&str]) -> Result<(), String> {
    let id = args
        .first()
        .ok_or("usage: cargo xtask fill <node> --hole <n> --body <file|->")?;
    let n: u32 = args
        .iter()
        .position(|a| *a == "--hole")
        .and_then(|i| args.get(i + 1))
        .ok_or("which hole: --hole <n>")?
        .parse()
        .map_err(|_| "--hole takes a number".to_string())?;
    let src = args
        .iter()
        .position(|a| *a == "--body")
        .and_then(|i| args.get(i + 1))
        .ok_or("the body, as a file or - for standard input: --body <file|->")?;

    let body = if *src == "-" {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
            .map_err(|e| e.to_string())?;
        buf
    } else {
        fs::read_to_string(src).map_err(|e| format!("{src}: {e}"))?
    };
    if body.trim().is_empty() {
        return Err(
            "the body is empty. An empty hole is a gap, and the gap pass already says so".into(),
        );
    }

    let tree = load(root)?;
    let sh = tree
        .sheets
        .get(*id)
        .ok_or_else(|| format!("no node '{id}'"))?;
    if sh.is_declared() {
        return Err(format!(
            "'{id}' is a declared value — a person picked its number, so it has no holes"
        ));
    }
    let step = sh
        .steps
        .iter()
        .find(|st| st.number == n)
        .ok_or_else(|| {
            format!(
                "'{id}' declares {} step(s); there is no hole {n}. The algorithm in the sheet decides how many there are",
                sh.steps.len()
            )
        })?;

    for (needle, why) in [
        ("---- HOLE", "a body may not carry a HOLE marker — one body would claim the next block as well"),
        ("---- end HOLE", "a body may not carry a HOLE marker — one body would claim the next block as well"),
        ("Fault::", "a body may not construct a fault. The guards are generated from the declared domain, with the reason attached"),
        ("return Err(", "a body may not return early. The generated tail maps the answer and its faults"),
    ] {
        if body.contains(needle) {
            return Err(format!("refused: {why} (found {needle:?})"));
        }
    }
    for bad in [
        ".sin()", ".cos()", ".exp()", ".ln()", ".powf(", ".sqrt()", ".atan2(", ".tan()", ".log10(",
    ] {
        if body.contains(bad) {
            return Err(format!(
                "refused: the body calls {bad} — route it through pmath, or cross-face agreement \
                 fails on the first night for a reason that is not a defect"
            ));
        }
    }

    let mut holes = vleo_sheet::load::read_holes(&sh.dir);
    let before = holes.get(&n).cloned().unwrap_or_default();
    holes.insert(n, body.clone());
    let text = gate::formatted(&emit::model_rs(sh, &holes));
    write_if_changed(&sh.dir.join("model.rs"), &text)?;

    // Read it back and prove the body landed where it was meant to. Writing a
    // file and announcing success is how a splice that silently dropped the
    // last line gets discovered three nodes later.
    let after = vleo_sheet::load::read_holes(&sh.dir);
    let landed = after
        .get(&n)
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if landed.is_empty() {
        return Err(format!(
            "hole {n} is still empty after the splice — nothing was written"
        ));
    }
    println!(
        "{id} hole {n} ({}) — {} line(s) spliced{}",
        step.text,
        landed.lines().count(),
        if before.trim().is_empty() {
            ""
        } else {
            ", replacing what was there"
        }
    );
    // The crate is the folder the node lives in, not its subsystem tag: gnc
    // rows live in vleo-mod-acs, and a command line that names a crate nobody
    // has is worse than no command line.
    let krate = sh
        .dir
        .ancestors()
        .nth(2)
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    println!("Now: cargo xtask gate {id} && cargo test -p {krate}");
    Ok(())
}

fn cmd_new(root: &Path, args: &[&str]) -> Result<(), String> {
    let id = args
        .first()
        .ok_or("usage: cargo xtask new <id> --like <sibling>")?;
    let like = args
        .iter()
        .position(|a| *a == "--like")
        .and_then(|i| args.get(i + 1))
        .ok_or("a new node clones the shape of a sibling: --like <sibling id>")?;
    let tree = load(root)?;
    let src = tree
        .sheets
        .get(*like)
        .ok_or_else(|| format!("no node '{like}' to clone the shape of"))?;
    // The folder is the identifier. This used to strip the subsystem prefix,
    // which is how it came to disagree with the seeder: `prop_throat_area`
    // landed in `nodes/throat_area` while the tree already held it under its
    // own name, and two folders then claimed one id. The gate caught it, but
    // the rule only has to exist once for that not to happen at all.
    let folder = id.to_string();
    let dir = root
        .join("crates")
        .join(&src.crate_name)
        .join("nodes")
        .join(&folder);
    if dir.exists() {
        return Err(format!("{} already exists", dir.display()));
    }
    let sheet = fs::read_to_string(src.dir.join("node.toml")).map_err(|e| e.to_string())?;
    // Blank what must be re-decided. A literal copy drags a stale source
    // citation and someone else's domain limits through thirty nodes.
    let mut out = String::new();
    for line in sheet.lines() {
        let l = line.trim_start();
        if l.starts_with("id = ") {
            out.push_str(&format!("id = \"{id}\"\n"));
        } else if l.starts_with("folder = ") {
            out.push_str(&format!("folder = \"{folder}\"   # frozen at seed\n"));
        } else if l.starts_with("label = ")
            || l.starts_with("text = ")
            || l.starts_with("expression = ")
            || l.starts_with("source = ")
            || l.starts_with("note = ")
            || l.starts_with("reason_lower = ")
            || l.starts_with("reason_upper = ")
            || l.starts_with("confirmed_by = ")
        {
            let key = l.split(" = ").next().unwrap();
            out.push_str(&format!(
                "{key} = \"\"   # REQUIRED — re-decide, do not inherit\n"
            ));
        } else {
            out.push_str(line);
            out.push('\n');
            // Criticality decides how many people read this node and whether
            // its hole is filled twice by different model families. A sibling's
            // answer is not this node's answer, so it is asked here rather than
            // inherited silently.
            if l.starts_with("tier = ") && !sheet.contains("criticality") {
                out.push_str(
                    "criticality = \"minor\"   # minor | significant — significant means two \
                     reviewers and a differential fill\n",
                );
            }
        }
    }
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join("node.toml"), out).map_err(|e| e.to_string())?;
    fs::write(
        dir.join("fixtures.toml"),
        "# Known-good values, and where each came from. An expected value may\n# never be produced by the code under test.\n",
    )
    .map_err(|e| e.to_string())?;
    println!("new: {}", dir.display());
    println!("Now fill the sheet, then `cargo xtask docs {id}` and `cargo xtask gate {id}`.");
    println!("An open field fails the gate by name, which is the mechanism: ambiguity becomes a blocking item on an engineer's screen rather than something an implementer resolves silently.");
    Ok(())
}

fn cmd_codeowners(root: &Path) -> Result<(), String> {
    let tree = load(root)?;
    let mut o = String::new();
    o.push_str("# GENERATED by `cargo xtask codeowners` from the owner field on each layer\n");
    o.push_str("# group. Ownership is a path rule, not a convention: everyone reads\n");
    o.push_str("# everything and writes only their own nodes, which is the separation\n");
    o.push_str("# wanted without losing the whole-graph check.\n#\n");
    o.push_str("# The generators, the gate and the shared crates are integrator-owned and\n");
    o.push_str("# need two reviewers: a defect there reaches every node at once.\n\n");
    o.push_str("/crates/vleo-units/       @integrator @kernel-deputy\n");
    o.push_str("/crates/vleo-core/        @integrator @kernel-deputy\n");
    o.push_str("/crates/vleo-sheet/       @integrator @kernel-deputy\n");
    o.push_str("/xtask/                   @integrator @kernel-deputy\n");
    o.push_str("/layers/                  @integrator @systems\n");
    o.push_str("/cases/                   @integrator\n");
    o.push_str("/sources/                 @integrator @systems\n");
    o.push_str("/web/                     @web-owner @integrator\n\n");
    let mut by_owner: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for sh in tree.ordered() {
        by_owner
            .entry(sh.owner.as_str())
            .or_default()
            .push(sh.crate_name.as_str());
    }
    let mut seen = std::collections::BTreeSet::new();
    for sh in tree.ordered() {
        let key = (sh.crate_name.clone(), sh.folder.clone());
        if seen.insert(key) {
            o.push_str(&format!(
                "/crates/{}/nodes/{}/ @{}\n",
                sh.crate_name, sh.folder, sh.owner
            ));
        }
    }
    let _ = by_owner;
    fs::write(root.join("CODEOWNERS"), &o).map_err(|e| e.to_string())?;
    println!("codeowners: {} lines", o.lines().count());
    Ok(())
}

fn cmd_bundle(root: &Path, args: &[&str]) -> Result<(), String> {
    match args.first().copied() {
        Some("publish") => {
            let dir = args
                .get(1)
                .map(PathBuf::from)
                .ok_or("usage: cargo xtask bundle publish <dir>")?;
            let b = vleo_data::load_bundle(&dir)?;
            let computed = vleo_data::hash_files(&dir, &b.manifest.files)?;
            let mp = dir.join("manifest.toml");
            let text = fs::read_to_string(&mp).map_err(|e| e.to_string())?;
            let mut out = String::new();
            for line in text.lines() {
                if line.trim_start().starts_with("content_hash") {
                    out.push_str(&format!("content_hash = \"{computed}\"\n"));
                } else {
                    out.push_str(line);
                    out.push('\n');
                }
            }
            fs::write(&mp, out).map_err(|e| e.to_string())?;
            println!(
                "published {}@{} — {}",
                b.manifest.name, b.manifest.version, computed
            );
            println!("A published version is never edited. A correction is a new version.");
            Ok(())
        }
        Some("verify") | None => {
            let dir = root.join("bundles");
            let mut n = 0;
            let mut bad = 0;
            for e in fs::read_dir(&dir).map_err(|e| e.to_string())? {
                let p = e.map_err(|e| e.to_string())?.path();
                if !p.is_dir() {
                    continue;
                }
                for v in fs::read_dir(&p).map_err(|e| e.to_string())? {
                    let vp = v.map_err(|e| e.to_string())?.path();
                    if !vp.join("manifest.toml").is_file() {
                        continue;
                    }
                    n += 1;
                    let b = vleo_data::load_bundle(&vp)?;
                    if b.verified {
                        println!(
                            "  \x1b[32mok\x1b[0m   {}@{} {}",
                            b.manifest.name, b.manifest.version, b.manifest.content_hash
                        );
                    } else {
                        bad += 1;
                        println!(
                            "  \x1b[31mFAIL\x1b[0m {}@{} — {}",
                            b.manifest.name,
                            b.manifest.version,
                            b.refusal.unwrap_or_default()
                        );
                    }
                }
            }
            println!("{n} bundle(s), {bad} refused");
            if bad > 0 {
                return Err("a tampered byte is refused, never warned about".into());
            }
            Ok(())
        }
        Some(other) => Err(format!("unknown bundle command '{other}'")),
    }
}

/// The variable register.
///
/// Generated from the sheets, like everything else. A register maintained by
/// hand drifts from the tree within a week, and then it is worse than absent:
/// somebody will trust it.
fn cmd_variables(root: &Path) -> Result<(), String> {
    let tree = load(root)?;
    let mut o = String::new();
    o.push_str("<!-- GENERATED by `cargo xtask variables`. Do not edit: the sheets are the source. -->\n\n");
    o.push_str("# The variable register\n\n");
    o.push_str("Every row in the tree, with the unit it publishes in, the range over which it\n");
    o.push_str("is declared valid, and the reason for each bound. A guard whose reason is not\n");
    o.push_str("written down gets deleted by the next person who finds it awkward, so the\n");
    o.push_str("reasons are part of the register rather than a comment in the code.\n\n");

    let declared = tree.ordered().iter().filter(|s| s.is_declared()).count();
    let computed = tree.sheets.len() - declared;
    o.push_str(&format!(
        "**{} rows** — {} a person picked, {} worked out. Two thirds of any design tree is\n\
         the first kind: cheaper than a computed node, and not free, because every margin\n\
         in the design is built out of them.\n\n",
        tree.sheets.len(),
        declared,
        computed
    ));

    let mut by_sub: BTreeMap<&str, Vec<&vleo_sheet::Sheet>> = BTreeMap::new();
    for sh in tree.ordered() {
        by_sub.entry(sh.subsystem.as_str()).or_default().push(sh);
    }
    for (sub, sheets) in &by_sub {
        let label = tree
            .groups
            .values()
            .find(|g| sheets.iter().any(|s| s.parent == g.id))
            .map(|g| g.label.clone())
            .unwrap_or_else(|| (*sub).to_string());
        o.push_str(&format!("\n## `{sub}` — {label}\n\n"));
        for sh in sheets {
            let unit = vleo_units::Unit::from_name(&sh.unit)
                .map(|u| u.symbol())
                .unwrap_or("?");
            o.push_str(&format!("### `{}` — {}\n\n", sh.id, sh.label));
            o.push_str(&format!("> {}\n\n", sh.question));
            o.push_str(&format!(
                "| | |\n|---|---|\n\
                 | symbol | `{}` |\n\
                 | type | `{}` |\n\
                 | unit | {} |\n\
                 | kind | {} |\n\
                 | owner | {} |\n\
                 | evidence tier | {} |\n\
                 | relation | `{}` |\n\
                 | source | `{}` |\n",
                sh.symbol, sh.ty, unit, sh.kind, sh.owner, sh.tier, sh.expression, sh.source
            ));
            if let Some(v) = sh.value {
                o.push_str(&format!("| declared value | **{v}** {unit} |\n"));
                o.push_str(&format!("| confirmed by | {} |\n", sh.confirmed_by));
            }
            o.push_str(&format!(
                "| valid over | {} … {} {} |\n",
                sh.lower, sh.upper, unit
            ));
            o.push('\n');
            o.push_str(&format!("- **lower bound** — {}\n", sh.reason_lower));
            o.push_str(&format!("- **upper bound** — {}\n", sh.reason_upper));
            if !sh.inputs.is_empty() {
                o.push_str(&format!(
                    "- **reads** — {}\n",
                    sh.inputs
                        .iter()
                        .map(|i| format!("`{}`", i.var))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            let consumers: Vec<String> = tree
                .sheets
                .values()
                .filter(|c| c.inputs.iter().any(|i| i.var == sh.id))
                .map(|c| format!("`{}`", c.id))
                .collect();
            o.push_str(&format!(
                "- **read by** — {}\n",
                if consumers.is_empty() {
                    "nothing yet. Every one of these is a leaf of the design, or an oversight."
                        .to_string()
                } else {
                    consumers.join(", ")
                }
            ));
            if !sh.kpis.is_empty() {
                o.push_str(&format!("- **contributes to** — {}\n", sh.kpis.join(", ")));
            }
            for a in &sh.assumptions {
                o.push_str(&format!(
                    "- **assumes** {} — fails when {}\n",
                    a.text, a.fails_when
                ));
            }
            if sh.fixtures.is_empty() && !sh.is_declared() {
                o.push_str("- **evidence** — none. Nothing outside this code has agreed with what it computes, so its validation credibility factor is zero, which governs the whole vector.\n");
            }
            for f in &sh.fixtures {
                o.push_str(&format!(
                    "- **evidence** {} — expect {} ± {} relative, from `{}` ({})\n",
                    f.label, f.expect, f.tolerance, f.source, f.provenance
                ));
            }
            if !sh.note.is_empty() {
                o.push_str(&format!("\n{}\n", sh.note));
            }
            o.push('\n');
        }
    }
    let p = root.join("docs").join("VARIABLES.md");
    fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(&p, &o).map_err(|e| e.to_string())?;
    println!(
        "variables: {} rows, {} KB -> {}",
        tree.sheets.len(),
        o.len() / 1024,
        p.display()
    );
    Ok(())
}
