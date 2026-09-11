//! The gate, and the assembly validations.
//!
//! One gate binary, called from three places: the authoring hook, the pipeline,
//! and by hand on the test machine. Where the hook runs one script and the
//! pipeline runs another they drift within a month and the hook becomes
//! theatre, so there is one of them.
//!
//! The checks are ordered by cost, not by importance. The first seven run on
//! every edit in about a second; the rest run on demand and in the pipeline.

use crate::emit;
use crate::load::{read_holes, Tree};
use crate::model::*;
use std::collections::{BTreeMap, BTreeSet};

/// Format a candidate the way the generator does before writing it, so the
/// regeneration check asks "did the content drift" rather than "has the
/// formatter run".
///
/// When `rustfmt` is not on the path, or refuses the candidate, this returns
/// the text **unchanged**. It must never return anything but valid source: this
/// value is written to disk as well as compared, and an earlier version
/// returned a whitespace-collapsed form on failure — which put every generated
/// module on one line, where the first `//` comment swallowed the rest of the
/// file. The comparison is allowed to be weaker than the formatting; the
/// content is not allowed to be wrong.
pub fn formatted(text: &str) -> String {
    use std::io::Write;
    let dir = std::env::temp_dir().join("vleo-gate");
    let _ = std::fs::create_dir_all(&dir);
    let p = dir.join(format!("candidate-{}.rs", crate::fnv1a(text)));
    if let Ok(mut f) = std::fs::File::create(&p) {
        if f.write_all(text.as_bytes()).is_ok() {
            drop(f);
            // `--skip-children` because a generated `mod.rs` declares submodules
            // that do not exist beside a temporary file, and resolving them is
            // not what is being asked here.
            let ok = std::process::Command::new("rustfmt")
                .args(["--edition", "2021", "--quiet", "--skip-children"])
                .arg(&p)
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                if let Ok(out) = std::fs::read_to_string(&p) {
                    let _ = std::fs::remove_file(&p);
                    return out;
                }
            }
            let _ = std::fs::remove_file(&p);
        }
    }
    text.to_string()
}

/// Collapse whitespace. The weaker comparison, used only when `rustfmt` is
/// absent.
fn normalise(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    /// Advisory. The gap pass reports what the sheet promised and nothing yet
    /// covers; its output is a list, not a verdict. It blocks the second human
    /// review rather than the build, because a tree that is 40% written should
    /// not have a red build every night — that is how a team learns to ignore
    /// one.
    Note(String),
    Fail(String),
}

#[derive(Clone, Debug)]
pub struct Check {
    pub name: &'static str,
    pub verdict: Verdict,
}

impl Check {
    fn pass(name: &'static str) -> Check {
        Check {
            name,
            verdict: Verdict::Pass,
        }
    }
    fn fail(name: &'static str, why: String) -> Check {
        Check {
            name,
            verdict: Verdict::Fail(why),
        }
    }
    fn note(name: &'static str, why: String) -> Check {
        Check {
            name,
            verdict: Verdict::Note(why),
        }
    }
    pub fn is_note(&self) -> bool {
        matches!(self.verdict, Verdict::Note(_))
    }
    pub fn failed(&self) -> bool {
        matches!(self.verdict, Verdict::Fail(_))
    }
}

/// The per-node checks.
pub fn gate_node(sh: &Sheet, tree: &Tree) -> Vec<Check> {
    let mut out = Vec::new();
    let holes = read_holes(&sh.dir);

    // A seeded row is checked for the four things a seed is responsible for and
    // nothing else. Everything a person has yet to write is the gap pass's to
    // report, and a gate that refuses most of a tree for six months is a gate
    // nobody reads.
    if sh.is_seeded() {
        let mut missing = Vec::new();
        for (name, v) in [
            ("id", &sh.id),
            ("label", &sh.label),
            ("parent", &sh.parent),
            ("owner", &sh.owner),
        ] {
            if v.trim().is_empty() {
                missing.push(name);
            }
        }
        out.push(if missing.is_empty() {
            Check::pass("seeded")
        } else {
            Check::fail(
                "seeded",
                format!("a seeded row still needs: {}", missing.join(", ")),
            )
        });
        out.push(if tree.groups.contains_key(&sh.parent) {
            Check::pass("parent")
        } else {
            Check::fail("parent", format!("'{}' is not a group", sh.parent))
        });
        let gaps = emit::gap_pass(sh, &holes);
        out.push(if gaps.is_empty() {
            Check::pass("gap-pass")
        } else {
            Check::note("gap-pass", gaps.join("; "))
        });
        return out;
    }

    // 1 — the sheet validates; no required field is blank.
    let mut missing = Vec::new();
    for (name, v) in [
        ("id", &sh.id),
        ("label", &sh.label),
        ("owner", &sh.owner),
        ("question", &sh.question),
        ("expression", &sh.expression),
        ("source", &sh.source),
        ("type", &sh.ty),
        ("unit", &sh.unit),
        ("symbol", &sh.symbol),
        ("reason_lower", &sh.reason_lower),
        ("reason_upper", &sh.reason_upper),
    ] {
        if v.trim().is_empty() {
            missing.push(name);
        }
    }
    out.push(if missing.is_empty() {
        Check::pass("schema")
    } else {
        Check::fail(
            "schema",
            format!("required fields blank: {}", missing.join(", ")),
        )
    });

    // 2 — a computed node declares at least one input.
    out.push(if sh.is_declared() || !sh.inputs.is_empty() {
        Check::pass("inputs")
    } else {
        Check::fail(
            "inputs",
            "a computed node with no declared input claims to compute something from nothing"
                .into(),
        )
    });

    // 3 — a declared value names a source and a confirmation.
    out.push(
        if !sh.is_declared() || (sh.value.is_some() && !sh.confirmed_by.trim().is_empty()) {
            Check::pass("declared-value")
        } else {
            Check::fail(
                "declared-value",
                "a declared value needs a number, a source and who confirmed it".into(),
            )
        },
    );

    // 4 — every input resolves, and the declared type agrees with the producer.
    let mut bad = Vec::new();
    for i in &sh.inputs {
        match tree.sheets.get(&i.var) {
            None => bad.push(format!("'{}' names no node", i.var)),
            Some(p) if p.ty != i.ty => bad.push(format!(
                "'{}' expects {} but {} publishes {}",
                i.binding, i.ty, p.id, p.ty
            )),
            Some(p) if p.state == "deprecated" => bad.push(format!(
                "'{}' is deprecated and may not be a new dependency",
                p.id
            )),
            _ => {}
        }
    }
    out.push(if bad.is_empty() {
        Check::pass("contract")
    } else {
        Check::fail("contract", bad.join("; "))
    });

    // 5 — every source resolves to an entry in sources/.
    let mut unresolved = BTreeSet::new();
    if !tree.sources.contains_key(&sh.source) {
        unresolved.insert(sh.source.clone());
    }
    for f in &sh.fixtures {
        if !tree.sources.contains_key(&f.source) {
            unresolved.insert(f.source.clone());
        }
    }
    out.push(if unresolved.is_empty() {
        Check::pass("sources")
    } else {
        Check::fail(
            "sources",
            format!(
                "these cite nothing in sources/: {}",
                unresolved.into_iter().collect::<Vec<_>>().join(", ")
            ),
        )
    });

    // 6 — regenerate and compare. A hand edit outside a hole fails here, which
    //     is what makes the generated region genuinely owned by the generator.
    let mut drift = Vec::new();
    for (name, want) in [
        ("model.rs", emit::model_rs(sh, &holes)),
        ("contract.rs", emit::contract_rs(sh)),
        ("mod.rs", emit::mod_rs(sh)),
        ("evidence.rs", emit::evidence_rs(sh)),
    ] {
        let p = sh.dir.join(name);
        match std::fs::read_to_string(&p) {
            // The committed file is formatted; what the generator emits is not
            // yet. Comparing after normalising whitespace asks the question the
            // check is actually for — did the *content* drift — rather than
            // whether the formatter has run.
            Ok(got) if got == formatted(&want) || normalise(&got) == normalise(&want) => {}
            Ok(_) => drift.push(name),
            Err(_) => drift.push(name),
        }
    }
    out.push(if drift.is_empty() {
        Check::pass("regenerate")
    } else {
        Check::fail(
            "regenerate",
            format!(
                "{} differ from what the sheet generates — run `cargo xtask docs`",
                drift.join(", ")
            ),
        )
    });

    // 7 — the gap pass.
    let gaps = emit::gap_pass(sh, &holes);
    out.push(if gaps.is_empty() {
        Check::pass("gap-pass")
    } else {
        Check::note("gap-pass", gaps.join("; "))
    });

    // 8 — fixture provenance. The one rule the evidence model rests on.
    let mut badfx = Vec::new();
    for f in &sh.fixtures {
        if !matches!(
            f.provenance.as_str(),
            "independent-derivation" | "published-source" | "independent-tool" | "physical-bound"
        ) {
            badfx.push(format!("'{}' has provenance '{}'", f.label, f.provenance));
        }
        if f.tolerance <= 0.0 {
            badfx.push(format!("'{}' has a non-positive tolerance", f.label));
        }
    }
    out.push(if badfx.is_empty() {
        Check::pass("provenance")
    } else {
        Check::fail(
            "provenance",
            format!(
                "an expected value may not come from the code under test: {}",
                badfx.join("; ")
            ),
        )
    });

    // 9 — the declared limits are ordered and the declared value sits inside them.
    let mut dom = Vec::new();
    if sh.lower >= sh.upper {
        dom.push("the lower limit is not below the upper".to_string());
    }
    if let Some(v) = sh.value {
        if v < sh.lower || v > sh.upper {
            dom.push(format!(
                "the declared value {v} is outside its own declared range"
            ));
        }
    }
    out.push(if dom.is_empty() {
        Check::pass("domain")
    } else {
        Check::fail("domain", dom.join("; "))
    });

    // 10 — the portable maths rule. The kernel crates may not reach the
    //      platform maths library, or cross-face agreement fails on night one
    //      for a reason that is not a defect.
    let mut leaks = Vec::new();
    for (n, body) in &holes {
        for bad in [
            ".sin()", ".cos()", ".exp()", ".ln()", ".powf(", ".sqrt()", ".atan2(", ".tan()",
            ".log10(",
        ] {
            if body.contains(bad) {
                leaks.push(format!("hole {n} calls {bad} — route it through pmath"));
            }
        }
    }
    out.push(if leaks.is_empty() {
        Check::pass("portable-maths")
    } else {
        Check::fail("portable-maths", leaks.join("; "))
    });

    out
}

/// The assembly validations. These can only be asked where the whole tree is
/// visible, which is why they belong here and not in a per-node gate.
pub fn validate_tree(tree: &Tree) -> Vec<Check> {
    let mut out = Vec::new();

    // V1 — every edge endpoint names a row that exists.
    let mut dangling = Vec::new();
    for sh in tree.ordered() {
        for i in &sh.inputs {
            if !tree.sheets.contains_key(&i.var) {
                dangling.push(format!("{} -> {}", sh.id, i.var));
            }
        }
        for k in &sh.kpis {
            if !tree.sheets.contains_key(k) {
                dangling.push(format!("{} contributes to {}", sh.id, k));
            }
        }
    }
    out.push(if dangling.is_empty() {
        Check::pass("V1 edge endpoints exist")
    } else {
        Check::fail("V1 edge endpoints exist", dangling.join(", "))
    });

    // V2 — no edge is declared twice. Each edge is declared exactly once, by
    //      the end the edge changes.
    let mut dup = Vec::new();
    for sh in tree.ordered() {
        let mut seen = BTreeSet::new();
        for i in &sh.inputs {
            if !seen.insert(i.var.clone()) {
                dup.push(format!("{} reads {} twice", sh.id, i.var));
            }
        }
    }
    out.push(if dup.is_empty() {
        Check::pass("V2 no duplicate edges")
    } else {
        Check::fail("V2 no duplicate edges", dup.join(", "))
    });

    // V3 — every node's parent resolves to a layer group.
    let mut orphans = Vec::new();
    for sh in tree.ordered() {
        if !tree.groups.contains_key(&sh.parent) {
            orphans.push(format!(
                "{} hangs under '{}', which is not a group",
                sh.id, sh.parent
            ));
        }
    }
    out.push(if orphans.is_empty() {
        Check::pass("V3 parents resolve")
    } else {
        Check::fail("V3 parents resolve", orphans.join(", "))
    });

    // V4 — every cycle in the derivation graph is declared in a case. A cycle
    //      is a cross-branch property: two individually clean branches can form
    //      one, so this belongs to the merged state and never to a per-node gate.
    let declared: BTreeSet<String> = tree
        .cases
        .values()
        .flat_map(|c| c.cycles.iter())
        .flat_map(|cy| cy.nodes.iter().cloned())
        .collect();
    let cycles = find_cycles(tree, &declared);
    out.push(if cycles.is_empty() {
        Check::pass("V4 every cycle is declared")
    } else {
        Check::fail(
            "V4 every cycle is declared",
            format!("undeclared loop: {}", cycles.join(" -> ")),
        )
    });

    // V5 — every computed node declares at least one input.
    let bad: Vec<String> = tree
        .ordered()
        .iter()
        .filter(|s| !s.is_declared() && !s.is_seeded() && s.inputs.is_empty())
        .map(|s| s.id.clone())
        .collect();
    out.push(if bad.is_empty() {
        Check::pass("V5 computed nodes have inputs")
    } else {
        Check::fail("V5 computed nodes have inputs", bad.join(", "))
    });

    // V6 — no two siblings resolve to the same folder name.
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut collide = Vec::new();
    for sh in tree.ordered() {
        if !seen.insert((sh.crate_name.clone(), sh.folder.clone())) {
            collide.push(format!("{}/{}", sh.crate_name, sh.folder));
        }
    }
    out.push(if collide.is_empty() {
        Check::pass("V6 no folder collisions")
    } else {
        Check::fail("V6 no folder collisions", collide.join(", "))
    });

    // V7 — every layer group has an owner. An ownerless subsystem routes
    //      reviews nowhere, silently.
    let un: Vec<String> = tree
        .groups
        .values()
        .filter(|g| g.owner.trim().is_empty())
        .map(|g| g.id.clone())
        .collect();
    out.push(if un.is_empty() {
        Check::pass("V7 every layer has an owner")
    } else {
        Check::fail("V7 every layer has an owner", un.join(", "))
    });

    // V8 — every source reference resolves, and nothing depends on a
    //      superseded one without being listed.
    let mut unresolved = BTreeSet::new();
    let mut superseded = BTreeSet::new();
    for sh in tree.ordered() {
        if sh.is_seeded() {
            continue;
        }
        for s in std::iter::once(&sh.source).chain(sh.fixtures.iter().map(|f| &f.source)) {
            match tree.sources.get(s) {
                None => {
                    unresolved.insert(format!("{} cites {}", sh.id, s));
                }
                Some(src) if src.status != "current" => {
                    superseded.insert(format!("{} cites {} ({})", sh.id, s, src.status));
                }
                _ => {}
            }
        }
    }
    out.push(if unresolved.is_empty() {
        Check::pass("V8 sources resolve")
    } else {
        Check::fail(
            "V8 sources resolve",
            unresolved.into_iter().collect::<Vec<_>>().join(", "),
        )
    });
    out.push(if superseded.is_empty() {
        Check::pass("V9 no superseded sources")
    } else {
        Check::fail(
            "V9 no superseded sources",
            superseded.into_iter().collect::<Vec<_>>().join(", "),
        )
    });

    // V10 — every relation edge names groups that exist.
    let mut badrel = Vec::new();
    for r in &tree.relations {
        if !tree.groups.contains_key(&r.from) || !tree.groups.contains_key(&r.to) {
            badrel.push(format!("{} -> {}", r.from, r.to));
        }
        if r.why.trim().is_empty() {
            badrel.push(format!(
                "{} -> {} is unlabelled, which is itself a finding",
                r.from, r.to
            ));
        }
    }
    out.push(if badrel.is_empty() {
        Check::pass("V10 relations resolve and are labelled")
    } else {
        Check::fail("V10 relations resolve and are labelled", badrel.join(", "))
    });

    // V11 — every declared cycle names nodes that exist and a convergence
    //       variable inside the loop.
    let mut badcy = Vec::new();
    for c in tree.cases.values() {
        for cy in &c.cycles {
            for n in &cy.nodes {
                if !tree.sheets.contains_key(n) {
                    badcy.push(format!(
                        "case {} declares a cycle through {}, which does not exist",
                        c.id, n
                    ));
                }
            }
            if !cy.nodes.contains(&cy.converge_on) {
                badcy.push(format!(
                    "case {} converges on {}, which is not in its own cycle",
                    c.id, cy.converge_on
                ));
            }
            if cy.tolerance <= 0.0 || cy.max_iter == 0 {
                badcy.push(format!(
                    "case {} declares a cycle with no usable stopping rule",
                    c.id
                ));
            }
        }
    }
    out.push(if badcy.is_empty() {
        Check::pass("V11 declared cycles are well formed")
    } else {
        Check::fail("V11 declared cycles are well formed", badcy.join(", "))
    });

    // V12 — one crate per owner.
    //
    // Every row of a group lives in the same crate, and no crate holds rows
    // from two layers. This is the isolation rule checked rather than trusted:
    // a group whose rows are in two crates is a group that two teams have to
    // edit together, and it happens silently — a routing rule that keys on the
    // wrong field, a row added under the old convention — until somebody
    // notices a change to one layer touching three crates.
    let mut byg: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut byc: BTreeMap<&str, BTreeSet<u8>> = BTreeMap::new();
    for sh in tree.sheets.values() {
        byg.entry(sh.parent.as_str())
            .or_default()
            .insert(sh.crate_name.as_str());
        byc.entry(sh.crate_name.as_str())
            .or_default()
            .insert(sh.layer);
    }
    let mut spread: Vec<String> = byg
        .iter()
        .filter(|(_, c)| c.len() > 1)
        .map(|(g, c)| {
            format!(
                "{} is split across {}",
                g,
                c.iter().copied().collect::<Vec<_>>().join(" and ")
            )
        })
        .collect();
    spread.extend(byc.iter().filter(|(_, l)| l.len() > 1).map(|(c, l)| {
        format!(
            "{} holds layers {}",
            c,
            l.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" and ")
        )
    }));
    out.push(if spread.is_empty() {
        Check::pass("V12 one crate per owner")
    } else {
        Check::fail("V12 one crate per owner", spread.join(", "))
    });

    out
}

/// Depth-first search for a cycle that no case declares. Returns the loop it
/// found, so the message names both ends rather than saying a cycle exists.
fn find_cycles(tree: &Tree, declared: &BTreeSet<String>) -> Vec<String> {
    let ids: Vec<&String> = tree.sheets.keys().collect();
    let mut colour: std::collections::BTreeMap<&str, u8> =
        ids.iter().map(|i| (i.as_str(), 0u8)).collect();
    let mut stack: Vec<&str> = Vec::new();

    fn walk<'a>(
        node: &'a str,
        tree: &'a Tree,
        declared: &BTreeSet<String>,
        colour: &mut std::collections::BTreeMap<&'a str, u8>,
        stack: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        colour.insert(node, 1);
        stack.push(node);
        if let Some(sh) = tree.sheets.get(node) {
            for i in &sh.inputs {
                let p = match tree.sheets.get(&i.var) {
                    Some(p) => p.id.as_str(),
                    None => continue,
                };
                if declared.contains(node) && declared.contains(p) {
                    continue; // a declared loop; the resolver relaxes it
                }
                match colour.get(p).copied().unwrap_or(0) {
                    0 => {
                        if let Some(c) = walk(p, tree, declared, colour, stack) {
                            return Some(c);
                        }
                    }
                    1 => {
                        let at = stack.iter().position(|x| *x == p).unwrap_or(0);
                        let mut loop_ = stack[at..]
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>();
                        loop_.push(p.to_string());
                        return Some(loop_);
                    }
                    _ => {}
                }
            }
        }
        stack.pop();
        colour.insert(node, 2);
        None
    }

    for id in ids {
        if colour.get(id.as_str()).copied().unwrap_or(0) == 0 {
            if let Some(c) = walk(id.as_str(), tree, declared, &mut colour, &mut stack) {
                return c;
            }
        }
    }
    Vec::new()
}
