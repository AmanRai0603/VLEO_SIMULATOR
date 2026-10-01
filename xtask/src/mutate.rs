//! Mutation testing: whether a node's tests would notice it being wrong.

use super::*;

/// How far the mutant's answer moves, for a node with no fixture to size it.
pub(super) const MUTANT_FLOOR: f64 = 1e-3;

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
pub(super) fn mutant_scale(sh: &vleo_sheet::model::Sheet) -> f64 {
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
pub(super) fn cmd_mutate(root: &Path, args: &[&str]) -> Result<(), String> {
    if args.contains(&"--literals") {
        let rest: Vec<&str> = args
            .iter()
            .copied()
            .filter(|a| *a != "--literals")
            .collect();
        return cmd_mutate_literals(root, &rest);
    }
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

/// Mutation testing, one decimal number at a time.
///
/// Scaling the answer asks whether the evidence notices the node being wrong
/// as a whole. It cannot ask whether it notices one coefficient being wrong: a
/// fixture taken at the one point where a term vanishes passes whatever that
/// term's coefficient is. So each decimal literal inside a HOLE body is moved,
/// in turn, by more than twice the loosest tolerance, and the node's tests are
/// run against it. A literal no test objects to is reported by line.
///
/// A report, not a gate: most rows have no fixture yet, and every survivor on
/// them is already a counted gap. Read it for the rows that do.
pub(super) fn cmd_mutate_literals(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
    let only = args.first().copied();
    let mut targets: Vec<&vleo_sheet::model::Sheet> = tree
        .ordered()
        .into_iter()
        .filter(|sh| sh.state == "published" && !sh.is_declared() && !sh.fixtures.is_empty())
        .filter(|sh| only.is_none_or(|o| sh.id == o))
        .collect();
    targets.sort_by(|a, b| a.id.cmp(&b.id));
    let (mut killed, mut survived) = (0usize, Vec::new());
    for sh in &targets {
        let path = sh.dir.join("model.rs");
        let original = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        for (line, mutant) in literal_mutants(&original, mutant_scale(sh)) {
            fs::write(&path, &mutant).map_err(|e| format!("{}: {e}", path.display()))?;
            let out = std::process::Command::new("cargo")
                .current_dir(root)
                .args(["test", "-q", "-p", &sh.crate_name, "--", &sh.id])
                .output();
            fs::write(&path, &original).map_err(|e| format!("{}: {e}", path.display()))?;
            let out = out.map_err(|e| format!("running cargo test: {e}"))?;
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            if !text.contains("test result:") {
                continue; // did not build: not a detection, and not a survivor
            }
            if out.status.success() {
                survived.push(format!("{} model.rs:{line}", sh.id));
            } else {
                killed += 1;
            }
        }
    }
    println!(
        "mutate --literals: {} node(s) with fixtures, {killed} literal mutant(s) caught, {} not",
        targets.len(),
        survived.len()
    );
    for s in &survived {
        println!("  no test noticed the number at {s} changing");
    }
    Ok(())
}

/// Every decimal literal inside a HOLE body, each moved by `scale` in its own
/// copy of `src`, with the line it is on.
pub(super) fn literal_mutants(src: &str, scale: f64) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut inside = false;
    let mut offset = 0usize;
    for (n, line) in src.split_inclusive('\n').enumerate() {
        let t = line.trim_start();
        if t.starts_with("// ---- HOLE ") {
            inside = true;
        } else if t.starts_with("// ---- end HOLE ") {
            inside = false;
        } else if inside {
            // The code part of the line: before a comment, outside strings.
            let code = vleo_sheet::gate::code_only(line);
            let cb: Vec<char> = code.chars().collect();
            let lb: Vec<char> = line.chars().collect();
            let mut i = 0;
            while i < cb.len() {
                let starts = cb[i].is_ascii_digit()
                    && (i == 0
                        || !(cb[i - 1].is_alphanumeric() || cb[i - 1] == '_' || cb[i - 1] == '.'));
                if !starts {
                    i += 1;
                    continue;
                }
                let mut j = i;
                while j < cb.len() && (cb[j].is_ascii_digit() || cb[j] == '_') {
                    j += 1;
                }
                let mut float = false;
                if j + 1 < cb.len() && cb[j] == '.' && cb[j + 1].is_ascii_digit() {
                    float = true;
                    j += 1;
                    while j < cb.len() && (cb[j].is_ascii_digit() || cb[j] == '_') {
                        j += 1;
                    }
                }
                if j < cb.len() && (cb[j] == 'e' || cb[j] == 'E') {
                    let mut k = j + 1;
                    if k < cb.len() && (cb[k] == '+' || cb[k] == '-') {
                        k += 1;
                    }
                    if k < cb.len() && cb[k].is_ascii_digit() {
                        float = true;
                        j = k;
                        while j < cb.len() && cb[j].is_ascii_digit() {
                            j += 1;
                        }
                    }
                }
                let text: String = cb[i..j].iter().filter(|c| **c != '_').collect();
                // Comments and strings were blanked, not removed, only when
                // they kept their length; check the literal is really there.
                let same = lb.len() >= j
                    && lb[i..j].iter().collect::<String>() == cb[i..j].iter().collect::<String>();
                if float && same {
                    if let Ok(v) = text.parse::<f64>() {
                        if v != 0.0 {
                            let byte_i: usize = lb[..i].iter().map(|c| c.len_utf8()).sum();
                            let byte_j: usize = lb[..j].iter().map(|c| c.len_utf8()).sum();
                            let mut m = String::with_capacity(src.len() + 8);
                            m.push_str(&src[..offset + byte_i]);
                            m.push_str(&format!("{:?}", v * scale));
                            m.push_str(&src[offset + byte_j..]);
                            out.push((n + 1, m));
                        }
                    }
                }
                i = j;
            }
        }
        offset += line.len();
    }
    out
}

/// Scale the one line every generated model.rs ends with.
///
/// Returns `None` when there is nothing to perturb, which is a real state and
/// not a failure: a node whose scaffold has never been generated has no answer
/// line yet.
pub(super) fn mutate_answer(src: &str, scale: f64) -> Option<String> {
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
