//! Writing a node by hand: a new one shaped like a sibling, its completion
//! questions, a hole body spliced in, and the checks that its tests test.

use super::*;

/// One recorded fill: a hole, who wrote its body, on what model, and the body.
///
/// The body is kept rather than hashed because the point of keeping it is to
/// run it again. A hash would prove two bodies differed and leave the
/// comparison — the thing the rule is actually asking for — impossible.
pub(crate) struct Fill {
    pub(crate) hole: u32,
    pub(crate) by: String,
    pub(crate) model: String,
    pub(crate) body: String,
}

pub(crate) fn fills_path(dir: &Path) -> PathBuf {
    dir.join("fills.toml")
}

pub(crate) fn read_fills(dir: &Path) -> Vec<Fill> {
    let Ok(text) = fs::read_to_string(fills_path(dir)) else {
        return Vec::new();
    };
    let Ok(v) = text.parse::<toml::Value>() else {
        return Vec::new();
    };
    v.get("fill")
        .and_then(toml::Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|f| {
                    Some(Fill {
                        hole: f.get("hole")?.as_integer()? as u32,
                        by: f.get("by")?.as_str()?.to_string(),
                        model: f.get("model")?.as_str()?.to_string(),
                        body: f.get("body")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Append a fill record.
///
/// The model-family rule, enforced as a fact about the file rather than as a
/// sentence in a prompt: a second body for the same hole from the same model is
/// refused. A model handed its own reasoning to check approves it, so two
/// bodies from one model are one body written twice.
pub(crate) fn check_fill_attribution(
    sh: &vleo_sheet::model::Sheet,
    hole: u32,
    model: &str,
    body: &str,
) -> Result<(), String> {
    for f in read_fills(&sh.dir) {
        if f.hole == hole && f.body.trim() == body.trim() {
            return Ok(());
        }
        if f.hole == hole && f.model == model {
            return Err(format!(
                "hole {hole} already has a body from {} on {model}, and this one is also on \
                 {model}. Two bodies from one model are one body written twice: a model given \
                 its own reasoning to check approves it. A second body has to come from a \
                 different model. Nothing was written.",
                f.by
            ));
        }
    }
    Ok(())
}

pub(crate) fn record_fill(
    sh: &vleo_sheet::model::Sheet,
    hole: u32,
    who: &str,
    model: &str,
    body: &str,
) -> Result<(), String> {
    let existing = read_fills(&sh.dir);
    if existing
        .iter()
        .any(|f| f.hole == hole && f.body.trim() == body.trim())
    {
        return Ok(());
    }
    let mut out = String::new();
    if existing.is_empty() {
        out.push_str(
            "# Every body written for this node's holes, and what wrote each.\n\
             #\n\
             # A significant node is filled twice by different models and the two\n\
             # compared over the declared domain. That comparison needs both bodies,\n\
             # so both are kept here rather than hashed. `cargo xtask differential\n\
             # <node>` runs it.\n\
             #\n\
             # Written by `cargo xtask fill --by --model`. Editing it by hand defeats the\n\
             # one thing it is for.\n",
        );
    } else {
        out.push_str(&fs::read_to_string(fills_path(&sh.dir)).unwrap_or_default());
    }
    out.push_str(&format!(
        "\n[[fill]]\nhole = {hole}\nby = {who:?}\nmodel = {model:?}\nbody = \"\"\"\n{}\"\"\"\n",
        if body.ends_with('\n') {
            body.to_string()
        } else {
            format!("{body}\n")
        }
    ));
    fs::write(fills_path(&sh.dir), out).map_err(|e| format!("fills.toml: {e}"))?;
    println!("  recorded: hole {hole} by {who} on {model}");
    Ok(())
}

/// Differential fill: run the node against every other body recorded for the
/// same hole.
///
/// The working model asks for the same hole filled independently by a second
/// model family and the two compared numerically over the declared domain. The
/// comparison is the node's own evidence — its fixtures, its generated
/// properties and its parity grid if it has one — because those are exactly the
/// numeric checks over the declared domain, and running a second, private grid
/// beside them would be a second definition of correct.
///
/// What this cannot do is produce the second body. Whoever implements a node —
/// a developer, or an assistant a developer runs — names the model with
/// `--model`, and the rule is only as strong as the models on offer: two tiers
/// of one vendor's models differ in size, not in training.
pub(crate) fn cmd_differential(root: &Path, args: &[&str]) -> Result<(), String> {
    let id = args
        .first()
        .ok_or("usage: cargo xtask differential <node>")?;
    let tree = load(root)?;
    let sh = tree
        .sheets
        .get(*id)
        .ok_or_else(|| format!("no node '{id}'"))?;
    if sh.is_declared() {
        return Err(format!("'{id}' is a declared value — it has no holes"));
    }

    let fills = read_fills(&sh.dir);
    if fills.is_empty() {
        return Err(format!(
            "no fills recorded for '{id}'. `cargo xtask fill --by <who> --model <model>` records one; \
             without a record there is nothing to compare and saying so is the only \
             honest answer"
        ));
    }

    let current = vleo_sheet::load::read_holes(&sh.dir);
    let path = sh.dir.join("model.rs");
    let original = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut compared = 0usize;
    let mut disagreed: Vec<String> = Vec::new();

    // Every recorded body, including whichever one is currently in the file.
    // Running only the alternatives reports "agrees" when the body that shipped
    // is the wrong one and the alternative is right — which is the disagreement
    // stated backwards, and the one case where getting this wrong is expensive.
    for f in &fills {
        let mut holes = current.clone();
        holes.insert(f.hole, f.body.clone());
        let text = gate::formatted(&emit::model_rs(sh, &holes));
        fs::write(&path, &text).map_err(|e| format!("{}: {e}", path.display()))?;
        let out = std::process::Command::new("cargo")
            .current_dir(root)
            .args(["test", "-q", "-p", &sh.crate_name, "--", &sh.id])
            .output();
        fs::write(&path, &original).map_err(|e| format!("{}: {e}", path.display()))?;
        let out = out.map_err(|e| format!("running cargo test: {e}"))?;
        compared += 1;
        if out.status.success() {
            println!(
                "  \x1b[32magrees\x1b[0m   hole {} — {} on {}",
                f.hole, f.by, f.model
            );
        } else {
            println!(
                "  \x1b[31mDISAGREES\x1b[0m hole {} — {} on {}",
                f.hole, f.by, f.model
            );
            disagreed.push(format!("hole {} ({} on {})", f.hole, f.by, f.model));
        }
    }

    // Two records that are the same body are one body recorded twice, whatever
    // two names sit against it.
    let distinct: std::collections::BTreeSet<&str> = fills.iter().map(|f| f.body.trim()).collect();
    if distinct.len() < 2 {
        return Err(format!(
            "'{id}' has {} recorded fill(s) and {} distinct body/bodies. A hole filled once \
             is not a differential fill, whatever the record says",
            fills.len(),
            distinct.len()
        ));
    }
    println!("differential: {compared} recorded body/bodies run against this node's evidence");
    if disagreed.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} recorded body/bodies disagree with this node's evidence: {}. Two \
             independent readings of the same sheet produced different answers, so at \
             least one of them read it wrong — or the sheet says less than the author \
             thought. This is a finding for the node owner, not something to resolve by \
             picking the body that passes.",
            disagreed.len(),
            disagreed.join(", ")
        ))
    }
}

/// How far the mutant's answer moves, for a node with no fixture to size it.
pub(crate) const MUTANT_FLOOR: f64 = 1e-3;

/// How far to move this node's answer.
///
/// Sized by the node's own loosest fixture tolerance rather than fixed. A fixed
/// perturbation asks an arbitrary question — a first run at a tenth of a percent
/// reported five nodes as unevidenced whose fixtures declare half a percent,
/// which is not a finding about those nodes but about the number chosen here.
///
/// Twice the loosest tolerance asks the question the node itself poses: the
/// evidence claims the answer is known to within so much, so an error larger
/// than that must be caught. If it is not, the tolerance is wider than anything
/// actually checks.
pub(crate) fn mutant_scale(sh: &vleo_sheet::model::Sheet) -> f64 {
    let loosest = sh
        .fixtures
        .iter()
        .map(|f| f.tolerance)
        .filter(|t| t.is_finite() && *t > 0.0)
        .fold(0.0f64, f64::max);
    1.0 + 2.0 * loosest.max(MUTANT_FLOOR)
}

/// Mutation testing, by the one mutation that applies to every node.
///
/// The gate proves the tests pass. It cannot prove they would fail, and those
/// are different claims: a fixture whose tolerance is wide enough to swallow
/// the relation being wrong passes for the whole life of the node and is read
/// by everyone as evidence. The only way to tell the two apart is to break the
/// implementation on purpose and check that something objects.
///
/// One mutation rather than a catalogue of them, and it is deliberately not a
/// clever one: every generated model.rs ends in `let answer: T = ...`, so
/// scaling that is type-correct everywhere and asks the question that matters —
/// if this node answered a tenth of a percent differently, would anything
/// notice? A node that says no has evidence that is decoration.
///
/// The file is restored from the bytes read before the edit. If a run is killed
/// between the two, `cargo xtask docs <node>` regenerates the file and the
/// mutation is outside every hole, so it does not survive.
pub(crate) fn cmd_mutate(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
    let only = args.first().copied();
    let mut targets: Vec<&vleo_sheet::model::Sheet> = tree
        .ordered()
        .into_iter()
        .filter(|sh| sh.state == "published")
        // A declared value has no relation to get wrong: the number is the
        // source, not a derivation from one. Perturbing it asks whether some
        // other node pins it, which is a different question and one the
        // contribution graph already answers. Two thirds of the tree is
        // declared values, so including them would bury the signal.
        .filter(|sh| !sh.is_declared())
        .filter(|sh| only.is_none_or(|o| sh.id == o))
        .collect();
    targets.sort_by(|a, b| a.id.cmp(&b.id));

    if targets.is_empty() {
        // A named row that cannot be mutated is not an error. The per-change
        // pipeline hands this command every node folder a change touched, and
        // most of them are legitimately not mutation targets — so erroring here
        // failed a job for a correct state, and said "is not a written node"
        // about a row that was written and simply had no relation in it.
        if let Some(o) = only {
            // A name that is not a row is still a mistake — a typo must not
            // pass quietly just because the surrounding cases are benign.
            let Some(named) = tree.sheets.get(o) else {
                return Err(format!("{o} is not a row in this tree"));
            };
            let why = match Some(named) {
                None => unreachable!(),
                Some(sh) if sh.is_seeded() => format!(
                    "{o} is seeded — nothing is generated from it yet, so there is nothing to \
                     perturb"
                ),
                Some(sh) if sh.is_declared() => format!(
                    "{o} is a declared value — the number is the source, not a derivation from \
                     one, so there is no relation to get wrong"
                ),
                Some(_) => format!("{o} has no answer to perturb"),
            };
            println!("{why}");
            return Ok(());
        }
        return Err("no written nodes".into());
    }

    let mut survived: Vec<String> = Vec::new();
    let mut unevidenced: Vec<String> = Vec::new();
    let mut unmutatable: Vec<String> = Vec::new();
    let mut killed = 0usize;

    for sh in &targets {
        let path = sh.dir.join("model.rs");
        let original = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;

        let Some(mutant) = mutate_answer(&original, mutant_scale(sh)) else {
            unmutatable.push(sh.id.clone());
            continue;
        };

        fs::write(&path, &mutant).map_err(|e| format!("{}: {e}", path.display()))?;
        let out = std::process::Command::new("cargo")
            .current_dir(root)
            .args(["test", "-q", "-p", &sh.crate_name, "--", &sh.id])
            .output();
        // Restore before anything else can fail. A mutant left on disk is a
        // wrong implementation committed by whoever runs git add next.
        fs::write(&path, &original).map_err(|e| format!("{}: {e}", path.display()))?;

        let out = out.map_err(|e| format!("running cargo test: {e}"))?;
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );

        // A non-zero exit is not a detection on its own. A mutant that does not
        // compile exits non-zero too, and counting that as a kill reports the
        // strongest possible evidence for a node nothing has ever checked.
        if !text.contains("test result:") {
            unmutatable.push(format!("{} — the mutant did not build", sh.id));
            continue;
        }
        // Zero tests is a passing run said differently.
        if out.status.success() && text.contains("0 passed; 0 failed") {
            unevidenced.push(sh.id.clone());
            continue;
        }
        if out.status.success() {
            // Two different results wearing one word. A node with a fixture
            // that survives is a finding: something claims to check this and
            // does not. A node with no fixture that survives is a gap already
            // counted against that row, and 102 rows of restatement is how a
            // check stops being read.
            //
            // The split is not obvious from the outside — eighteen rows with no
            // fixture were killed anyway, by a guard or by a test elsewhere in
            // their crate — so it is measured here rather than assumed.
            if sh.fixtures.is_empty() {
                unevidenced.push(sh.id.clone());
            } else {
                println!(
                    "  \x1b[31mSURVIVED\x1b[0m {} (moved {:.3}%)",
                    sh.id,
                    (mutant_scale(sh) - 1.0) * 100.0
                );
                survived.push(sh.id.clone());
            }
        } else {
            println!("  \x1b[32mkilled\x1b[0m   {}", sh.id);
            killed += 1;
        }
    }

    println!(
        "mutate: {} of {} mutant(s) killed{}",
        killed,
        killed + survived.len() + unevidenced.len(),
        if unevidenced.is_empty() {
            String::new()
        } else {
            format!(
                " — {} of the survivor(s) have no fixture at all, which is a gap already \
                 counted against those rows rather than a finding here",
                unevidenced.len()
            )
        }
    );
    for id in &unmutatable {
        println!("  no answer to perturb: {id}");
    }
    if !survived.is_empty() {
        return Err(format!(
            "{} node(s) answered by more than twice their own declared tolerance and every \
             test still passed: {}. The evidence on these rows is decoration — a tolerance \
             wider than anything that actually checks it, or no fixture reaching this answer \
             at all. Add the case that pins it, or tighten the tolerance to what the source \
             supports. Do not weaken this check.",
            survived.len(),
            survived.join(", ")
        ));
    }
    Ok(())
}

/// Scale the one line every generated model.rs ends with.
///
/// Returns `None` when there is nothing to perturb, which is a real state and
/// not a failure: a node whose scaffold has never been generated has no answer
/// line yet.
pub(crate) fn mutate_answer(src: &str, scale: f64) -> Option<String> {
    let at = src.find("    let answer: ")?;
    let rest = &src[at..];
    let semi = rest.find(";\n")?;
    let line = &rest[..semi];
    let (decl, expr) = line.split_once(" = ")?;
    let ty = decl.trim_start().strip_prefix("let answer: ")?;
    Some(format!(
        "{}{decl} = {ty}::new(({expr}).get() * {scale:?}){}",
        &src[..at],
        &rest[semi..]
    ))
}

// ---------------------------------------------------------------------------

pub(crate) fn cmd_declare(root: &Path, args: &[&str]) -> Result<(), String> {
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
pub(crate) fn cmd_ready(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
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
pub(crate) fn cmd_fill(root: &Path, args: &[&str]) -> Result<(), String> {
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
    if vleo_sheet::method::node_program(sh).is_some() {
        return Err(format!(
            "'{id}' is built from its method: its code is the method translated by rule, and \
             it has no holes. Change the method, on the node's form"
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
    if let Some(bad) = gate::platform_maths(&body).first() {
        return Err(format!(
            "refused: the body calls {bad} — route it through pmath, or cross-face agreement \
             fails on the first night for a reason that is not a defect"
        ));
    }

    // Everything that can refuse, refuses before the file is touched. A splice
    // that lands and then reports "nothing was written" is worse than either
    // outcome on its own.
    let attribution = args
        .iter()
        .position(|a| *a == "--by")
        .and_then(|i| args.get(i + 1))
        .copied();
    let model = args
        .iter()
        .position(|a| *a == "--model")
        .and_then(|i| args.get(i + 1))
        .copied();
    let attribution = match (attribution, model) {
        (Some(who), Some(model)) => {
            check_fill_attribution(sh, n, model, &body)?;
            Some((who, model))
        }
        (Some(who), None) => {
            return Err(format!(
                "a body by {who} needs the model that wrote it: --model <name>. Two bodies from \
                 one model are one body written twice, so the model is what the comparison \
                 turns on. Nothing was written."
            ))
        }
        (None, _) if sh.criticality == "significant" => {
            return Err(format!(
                "'{id}' is significant, so its holes are filled twice by different models and \
                 the two compared. An unattributed body cannot be compared to anything: pass \
                 --by <who> --model <model>. Nothing was written."
            ))
        }
        (None, _) => None,
    };

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
    if let Some((who, model)) = attribution {
        record_fill(sh, n, who, model, &body)?;
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

/// Clone a sibling's sheet, blanking every field that must be re-decided.
///
/// Pure, and separated from `cmd_new` so it can be tested: each rule below
/// exists because a clone carried something it should not have, and a rule with
/// no test is a rule that comes back. Returns the sheet and the comment lines it
/// could not blank, which are prose about the sibling and have no marker saying
/// which sentences are row-specific.
pub(crate) fn clone_sheet(
    sheet: &str,
    id: &str,
    folder: &str,
    src_order: u32,
) -> (String, Vec<String>) {
    // Blank what must be re-decided. A literal copy drags a stale source
    // citation and someone else's domain limits through thirty nodes.
    let mut out = String::new();
    // A blanked field whose value is a multi-line string leaves its BODY behind,
    // and the body is not TOML on its own. `text = "\"\"\"` became
    // `text = ""` and the twenty prose lines under it were still there,
    // starting with a bare word where a key was expected, so the sheet the tool
    // had just written could not be parsed by the tool's own next command. It
    // happened twice before this skipped the body.
    let mut in_blanked_block = false;
    // Which [section] the line belongs to. `number` is a value under [value] and
    // a step index under [[algorithm.step]]; blanking both would renumber the
    // algorithm, so the key alone is not enough to decide.
    let mut section = String::new();
    let mut carried: Vec<String> = Vec::new();
    // THE SIBLING'S RECORD IS NOT INHERITED. Its versions say why the SIBLING
    // changed, its risks are registered once, on it, and its plain-words
    // explanation is about its own relation — so each is dropped whole, not
    // blanked: a new row starts with no history, and says its first belief on
    // its own form. Nor is what it CONTRIBUTES to: a KPI the sibling feeds is
    // a contract edge of the sibling's, and a new row that inherited it claimed
    // to move a KPI nobody had asked it to.
    let mut dropped = false;
    for line in sheet.lines() {
        let l = line.trim_start();
        if l.starts_with('[') {
            section = l.to_string();
            dropped = matches!(
                l,
                "[[version]]" | "[[risk]]" | "[explain]" | "[contributes]"
            );
        }
        if dropped {
            continue;
        }
        // A comment block is prose about the SIBLING, and there is no way to tell
        // its row-specific sentences from the template's generic ones. So it is
        // carried and reported rather than carried silently: the sibling's header
        // explained a G scale on a row that had nothing to do with one.
        if l.starts_with("# ") && l.len() > 40 && !carried.iter().any(|c| c == l) {
            carried.push(l.to_string());
        }
        if in_blanked_block {
            if l == "\"\"\"" || l.ends_with("\"\"\"") {
                in_blanked_block = false;
            }
            continue;
        }
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
            // `migrated_from` is a source citation, and this loop exists
            // because "a literal copy drags a stale source citation through
            // thirty nodes". It was not in the list, so a clone pointed at the
            // sibling's MATLAB function and at a parity grid that was not its
            // own.
            || l.starts_with("migrated_from = ")
            // A `fails_when` is the other half of an `[[assumption]]` whose
            // `text` is blanked above, so inheriting it leaves the sheet stating
            // how a claim it no longer makes would fail. The clone carried three
            // of them about the NOAA G scale onto a row about Kp slots.
            || l.starts_with("fails_when = ")
            // `why` and `reading` are the theory tab: a derivation of the
            // SIBLING's relation, beside a blanked `expression`. §31.2a is what
            // an inherited magnitude in a theory tab costs.
            || l.starts_with("why = ")
            || l.starts_with("reading = ")
            // The symbol is the row's own name for its own answer. Two rows
            // sharing one is the defect the `no-identity` check looks for.
            || l.starts_with("symbol = ")
            // A value under [value] is a number a person picked for another row.
            // Under [[algorithm.step]] the same key is a step index, which is
            // shape and is inherited.
            || (l.starts_with("number = ") && section == "[value]")
        {
            let key = l.split(" = ").next().unwrap();
            // A blanked NUMBER is 0.0 and not "": the sheet has to stay TOML the
            // tool's own next command can read, and `number = ""` is a string
            // where the loader wants a float. The gate still refuses it, on
            // `declared-value`, which is the check that is actually true.
            let blank = if key == "number" { "0.0" } else { "\"\"" };
            out.push_str(&format!(
                "{key} = {blank}   # REQUIRED — re-decide, do not inherit\n"
            ));
            // Opened a \"\"\" block and did not close it on the same line: the
            // rest belongs to the value that was just blanked.
            let after = l.split_once(" = ").map(|x| x.1).unwrap_or("");
            if after.starts_with("\"\"\"") && !after[3..].contains("\"\"\"") {
                in_blanked_block = true;
            }
        } else if l.starts_with("state = ") {
            // A NEW ROW IS SEEDED, WHATEVER THE SIBLING IS. Inheriting
            // `published` gave a folder with every field blank a state that
            // means "specified": it counted as published in `xtask status`, in
            // the index the face reads and in /v1/branches' idea of an active
            // branch, and the gate then refused it for four separate reasons at
            // once instead of the one that is true — that nobody has written it
            // yet.
            out.push_str("state = \"empty\"\n");
        } else if l.starts_with("order = ") {
            // THE SIBLING'S PLACE IS TAKEN. `order` is globally contiguous and
            // one row per place is an assembly check, so copying the sibling's
            // number guarantees a collision — the tool wrote a tree its own
            // gate refused, every time, and the person then renumbered 32 rows
            // by hand. The new row goes immediately after the sibling and
            // everything at or beyond that place moves up one, below.
            out.push_str(&format!("order = {}\n", src_order + 1));
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
    (out, carried)
}

pub(crate) fn cmd_new(root: &Path, args: &[&str]) -> Result<(), String> {
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
    let src_order = src.order;
    let (out, carried) = clone_sheet(&sheet, id, &folder, src_order);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join("node.toml"), out).map_err(|e| e.to_string())?;
    if !carried.is_empty() {
        println!(
            "{} comment block line(s) were carried from {like} and may be about that row \
             rather than this one. Read them before `docs`:",
            carried.len()
        );
        for c in carried.iter().take(4) {
            println!("   {}", &c[..c.len().min(96)]);
        }
        if carried.len() > 4 {
            println!("   ... and {} more", carried.len() - 4);
        }
    }
    // Make room. Every sheet already at or beyond the new row's place moves up
    // one, so the tree stays contiguous and the assembly check stays quiet.
    // `order` is outside the sheet hash, so this rewrites no generated artefact.
    let mut shifted = 0usize;
    for sh in tree.ordered() {
        if sh.id.as_str() == *id || sh.order <= src_order {
            continue;
        }
        set_order(&sh.dir.join("node.toml"), sh.order + 1)?;
        shifted += 1;
    }
    if shifted > 0 {
        println!(
            "made room at {}: {} row(s) moved up one",
            src_order + 1,
            shifted
        );
    }
    fs::write(
        dir.join("fixtures.toml"),
        "# Known-good values, and where each came from. An expected value may\n# never be produced by the code under test.\n",
    )
    .map_err(|e| e.to_string())?;
    println!("new: {}", dir.display());
    println!("`cargo xtask declare {id}` lists what must be answered first.");
    println!("Then `cargo xtask docs {id}`.");
    println!("Generation refuses while any field is open, and names the fields. That refusal is the mechanism: ambiguity becomes a blocking item on an engineer's screen rather than something an implementer resolves silently.");
    println!("When it generates, `cargo xtask ready {id}` says whether a person should be asked to look yet.");
    Ok(())
}
