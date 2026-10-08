//! Reports over the tree: status, gaps, readiness, what answers and where it goes.

use super::*;

pub(super) fn cmd_status(root: &Path) -> Result<(), String> {
    let tree = read(root)?;
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
    evidence_debt(&tree);
    Ok(())
}

/// What each computing row's answer is checked against, by its best fixture.
///
/// The order is docs/NODE_AUTHORING.md's: an independent derivation first, then
/// a published source, another tool, a physical bound. A row with no fixture is
/// checked by nothing but itself. Counted over the specified rows that compute,
/// because a declared row has nothing to check.
fn evidence_debt(tree: &Tree) {
    const ORDER: [(&str, &str); 4] = [
        ("independent-derivation", "an independent derivation"),
        ("published-source", "a published source"),
        ("independent-tool", "another tool"),
        ("physical-bound", "a physical bound"),
    ];
    let computing: Vec<_> = tree
        .ordered()
        .into_iter()
        .filter(|s| !s.is_seeded() && !s.is_declared())
        .collect();
    let n = computing.len();
    let mut best = [0usize; ORDER.len()];
    for sh in &computing {
        if let Some(i) = ORDER
            .iter()
            .position(|(p, _)| sh.fixtures.iter().any(|f| f.provenance == *p))
        {
            best[i] += 1;
        }
    }
    let none = computing.iter().filter(|s| s.fixtures.is_empty()).count();
    let no_theory = computing.iter().filter(|s| s.theory.is_empty()).count();
    println!();
    println!("evidence, over the {n} specified rows that compute:");
    println!(
        "  {none:>4} of {n} have no fixture — nothing checks their answer but the code itself"
    );
    for ((_, words), c) in ORDER.iter().zip(best) {
        println!("  {c:>4} of {n} are checked, at best, against {words}");
    }
    println!("  {no_theory:>4} of {n} say nothing of where their relation comes from (no theory)");
}

pub(super) fn cmd_gap(root: &Path) -> Result<(), String> {
    let tree = read(root)?;
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

pub(super) fn cmd_declare(root: &Path, args: &[&str]) -> Result<(), String> {
    let id = args
        .first()
        .ok_or("usage: cargo xtask declare <node> [--source <path>]")?;
    let source = args
        .iter()
        .position(|a| *a == "--source")
        .and_then(|i| args.get(i + 1));
    let tree = load(root)?;
    let sh = tree.sheets.get(*id).ok_or_else(|| {
        format!("no node '{id}'. `cargo xtask new {id} --like <sibling>` starts one")
    })?;

    if args.contains(&"--json") {
        print!("{}", vleo_sheet::form::json(sh)?);
        return Ok(());
    }

    println!(
        "\x1b[1m{}\x1b[0m — {}",
        sh.id,
        if sh.label.is_empty() {
            "(no label yet)"
        } else {
            &sh.label
        }
    );
    if let Some(src) = source {
        println!("  drafting from {src}");
    }
    println!("  {}", sh.dir.join("node.toml").display());
    println!();

    // Grouped, in the order the form asks them, and the same grouping the
    // browser shows — both read `form::FIELDS`, so a field added to the sheet
    // appears in both faces or in neither.
    let asks = vleo_sheet::form::asks(sh);
    let mut open = 0usize;
    for g in vleo_sheet::form::groups() {
        let mine: Vec<&vleo_sheet::form::Ask> = asks
            .iter()
            .filter(|a| a.group == g && a.available)
            .collect();
        if mine.is_empty() {
            continue;
        }
        println!("  \x1b[2m{g}\x1b[0m");
        for a in mine {
            if a.open {
                open += 1;
                println!("  \x1b[33m?\x1b[0m  {}", a.ask);
                println!("     {} — without it: {}", a.field, a.why);
            } else {
                let v = vleo_sheet::form::value(sh, a.field);
                if v.trim().is_empty() {
                    // Blank and not blocking. It does not stop a scaffold being
                    // emitted, and saying so is the point: a row can generate,
                    // compile and still not be finished.
                    println!(
                        "  \x1b[36m-\x1b[0m  {} — blank, and does not block",
                        a.field
                    );
                } else {
                    println!("  \x1b[32m·\x1b[0m  {} = {}", a.field, truncate(&v, 68));
                }
            }
        }
    }

    // The derivation. Not a blocking field for generation either — an underived
    // relation still generates, still compiles and still has fixtures that pass
    // — but the row does not answer, so this is the first thing to say about a
    // function that has one and is silent. It is printed before authorship
    // because it is the stronger of the two: no derivation is no number, no
    // attribution is a number worth less.
    println!();
    if !sh.steps.is_empty() && sh.theory.is_empty() {
        println!("  \x1b[33m?\x1b[0m  where this relation came from — and THE ROW DOES NOT ANSWER until it is here");
        println!("     [theory] why, reading, and a [[theory.step]] per step. A function is");
        println!("     defined by its derivation, not by its expression and not by its citation:");
        println!(
            "     a relation an assistant invented carries a citation just as convincingly, and"
        );
        println!("     the expression is one line anybody can type. Until this is written the");
        println!("     resolver refuses the row and every reader of it blocks by name.");
    } else if !sh.steps.is_empty() {
        println!("  \x1b[32m·\x1b[0m  derived — the sheet says where the relation came from");
    }

    // Authorship. Not a blocking field for generation — a relation with no name
    // against it still generates — but a gap, so the node cannot reach H2.
    if sh.expression.trim().is_empty() {
    } else if sh.relation_by.trim().is_empty() {
        println!("  \x1b[33m?\x1b[0m  who supplied this relation, and when");
        println!(
            "     [maths] confirmed_by — an assistant may never supply mathematics, and without"
        );
        println!("     a name nothing can tell whether one did. It does not make the formula");
        println!("     right; it makes it somebody's, which is what H1b needs to be a review.");
    } else {
        println!(
            "  \x1b[32m·\x1b[0m  relation supplied by {}",
            sh.relation_by
        );
    }

    // The decisions that are not fields on the sheet but change what happens.
    println!(
        "  criticality = {} — {}",
        sh.criticality,
        if sh.criticality == "significant" {
            "two reviewers, and the hole filled twice by different model families"
        } else {
            "one reviewer; raise it on purpose, not by default"
        }
    );
    if !sh.migrated_from.is_empty() {
        println!(
            "  migrated_from = {} — its numbers go in parity.csv, never in fixtures.toml",
            sh.migrated_from
        );
    }
    if !sh.is_declared() {
        println!(
            "  {} numbered step(s), {} declared input(s)",
            sh.steps.len(),
            sh.inputs.len()
        );
    }

    // THE STATE, AND WHAT IS BETWEEN THE ROW AND MOVING IT. A row can answer
    // every question this form asks and still generate nothing, because nothing
    // is generated from a seeded row at all — which is the one thing about a
    // sheet that a reader is most likely to get wrong. The reasons are
    // `form::unpublishable`, the same list the browser shows.
    println!();
    if sh.is_seeded() {
        let why = vleo_sheet::form::unpublishable(sh);
        if why.is_empty() {
            println!("  \x1b[33m?\x1b[0m  seeded — nothing is generated from this row yet, and it is ready to publish");
        } else {
            println!("  \x1b[33m?\x1b[0m  seeded — nothing is generated from this row yet, and it is not ready:");
            for w in why {
                println!("     {w}");
            }
        }
    }

    println!();
    if open == 0 {
        println!("0 gaps open · ready to generate — `cargo xtask docs {id}`");
    } else {
        println!(
            "\x1b[33m{open} question(s) open.\x1b[0m Generation refuses until they are answered. \
             That refusal is the mechanism: it turns ambiguity from something an implementer \
             settles quietly into a blocking item on an engineer's screen."
        );
    }
    Ok(())
}

pub(super) fn truncate(s: &str, n: usize) -> String {
    let one = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() <= n {
        one
    } else {
        format!("{}…", one.chars().take(n - 1).collect::<String>())
    }
}

/// Has this node earned a person's attention yet?
///
/// The working model's change 2: a node tester that must pass before anybody is
/// asked to look. Both human reviews used to sit at the front of the work, and
/// nothing human sat where "done" is decided except a merge approval that is a
/// formality by then — so the person was being spent on specification, where
/// they are irreplaceable, and also on first-pass defect-finding, where a
/// machine is better, faster and free.
///
/// Three stages, in order, stopping at the first that is not clean:
///
///   1. the gate — does what exists pass
///   2. the gap pass — is anything the sheet promised absent
///   3. what criticality demands — a significant node gets a second
///      independent check, and a migrated one gets its parity grid
pub(super) fn cmd_ready(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = read(root)?;
    let only = args.first().copied();
    let mut asked = 0usize;
    let mut ready = 0usize;
    let mut held: Vec<(String, String, String)> = Vec::new();

    for sh in tree.ordered() {
        if let Some(o) = only {
            if sh.id != o {
                continue;
            }
        }
        if sh.is_seeded() && only.is_none() {
            continue;
        }
        asked += 1;
        let checks = gate::gate_node(sh, &tree);
        if let Some(c) = checks.iter().find(|c: &&gate::Check| c.failed()) {
            let why = match &c.verdict {
                gate::Verdict::Fail(w) => w.clone(),
                _ => String::new(),
            };
            held.push((
                sh.id.clone(),
                "the gate".into(),
                format!("{} — {why}", c.name),
            ));
            continue;
        }
        let gaps = emit::gap_pass(sh, &vleo_sheet::load::read_holes(&sh.dir));
        if !gaps.is_empty() {
            held.push((sh.id.clone(), "the gap pass".into(), gaps.join("; ")));
            continue;
        }
        if sh.criticality == "significant" && sh.fixtures.len() < 2 {
            held.push((
                sh.id.clone(),
                "criticality".into(),
                "significant, and fewer than two independent checks behind it".into(),
            ));
            continue;
        }
        ready += 1;
    }

    if asked == 0 {
        return Err(format!("no node matched '{}'", only.unwrap_or("")));
    }
    // Named individually for one node, counted by kind for the tree. A list of
    // 250 lines is a list nobody reads, and the useful question at tree scale is
    // which *kind* of thing is holding the most.
    if only.is_some() {
        for (id, stage, why) in &held {
            println!("  \x1b[33mhold\x1b[0m {id} — {stage}: {why}");
        }
    }
    println!(
        "ready: {ready} of {asked} node(s) have passed every machine stage and are waiting on H2"
    );
    if !held.is_empty() {
        let mut by: BTreeMap<&str, usize> = BTreeMap::new();
        for (_, _, why) in &held {
            // The gap pass joins its findings with "; ", and a node is usually
            // held by more than one. Counting the node once per kind says what
            // to fix; counting nodes says only that there is work.
            for one in why.split("; ") {
                let kind = match one {
                    w if w.contains("no fixture") => {
                        "no fixture — nothing outside this code has agreed with it"
                    }
                    w if w.contains("nobody's name against it") => {
                        "the relation has nobody's name against it"
                    }
                    w if w.contains("seeded") => "seeded, not yet specified",
                    w if w.contains("no source") => "no source cited",
                    w if w.contains("parity") || w.contains("migrated") => {
                        "migrated, and no parity grid beside it"
                    }
                    w if w.contains("significant") => {
                        "significant, with fewer than two checks behind it"
                    }
                    w if w.contains("domain edge") || w.contains("range edge") => {
                        "a declared range edge with no case behind it"
                    }
                    _ => "other",
                };
                *by.entry(kind).or_default() += 1;
            }
        }
        println!("{} held, by what is holding them:", held.len());
        let mut rows: Vec<(&&str, &usize)> = by.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1));
        for (kind, n) in rows {
            println!("  {n:>5}  {kind}");
        }
        println!(
            "A person asked to look at these is being asked to find what a machine finds free."
        );
    }
    Ok(())
}

pub(super) fn cmd_variables(root: &Path) -> Result<(), String> {
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

/// Which rows answer, which do not, and what is in the way.
///
/// The question a person actually has in front of the tool is "is this number
/// real", and until now nothing answered it in one place: a row with an
/// expression and a citation printed a number exactly like a row somebody had
/// worked out. The two are not the same, and the difference is the derivation
/// — see `NodeDef::is_defined`.
///
/// Three states, and the third is the one worth having:
///
///   ACTIVE      it answers. An input, which is defined by carrying a value, or
///               a function whose relation the sheet derives, reading only
///               active rows.
///   UNDEFINED   a function whose relation is stated and never derived. Its own
///               fault, and one sheet edit away from being fixed.
///   BLOCKED     defined in itself; something it reads is not. Named, because
///               "not active" without the name is a dead end.
///
/// It is computed from the sheets, so it is answerable on a tree that cannot
/// run — which is most of a tree for most of a programme.
/// Whether a row answers, and if not, what is in the way.
///
/// Lifted out of `cmd_active` when `cmd_reach` needed the same answer. Two
/// walks computing "does this row answer" would be two definitions of it, and
/// the one that drifts is always the copy nobody is looking at.
#[derive(Clone)]
pub(super) enum Verdict {
    Active,
    Seeded,
    /// Retired. It still answers — that is deliberate, so a consumer that has
    /// not migrated keeps working — but it is not work in progress and nothing
    /// is supposed to read it. Counted apart from `Active` because folding the
    /// two overstates what the tree currently produces, and because a retired
    /// row with no consumer is the intended end state rather than a finding.
    Retired,
    Undefined,
    BlockedBy(String),
}

/// Every row's verdict, computed once from the sheets.
pub(super) fn active_verdicts(tree: &Tree) -> BTreeMap<&str, Verdict> {
    // Undefined in itself. A seeded row is not undefined — it is unwritten, and
    // the tree already counts those separately. Conflating the two would report
    // 1076 rows as a derivation problem when they are a nothing-has-been-written
    // problem.
    let mut undefined: BTreeSet<&str> = BTreeSet::new();
    for sh in tree.ordered() {
        if sh.is_seeded() {
            continue;
        }
        if !sh.steps.is_empty() && sh.theory.is_empty() {
            undefined.insert(sh.id.as_str());
        }
    }

    // Walk each row's closure. The first row in the way is the one reported:
    // a person fixes one thing at a time, and naming the whole set of ancestors
    // is a list nobody reads.
    fn walk<'a>(
        id: &'a str,
        tree: &'a vleo_sheet::load::Tree,
        undefined: &BTreeSet<&str>,
        verdict: &mut BTreeMap<&'a str, Verdict>,
        on_stack: &mut BTreeSet<String>,
    ) -> Verdict {
        if let Some(v) = verdict.get(id) {
            return v.clone();
        }
        // A declared cycle is walked once; treat a revisit as satisfied rather
        // than recursing forever. Whether the cycle converges is the resolver's
        // question, not this one's.
        if !on_stack.insert(id.to_string()) {
            return Verdict::Active;
        }
        let sh = match tree.sheets.get(id) {
            Some(s) => s,
            None => {
                on_stack.remove(id);
                return Verdict::Active;
            }
        };
        let v = if sh.is_seeded() {
            Verdict::Seeded
        } else if sh.state == "deprecated" {
            Verdict::Retired
        } else if undefined.contains(id) {
            Verdict::Undefined
        } else {
            let mut blocker = None;
            for inp in &sh.inputs {
                // A variable id is a node id or `<node id>.<extra>`, and a node
                // id never contains a dot — so the producer is everything
                // before the first one. Resolved against the tree rather than
                // trusted, because an input naming a row that does not exist is
                // assembly's refusal to make, not this command's.
                let name = producer_of(&inp.var);
                let Some((key, _)) = tree.sheets.get_key_value(name) else {
                    continue;
                };
                let up: &'a str = key.as_str();
                if up == id {
                    continue;
                }
                match walk(up, tree, undefined, verdict, on_stack) {
                    // Retired still answers, so a consumer of one is not
                    // blocked. Whether it should still be reading it is what
                    // `Retirement::Wrong` and the deprecation notice are for.
                    Verdict::Active | Verdict::Retired => {}
                    Verdict::Seeded | Verdict::Undefined => {
                        blocker = Some(up.to_string());
                        break;
                    }
                    Verdict::BlockedBy(b) => {
                        blocker = Some(b);
                        break;
                    }
                }
            }
            match blocker {
                Some(b) => Verdict::BlockedBy(b),
                None => Verdict::Active,
            }
        };
        on_stack.remove(id);
        verdict.insert(sh.id.as_str(), v.clone());
        v
    }
    let mut verdict: BTreeMap<&str, Verdict> = BTreeMap::new();
    let ids: Vec<&str> = tree.ordered().iter().map(|s| s.id.as_str()).collect();
    for id in &ids {
        let mut on_stack = BTreeSet::new();
        walk(id, tree, &undefined, &mut verdict, &mut on_stack);
    }
    verdict
}

/// Every row whose number reaches a KPI closure.
///
/// Computed by walking BACKWARDS from the KPIs once, rather than asking each
/// row "can you reach one" and memoising the answer. The forward version is the
/// obvious one and it is wrong on a cycle: it must seed `false` before
/// recursing so a loop terminates, and that provisional `false` then gets
/// memoised for any row whose real answer arrived later by another edge. A row
/// inside a declared cycle that genuinely reaches a KPI reported that it did
/// not, which is the one direction this report must never be wrong in — it
/// would invent unread work that is being read.
///
/// Backwards there is no such subtlety: reachability is a set, a seen-set ends
/// every walk, and the answer does not depend on which row was asked first.
pub(super) fn reaching_kpi<'a>(
    parents: &BTreeMap<&'a str, Vec<&'a str>>,
    kpis: &BTreeSet<&'a str>,
) -> BTreeSet<&'a str> {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut stack: Vec<&str> = kpis.iter().copied().collect();
    while let Some(n) = stack.pop() {
        if !seen.insert(n) {
            continue;
        }
        if let Some(ps) = parents.get(n) {
            stack.extend(ps.iter().copied());
        }
    }
    seen
}

/// How much answering work stands behind a row — its upstream closure, counting
/// only rows that answer. A terminal row with fifty rows behind it and one with
/// none are the same line in a count and very different findings.
pub(super) fn work_behind(
    id: &str,
    parents: &BTreeMap<&str, Vec<&str>>,
    answers: &dyn Fn(&str) -> bool,
) -> usize {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut stack = vec![id];
    while let Some(n) = stack.pop() {
        if !seen.insert(n) {
            continue;
        }
        if let Some(ps) = parents.get(n) {
            stack.extend(ps.iter().copied());
        }
    }
    seen.remove(id);
    seen.into_iter().filter(|n| answers(n)).count()
}

/// Where each answer GOES — the other half of `active`.
///
/// `active` asks whether a row produces a number. This asks what the number is
/// for, and they are not the same question: a subsystem can answer on every row
/// it has and still be wired to nothing.
///
/// The measure is the KPI closure, because that is what this tree says it is
/// for — twelve promises to a customer, and every other row exists to move one
/// of them. So an answer that reaches no KPI is not wrong, and it is not a
/// defect in the row; it is work the design is not currently reading. Stating
/// that is the whole job here. Deciding what to do about it is not: a crossing
/// nothing reads can be wired up, retired, or kept as a deliberate reference
/// beside the design point, and which of those is right is a design decision
/// with a person's name on it.
///
/// TERMINAL is reported beside it and is the sharper number: a row that answers
/// and has no consumer at all. Ranked by how much answering work stands behind
/// it, because one terminal row with fifty rows behind it and fifty terminal
/// rows with nothing behind them are very different findings and the count
/// alone cannot tell them apart.
///
/// A CONCLUSION is excluded from that list by kind, not by name. An achieved
/// row is a margin and a KPI row is a promise; both are the end of a chain, so
/// having no consumer is what they are for. Counting those as unread work would
/// put five correct rows at the top of a list of findings, and a check that
/// cries wolf on its own best rows is a check people stop reading.
pub(super) fn cmd_reach(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = read(root)?;
    let only = args.iter().find(|a| !a.starts_with("--")).copied();
    let verdict = active_verdicts(&tree);
    let answers = |id: &str| matches!(verdict.get(id), Some(Verdict::Active));

    // The derivation graph, read the other way. Declared by the consumer, so
    // the consumer list is derived here and never stored — the same rule the
    // face follows, for the same reason.
    let mut consumers: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for sh in tree.ordered() {
        for inp in &sh.inputs {
            if let Some((k, _)) = tree.sheets.get_key_value(producer_of(&inp.var)) {
                if k.as_str() != sh.id.as_str() {
                    consumers
                        .entry(k.as_str())
                        .or_default()
                        .push(sh.id.as_str());
                }
            }
        }
    }

    let kpis: BTreeSet<&str> = tree
        .ordered()
        .iter()
        .filter(|s| s.kind == "kpi")
        .map(|s| s.id.as_str())
        .collect();

    // Parents: the same edges as `consumers`, reversed. Both walks use it.
    let mut parents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for sh in tree.ordered() {
        for inp in &sh.inputs {
            if let Some((k, _)) = tree.sheets.get_key_value(producer_of(&inp.var)) {
                if k.as_str() != sh.id.as_str() {
                    parents.entry(sh.id.as_str()).or_default().push(k.as_str());
                }
            }
        }
    }

    let reaches = reaching_kpi(&parents, &kpis);

    let rows: Vec<&vleo_sheet::model::Sheet> = tree
        .ordered()
        .into_iter()
        .filter(|s| only.is_none_or(|sub| s.subsystem == sub))
        .collect();
    if rows.is_empty() {
        return Err(format!(
            "no rows in subsystem '{}'",
            only.unwrap_or("<none>")
        ));
    }

    let mut by_sub: BTreeMap<&str, [usize; 3]> = BTreeMap::new();
    for sh in &rows {
        if !answers(&sh.id) {
            continue;
        }
        let e = by_sub.entry(sh.subsystem.as_str()).or_default();
        e[0] += 1;
        if reaches.contains(sh.id.as_str()) {
            e[1] += 1;
        }
        if consumers.get(sh.id.as_str()).is_none_or(|c| c.is_empty()) {
            e[2] += 1;
        }
    }
    println!(
        "{:<12} {:>7} {:>12} {:>10}",
        "subsystem", "answer", "reach a KPI", "terminal"
    );
    let mut order: Vec<&&str> = by_sub.keys().collect();
    order.sort_by_key(|s| std::cmp::Reverse(by_sub[*s][0]));
    for s in order {
        let c = by_sub[*s];
        println!("{:<12} {:>7} {:>12} {:>10}", s, c[0], c[1], c[2]);
    }
    let tot = |i: usize| by_sub.values().map(|c| c[i]).sum::<usize>();
    println!(
        "{:<12} {:>7} {:>12} {:>10}\n",
        "total",
        tot(0),
        tot(1),
        tot(2)
    );

    // A subsystem that answers and reaches nothing is the finding worth a
    // paragraph rather than a row, so it gets one, with the crossing named.
    for (s, c) in &by_sub {
        if c[0] == 0 || c[1] > 0 {
            continue;
        }
        println!(
            "{s} answers on {} row(s) and reaches no KPI closure. Its work is computed \
             and not read.",
            c[0]
        );
        for sh in tree.ordered() {
            if sh.subsystem == *s && !sh.crosses_to.is_empty() {
                let cs = consumers.get(sh.id.as_str()).cloned().unwrap_or_default();
                println!("  crossing  {} -> {}", sh.id, sh.crosses_to);
                for up in &cs {
                    let onward = consumers.get(up).cloned().unwrap_or_default();
                    println!(
                        "    {:<38} read by {}",
                        up,
                        if onward.is_empty() {
                            "NOBODY".to_string()
                        } else {
                            onward.join(", ")
                        }
                    );
                }
            }
        }
        println!();
    }

    let is_conclusion = |k: &str| k == "achieved" || k == "kpi";
    let terminal: Vec<&&vleo_sheet::model::Sheet> = rows
        .iter()
        .filter(|s| answers(&s.id))
        .filter(|s| consumers.get(s.id.as_str()).is_none_or(|c| c.is_empty()))
        .collect();
    let (concl, mut mid): (
        Vec<&vleo_sheet::model::Sheet>,
        Vec<&vleo_sheet::model::Sheet>,
    ) = terminal
        .iter()
        .map(|s| **s)
        .partition(|s| is_conclusion(s.kind.as_str()));
    mid.sort_by_key(|s| {
        (
            std::cmp::Reverse(work_behind(s.id.as_str(), &parents, &answers)),
            s.id.clone(),
        )
    });
    if !mid.is_empty() {
        println!("unread — it answers, it is not a conclusion, and nothing reads it:");
        for s in mid.iter().take(12) {
            println!(
                "  {:<42} {:>3} answering row(s) behind it",
                s.id,
                work_behind(s.id.as_str(), &parents, &answers)
            );
        }
        if mid.len() > 12 {
            println!("  … and {} more", mid.len() - 12);
        }
        println!();
    }
    if !concl.is_empty() {
        println!(
            "{} conclusion(s) also have no consumer, which is what a margin and a KPI are \
             for: {}",
            concl.len(),
            concl
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!();
    }
    println!(
        "{} of {} answering row(s) reach one of the {} KPI closure(s). Reaching none is \
         not a defect in a row — it is the design not reading it, and what to do about \
         that is a decision with a person's name on it.",
        tot(1),
        tot(0),
        kpis.len()
    );
    Ok(())
}

pub(super) fn cmd_active(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = read(root)?;
    let only = args.iter().find(|a| !a.starts_with("--")).copied();
    let verdict = active_verdicts(&tree);
    // The rows that answer, one id per line — for a script, or for the list of
    // rows whose plain words and first belief are the next thing to write.
    if args.contains(&"--names") {
        for sh in tree.ordered() {
            if only.is_none_or(|sub| sh.subsystem == sub)
                && matches!(verdict.get(sh.id.as_str()), Some(Verdict::Active))
            {
                println!("{}", sh.id);
            }
        }
        return Ok(());
    }

    // How much each undefined row is costing, measured rather than guessed:
    // the rows that name it as their blocker. This is the order to fix them in.
    let mut cost: BTreeMap<&str, usize> = BTreeMap::new();
    for sh in tree.ordered() {
        if let Some(Verdict::BlockedBy(b)) = verdict.get(sh.id.as_str()) {
            if let Some((k, _)) = tree.sheets.get_key_value(b.as_str()) {
                *cost.entry(k.as_str()).or_default() += 1;
            }
        }
    }

    let rows: Vec<&vleo_sheet::model::Sheet> = tree
        .ordered()
        .into_iter()
        .filter(|s| only.is_none_or(|sub| s.subsystem == sub))
        .collect();
    if rows.is_empty() {
        return Err(format!(
            "no rows in subsystem '{}'",
            only.unwrap_or("<none>")
        ));
    }

    let mut by_sub: BTreeMap<&str, [usize; 5]> = BTreeMap::new();
    for sh in &rows {
        let e = by_sub.entry(sh.subsystem.as_str()).or_default();
        match verdict.get(sh.id.as_str()) {
            Some(Verdict::Active) => e[0] += 1,
            Some(Verdict::Undefined) => e[1] += 1,
            Some(Verdict::BlockedBy(_)) => e[2] += 1,
            Some(Verdict::Retired) => e[4] += 1,
            _ => e[3] += 1,
        }
    }
    println!(
        "{:<12} {:>7} {:>10} {:>10} {:>8} {:>8}",
        "subsystem", "active", "undefined", "blocked", "seeded", "retired"
    );
    for (s, c) in &by_sub {
        println!(
            "{:<12} {:>7} {:>10} {:>10} {:>8} {:>8}",
            s, c[0], c[1], c[2], c[3], c[4]
        );
    }
    let tot = |i: usize| by_sub.values().map(|c| c[i]).sum::<usize>();
    println!(
        "{:<12} {:>7} {:>10} {:>10} {:>8} {:>8}\n",
        "total",
        tot(0),
        tot(1),
        tot(2),
        tot(3),
        tot(4)
    );

    if tot(1) > 0 {
        println!("undefined — the relation is stated and never derived:");
        let mut list: Vec<(&str, usize)> = rows
            .iter()
            .filter(|s| matches!(verdict.get(s.id.as_str()), Some(Verdict::Undefined)))
            .map(|s| (s.id.as_str(), cost.get(s.id.as_str()).copied().unwrap_or(0)))
            .collect();
        // Worst first: a row nothing reads is a different job from one that is
        // standing in front of forty.
        list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        for (id, n) in list.iter().take(24) {
            let sh = &tree.sheets[*id];
            println!(
                "  {:<40} {:>3} row(s) wait on it   {}",
                id, n, sh.expression
            );
        }
        if list.len() > 24 {
            println!("  … and {} more", list.len() - 24);
        }
        println!();
    }

    // THE INVERSE, AND IT IS THE ONE PEOPLE ACTUALLY ASK FOR: not what is
    // missing, but what has been defined. "Has anything been given mathematics
    // that should not have been" is a fair question to ask of a tree seven
    // agents write into, and answering it should not require reading git
    // history — which is what it took the first time somebody asked.
    let defined: Vec<&&vleo_sheet::model::Sheet> = rows
        .iter()
        // Retired excluded, like everywhere else in this command: a deprecated
        // row still answers, deliberately, but it is not work in progress and
        // counting it here reads as five more defined functions than there are.
        .filter(|s| {
            !s.steps.is_empty() && !s.theory.is_empty() && !s.is_seeded() && s.state != "deprecated"
        })
        .collect();
    if !defined.is_empty() {
        let mut per: BTreeMap<&str, usize> = BTreeMap::new();
        for s in &defined {
            *per.entry(s.subsystem.as_str()).or_default() += 1;
        }
        let mut order: Vec<(&&str, &usize)> = per.iter().collect();
        order.sort_by_key(|(s, n)| (std::cmp::Reverse(**n), **s));
        println!(
            "defined — a function whose relation the sheet derives ({} in all):",
            defined.len()
        );
        println!(
            "  {}",
            order
                .iter()
                .map(|(s, n)| format!("{s} {n}"))
                .collect::<Vec<_>>()
                .join(" · ")
        );
        if args.contains(&"--defined") {
            for s in &defined {
                println!("    {:<42} {}", s.id, s.subsystem);
            }
        } else {
            println!("  --defined names them");
        }
        println!();
    }

    println!(
        "{} of {} rows answer. A function is defined by its derivation, not by its\n\
         expression and not by its citation — `cargo xtask declare <node>` says what\n\
         one is missing, and `confirm --list` is the separate question of who has\n\
         read the relation against its source.",
        tot(0),
        rows.len()
    );
    Ok(())
}
