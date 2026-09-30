//! The generators and the gate: each node's files, the assembly, and the
//! registers generated from the tree — never edited by hand.

use super::*;

pub(crate) fn cmd_docs(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
    let only = args.first().copied();
    let mut written = 0usize;
    let mut touched = 0usize;
    let mut refused: Vec<(String, Vec<&'static str>)> = Vec::new();
    for sh in tree.ordered() {
        if let Some(o) = only {
            if sh.id != o {
                continue;
            }
        }
        // An open field is not a warning. The scaffold cannot be emitted
        // without every type, bound and precondition, so generation refuses
        // rather than producing a file that looks finished and is not. That
        // refusal is the mechanism: it turns ambiguity from something an
        // implementer settles quietly into a blocking item on an engineer's
        // screen. A seeded row is exempt — it has not been started, and its
        // page and metadata say exactly that.
        if !sh.is_seeded() {
            let missing = vleo_sheet::form::unfilled(sh);
            if !missing.is_empty() {
                refused.push((sh.id.clone(), missing));
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
    if !refused.is_empty() {
        for (id, missing) in &refused {
            println!(
                "  \x1b[31mrefused\x1b[0m {id} — nothing to generate from: {}",
                missing.join(", ")
            );
        }
        return Err(format!(
            "{} node(s) have an open field. The scaffold is a function of the sheet: no type, \n\
             no signature; no bound, no guard; no reason, and the guard is deleted by whoever \n\
             next finds it awkward. Answer them and run this again.",
            refused.len()
        ));
    }
    if touched == 0 {
        return Err(format!("no node matched '{}'", only.unwrap_or("")));
    }
    // Every node's method, translated into the kernel, whichever node was asked
    // for: the kernel module list is the whole tree's.
    written += emit::sync_methods(&tree)?;
    // The method language's reference page, from the tables the checker reads,
    // so the page and the checker cannot describe two languages.
    if only.is_none() {
        let md = vleo_sheet::method::reference_md();
        if write_if_changed(&root.join("docs/PSEUDOCODE.md"), &md)? {
            written += 1;
        }
    }
    println!("docs: {touched} node(s), {written} artefact(s) written");
    Ok(())
}

pub(crate) fn cmd_assemble(root: &Path, args: &[&str]) -> Result<(), String> {
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

pub(crate) fn cmd_gate(root: &Path, args: &[&str]) -> Result<(), String> {
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

pub(crate) fn cmd_codeowners(root: &Path) -> Result<(), String> {
    let tree = load(root)?;
    let mut o = String::new();
    o.push_str("# GENERATED by `cargo xtask codeowners` from the owner field on each NODE\n");
    o.push_str("# SHEET. It said \"each layer group\" and read node.toml, so changing a\n");
    o.push_str("# group's owner moved nobody and looked like it had. Ownership is a path\n");
    o.push_str("# rule, not a convention: everyone reads\n");
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
    o.push_str("/web/                     @web-owner @integrator\n");
    // What decides what merges, and what every run passes through, is owned
    // like the kernel: a defect there reaches everything at once.
    o.push_str("/.github/                 @integrator @kernel-deputy\n");
    o.push_str("/tools/                   @integrator @kernel-deputy\n");
    o.push_str("/crates/vleo-bus/         @integrator @kernel-deputy\n");
    o.push_str("/crates/vleo-modules/     @integrator @kernel-deputy\n");
    o.push_str("/crates/vleo-data/        @integrator @systems\n");
    o.push_str("/bundles/                 @integrator @systems\n");
    // The faces: the doors onto the one kernel.
    for face in [
        "vleo-server",
        "vleo-daemon",
        "vleo-app",
        "vleo-cli",
        "vleo-ffi",
        "vleo-py",
        "vleo-wasm",
        "vleo-method-wasm",
    ] {
        o.push_str(&format!(
            "/crates/{face}/{:w$}@integrator\n",
            "",
            w = 17 - face.len()
        ));
    }
    o.push_str("/docs/                    @integrator\n\n");
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

    // GitHub's one documented limit on this file, checked from
    // github/docs@main: "CODEOWNERS files must be under 3 MB in size. A
    // CODEOWNERS file over this limit will not be loaded, which means that
    // code owner information is not shown and the appropriate code owners
    // will not be requested to review changes in a pull request." No maximum
    // number of rules is stated anywhere in that page.
    //
    // Worth a check rather than a note because of how it fails: over the
    // limit the file is ignored in full and every pull request looks like one
    // with no owners, which is the same green as a pull request whose owners
    // all approved. One row costs about 66 bytes, so 3 MB is around 47,000 of
    // them and a 1,329-row tree uses 3% of it — but the tree is the thing that
    // grows, and this is the check that notices.
    const CODEOWNERS_LIMIT: usize = 3 * 1024 * 1024;
    if o.len() >= CODEOWNERS_LIMIT {
        return Err(format!(
            "the generated CODEOWNERS is {} bytes and GitHub ignores the file entirely at \
             3 MB. Over the limit every pull request shows no owners, which looks exactly \
             like a pull request whose owners approved. Consolidate rows onto wildcard \
             patterns per crate before writing this.",
            o.len()
        ));
    }
    fs::write(root.join("CODEOWNERS"), &o).map_err(|e| e.to_string())?;
    println!(
        "codeowners: {} lines, {} KB of the 3 MB GitHub will load ({}%)",
        o.lines().count(),
        o.len() / 1024,
        o.len() * 100 / CODEOWNERS_LIMIT
    );
    Ok(())
}

/// The variable register.
///
/// Generated from the sheets, like everything else. A register maintained by
/// hand drifts from the tree within a week, and then it is worse than absent:
/// somebody will trust it.
/// The de-risking narrative: every recorded change and every risk, laid out
/// from the sheets. Two files, one to read and one for a spreadsheet.
pub(crate) fn cmd_derisk(root: &Path, _args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
    let md = vleo_sheet::derisk::narrative_md(&tree);
    let csv = vleo_sheet::derisk::narrative_csv(&tree);
    let a = write_if_changed(&root.join("docs/DERISK_NARRATIVE.md"), &md)?;
    let b = write_if_changed(&root.join("docs/derisking.csv"), &csv)?;
    let (changes, starts) = vleo_sheet::derisk::narrative(&tree);
    let reg = vleo_sheet::derisk::register(&tree);
    println!(
        "derisk: {} change(s), {} starting belief(s), {} risk(s) registered, {} open — {}",
        changes.len(),
        starts.len(),
        reg.len(),
        reg.iter().filter(|r| r.now != "closed").count(),
        if a || b {
            "docs/DERISK_NARRATIVE.md and docs/derisking.csv written"
        } else {
            "nothing to write"
        }
    );
    Ok(())
}

/// The three role guides, rendered from the manual (vleo_sheet::guide).
pub(crate) fn cmd_guides(root: &Path) -> Result<(), String> {
    let m = vleo_sheet::manual::load(root)?;
    let version = workspace_version(root)?;
    let dir = root.join("docs").join("roles");
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for role in vleo_sheet::manual::ROLES {
        let html = vleo_sheet::guide::render(&m, role, &version)?;
        let out = dir.join(format!("{role}.html"));
        fs::write(&out, &html).map_err(|e| format!("{}: {e}", out.display()))?;
        println!("guides: {} — {} KB", out.display(), html.len() / 1024);
    }
    Ok(())
}

pub(crate) fn cmd_variables(root: &Path) -> Result<(), String> {
    let tree = load(root)?;
    let mut o = String::new();
    o.push_str("<!-- GENERATED by `cargo xtask variables`. Do not edit: the sheets are the source. -->\n\n");
    o.push_str("# The variable register\n\n");
    o.push_str(
        "> **Answer first.** Every variable in the tree — its unit, the range it is declared \
         valid over, the reason for each bound, and what reads it. Generated from the sheets.\n>\n\
         > **Kind:** reference · **For:** everyone\n\n",
    );
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
                .filter(|c| {
                    // A node reading `<this row>.<member>` reads this row. The
                    // register's whole purpose is saying what reads what, so
                    // matching input names against node ids alone would leave
                    // a set row's readers out of its own entry.
                    c.inputs.iter().any(|i| {
                        i.var == sh.id
                            || i.var.split_once('.').is_some_and(|(node, _)| node == sh.id)
                    })
                })
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
            if !sh.publishes.is_empty() {
                // This file claims to hold every variable in the tree, and a
                // set row answers with more than one. Each member carries its
                // own type, unit and range, decided separately and defended
                // separately, so each gets its own entry rather than a name in
                // a list under the row's range.
                o.push_str(&format!(
                    "- **publishes a set of {}** — this row's own answer, above, and the members below. Each is read as `{}.<member>`.\n",
                    1 + sh.publishes.len(),
                    sh.id
                ));
                for pb in &sh.publishes {
                    let pu = vleo_units::Unit::from_name(&pb.unit)
                        .map(|u| u.symbol())
                        .unwrap_or("?");
                    let readers: Vec<String> = tree
                        .sheets
                        .values()
                        .filter(|c| {
                            c.inputs
                                .iter()
                                .any(|i| i.var == format!("{}.{}", sh.id, pb.id))
                        })
                        .map(|c| format!("`{}`", c.id))
                        .collect();
                    o.push_str(&format!(
                        "\n#### `{id}.{m}` — {label}\n\n\
                         | | |\n|---|---|\n\
                         | symbol | `{sym}` |\n\
                         | type | `{ty}` |\n\
                         | unit | {pu} |\n\
                         | valid over | {lo} … {hi} {pu} |\n\n\
                         - **lower bound** — {rl}\n\
                         - **upper bound** — {ru}\n\
                         - **read by** — {by}\n",
                        id = sh.id,
                        m = pb.id,
                        label = pb.label,
                        sym = pb.symbol,
                        ty = pb.ty,
                        pu = pu,
                        lo = pb.lower,
                        hi = pb.upper,
                        rl = pb.reason_lower,
                        ru = pb.reason_upper,
                        by = if readers.is_empty() {
                            "nothing yet. A member nothing reads is a member the set does not need, or an oversight.".to_string()
                        } else {
                            readers.join(", ")
                        }
                    ));
                }
                o.push('\n');
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
