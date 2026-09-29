//! Hole fills: splicing a body in, recording who wrote it, and comparing two fills.

use super::*;

/// Today, as the sheets write it.
pub(super) fn today() -> String {
    // The sheets carry a plain ISO date and nothing reads it as a timestamp, so
    // a date is all this needs. It once came from the `date` program, which
    // Windows does not have.
    vleo_units::calendar::Civil::from_unix(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    )
    .date()
    .to_string()
}

/// The relations with nobody's name against them, and who owes each one.
///
/// Two thirds of the tree is declared values and every one of those already
/// carries a confirmation. What is missing is the other third: the relations,
/// where the name matters most, because a declared value that nobody confirmed
/// is a number somebody has to defend and a relation that nobody confirmed
/// might not have come from a person at all.
pub(super) fn cmd_confirm(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;

    if args.first() == Some(&"--list") {
        let only = args.get(1).copied();
        let mut by_owner: BTreeMap<&str, Vec<&vleo_sheet::model::Sheet>> = BTreeMap::new();
        for sh in tree.ordered() {
            if sh.state != "published" || sh.is_declared() {
                continue;
            }
            if !sh.relation_by.trim().is_empty() {
                continue;
            }
            if only.is_some_and(|o| sh.subsystem != o) {
                continue;
            }
            by_owner.entry(sh.owner.as_str()).or_default().push(sh);
        }
        let total: usize = by_owner.values().map(Vec::len).sum();
        if total == 0 {
            println!("every written relation has a name against it");
            return Ok(());
        }
        for (owner, rows) in &by_owner {
            println!("\n\x1b[1m{owner}\x1b[0m — {} relation(s)", rows.len());
            for sh in rows {
                println!("  {:<32} {}", sh.id, sh.expression);
                println!("  {:<32} source: {}", "", sh.source);
            }
        }
        println!(
            "\n{total} relation(s) with nobody's name against them, across {} owner(s).",
            by_owner.len()
        );
        println!(
            "Each needs: cargo xtask confirm <node> --by \"<your name>\"\n\
             A name here is a person saying the relation is right and is theirs. It is not\n\
             a formatting step, and nothing else in the tool can supply it."
        );
        return Ok(());
    }

    let id = args
        .first()
        .ok_or("usage: cargo xtask confirm <node> --by \"<name>\"  |  confirm --list")?;
    let who = args
        .iter()
        .position(|a| *a == "--by")
        .and_then(|i| args.get(i + 1))
        .ok_or("who is confirming it: --by \"<name>\"")?
        .trim();

    let sh = tree
        .sheets
        .get(*id)
        .ok_or_else(|| format!("no node '{id}'"))?;
    if sh.is_declared() {
        return Err(format!(
            "'{id}' is a declared value. Its confirmation belongs under [value], not [maths], \
             and it already has one: {}",
            sh.confirmed_by
        ));
    }
    if sh.expression.trim().is_empty() {
        return Err(format!("'{id}' has no relation to confirm"));
    }
    if who.is_empty() {
        return Err("--by is empty".into());
    }

    // An assistant may never supply mathematics. Stated as a sentence it is a hope;
    // this makes it a fact about what can be written to the file.
    let lower = who.to_lowercase();
    for bad in vleo_sheet::form::agent_identities(root) {
        if lower == bad
            || lower.starts_with(&format!("{bad} "))
            || lower.contains(&format!("{bad}/"))
        {
            return Err(format!(
                "refused: '{who}' is an assistant's name. An assistant may never supply mathematics, and this \
                 field is the only thing that can tell whether one did. It takes the name of a \
                 person who has read the relation against its source and is prepared to own it. \
                 Nothing was written."
            ));
        }
    }
    if !sh.relation_by.trim().is_empty() {
        return Err(format!(
            "'{id}' is already confirmed by {}. Changing an attribution is a review decision, \
             not a command — edit the sheet and say why in the commit.",
            sh.relation_by
        ));
    }

    // What is being confirmed, in front of the person at the moment they
    // confirm it. A name put against a relation nobody re-read is a keystroke,
    // not a confirmation.
    println!("\n\x1b[1m{}\x1b[0m — {}", sh.id, sh.label);
    println!("  asks       {}", sh.question);
    println!("  relation   {}", sh.expression);
    println!("  source     {}", sh.source);
    if !sh.assumptions.is_empty() {
        for a in &sh.assumptions {
            println!("  assumes    {} — fails when {}", a.text, a.fails_when);
        }
    }
    println!(
        "  answers    {} in {} over {} … {}",
        sh.symbol, sh.unit, sh.lower, sh.upper
    );

    let stamp = format!("{who} / {}", today());
    let path = sh.dir.join("node.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;

    // Textual insertion rather than a TOML round-trip: these sheets carry more
    // comment than content, and every one of those comments is somebody's
    // reason. A serialiser would silently drop the lot.
    let at = text
        .find("\n[maths]\n")
        .map(|i| i + "\n[maths]\n".len())
        .ok_or_else(|| format!("{}: no [maths] section", path.display()))?;
    let line =
        format!("confirmed_by = {stamp:?}   # a person read this relation against its source\n");
    let out = format!("{}{}{}", &text[..at], line, &text[at..]);
    fs::write(&path, &out).map_err(|e| format!("{}: {e}", path.display()))?;

    // Read it back through the loader, not the string we just wrote. Writing a
    // file and announcing success is how a malformed sheet gets found three
    // nodes later.
    let after = load(root)?;
    let got = after
        .sheets
        .get(*id)
        .map(|s| s.relation_by.clone())
        .unwrap_or_default();
    if got != stamp {
        fs::write(&path, &text).map_err(|e| format!("{}: {e}", path.display()))?;
        return Err(format!(
            "the sheet did not read back as expected (got {got:?}, wanted {stamp:?}). The file \
             has been put back as it was."
        ));
    }
    println!("\n  confirmed by {stamp}");
    println!("Now: cargo xtask gate {id} && cargo xtask ready {id}");
    Ok(())
}

// ---------------------------------------------------------------------------

/// One recorded fill: a hole, who wrote its body, on what model, and the body.
///
/// The body is kept rather than hashed because the point of keeping it is to
/// run it again. A hash would prove two bodies differed and leave the
/// comparison — the thing the rule is actually asking for — impossible.
pub(super) struct Fill {
    hole: u32,
    by: String,
    model: String,
    body: String,
}

pub(super) fn fills_path(dir: &Path) -> PathBuf {
    dir.join("fills.toml")
}

pub(super) fn read_fills(dir: &Path) -> Vec<Fill> {
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
pub(super) fn check_fill_attribution(
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

pub(super) fn record_fill(
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
pub(super) fn cmd_differential(root: &Path, args: &[&str]) -> Result<(), String> {
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
pub(super) fn cmd_fill(root: &Path, args: &[&str]) -> Result<(), String> {
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
    if let Some(bad) = vleo_sheet::gate::platform_maths(&body).first() {
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
