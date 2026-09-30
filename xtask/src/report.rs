//! Reading the tree: counts, what answers and what does not, where each
//! answer goes, what is promised and not covered, and the graphs.

use super::*;

pub(crate) fn cmd_status(root: &Path) -> Result<(), String> {
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

pub(crate) fn cmd_gap(root: &Path) -> Result<(), String> {
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

/// Which ring a crate sits in. Lower depends on nothing higher.
///
/// The four rings are the repository's one architectural rule, and until now
/// the only thing enforcing them was the compiler refusing a cycle — which
/// permits every wrong-direction edge that is not also circular. `vleo-core`
/// gaining a dependency on `vleo-bus` would compile, and would quietly make the
/// kernel depend on transport.
///
/// `vleo-sheet` sits beside the kernel rather than in the chain: it is what a
/// sheet MEANS, it reads only units, and both the generators and the daemon
/// read it. `vleo-data` is reference data and sits at the bus's level.
pub(crate) fn ring(crate_name: &str) -> Option<(u8, &'static str)> {
    Some(match crate_name {
        "vleo-units" => (0, "RING 0 — quantities and portable maths"),
        "vleo-core" => (1, "RING 1 — the kernel: physics and the relations"),
        "vleo-sheet" => (1, "beside the kernel — what a sheet means"),
        "vleo-bus" => (2, "RING 2 — transport"),
        "vleo-data" => (2, "reference data"),
        "vleo-modules" => (4, "the facade over every node crate"),
        "vleo-server" => (5, "the server both the daemon and the Python package start"),
        "vleo-cli" | "vleo-daemon" | "vleo-ffi" | "vleo-py" | "vleo-wasm" | "vleo-method-wasm" => {
            (6, "a face")
        }
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
pub(crate) fn crate_direction(root: &Path) -> Result<Vec<String>, String> {
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

pub(crate) fn cmd_graph(root: &Path) -> Result<(), String> {
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

pub(crate) fn deepest_chain(tree: &Tree) -> Vec<String> {
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

pub(crate) fn truncate(s: &str, n: usize) -> String {
    let one = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() <= n {
        one
    } else {
        format!("{}…", one.chars().take(n - 1).collect::<String>())
    }
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
pub(crate) enum Verdict {
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

/// The producing row behind a variable id.
///
/// A variable id is a node id or `<node id>.<extra>` — a node id never contains
/// a dot — so the producer is everything before the first one.
pub(crate) fn producer_of(var: &str) -> &str {
    var.split('.').next().unwrap_or(var)
}

/// Every row's verdict, computed once from the sheets.
pub(crate) fn active_verdicts(tree: &Tree) -> BTreeMap<&str, Verdict> {
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
pub(crate) fn reaching_kpi<'a>(
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
pub(crate) fn work_behind(
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
pub(crate) fn cmd_reach(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
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

pub(crate) fn cmd_active(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
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

#[cfg(test)]
mod reach_tests {
    use super::*;

    type Map<'a> = BTreeMap<&'a str, Vec<&'a str>>;

    /// Build `consumers` / `parents` from a list of (producer, consumer) edges.
    fn graph<'a>(edges: &[(&'a str, &'a str)]) -> (Map<'a>, Map<'a>) {
        let (mut cons, mut par): (Map, Map) = (BTreeMap::new(), BTreeMap::new());
        for (p, c) in edges {
            cons.entry(p).or_default().push(c);
            par.entry(c).or_default().push(p);
        }
        (cons, par)
    }

    fn kpis<'a>(ids: &[&'a str]) -> BTreeSet<&'a str> {
        ids.iter().copied().collect()
    }

    #[test]
    fn a_chain_that_ends_at_a_kpi_reaches_it() {
        let (_, par) = graph(&[("a", "b"), ("b", "c"), ("c", "k")]);
        let r = reaching_kpi(&par, &kpis(&["k"]));
        for n in ["a", "b", "c", "k"] {
            assert!(r.contains(n), "{n} feeds the KPI and was not counted");
        }
    }

    /// The case this whole command exists for: work that runs and goes nowhere.
    #[test]
    fn a_chain_that_ends_nowhere_does_not() {
        let (_, par) = graph(&[("a", "b"), ("b", "c"), ("x", "k")]);
        let r = reaching_kpi(&par, &kpis(&["k"]));
        for n in ["a", "b", "c"] {
            assert!(!r.contains(n), "{n} reaches nothing and was counted");
        }
        assert!(r.contains("x"), "the control case");
    }

    /// A KPI is its own witness; otherwise every KPI reports as unreached.
    #[test]
    fn a_kpi_reaches_itself() {
        let (_, par) = graph(&[]);
        assert!(reaching_kpi(&par, &kpis(&["k"])).contains("k"));
    }

    /// A declared cycle must terminate and must not hide a real path.
    ///
    /// This is the case that killed the first implementation. It asked each row
    /// "can you reach a KPI", seeding `false` before recursing so the loop
    /// terminated — and then memoised that provisional `false` for `a`, whose
    /// real answer arrived later through `b`. `a` reported unread work that was
    /// being read. Walking backwards from the KPIs has no such ordering.
    #[test]
    fn a_cycle_terminates_and_still_finds_the_path() {
        let (_, par) = graph(&[("a", "b"), ("b", "a"), ("b", "k")]);
        let r = reaching_kpi(&par, &kpis(&["k"]));
        assert!(r.contains("a"), "a reaches k through b");
        assert!(r.contains("b"));

        let (_, par2) = graph(&[("a", "b"), ("b", "a")]);
        let r2 = reaching_kpi(&par2, &kpis(&["k"]));
        assert!(
            !r2.contains("a"),
            "a closed loop reaching nothing is not reached"
        );
    }

    /// The same cycle with the KPI edge on the OTHER side of it.
    ///
    /// Both orientations are here because the forward-memo version this
    /// replaced fails on exactly one of them, and which one depends on the
    /// order rows happen to be visited in. A single orientation passes against
    /// the broken implementation about half the time, which is the same as not
    /// testing it: the first draft of this file had only the other one, and the
    /// re-introduced bug went straight through it.
    #[test]
    fn the_cycle_holds_whichever_side_the_kpi_edge_is_on() {
        let (_, par) = graph(&[("a", "b"), ("b", "a"), ("a", "k")]);
        let r = reaching_kpi(&par, &kpis(&["k"]));
        assert!(r.contains("a"), "a feeds k directly");
        assert!(r.contains("b"), "b reaches k through a, around the cycle");
    }

    #[test]
    fn work_behind_counts_the_upstream_closure_once() {
        // a diamond: d reads b and c, both read a.
        let (_, par) = graph(&[("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")]);
        let all = |_: &str| true;
        assert_eq!(work_behind("d", &par, &all), 3, "a counted once, not twice");
        assert_eq!(
            work_behind("a", &par, &all),
            0,
            "nothing is behind the root"
        );
    }

    /// Only answering rows count. A terminal row with fifty blocked rows behind
    /// it is not fifty rows of wasted work — it is fifty rows of nothing.
    #[test]
    fn work_behind_counts_only_rows_that_answer() {
        let (_, par) = graph(&[("a", "b"), ("b", "c")]);
        let silent = |id: &str| id != "a";
        assert_eq!(work_behind("c", &par, &silent), 1);
    }

    #[test]
    fn work_behind_terminates_on_a_cycle() {
        let (_, par) = graph(&[("a", "b"), ("b", "a")]);
        let all = |_: &str| true;
        assert_eq!(work_behind("a", &par, &all), 1);
    }
}
