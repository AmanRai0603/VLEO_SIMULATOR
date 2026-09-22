//! `cargo xtask` — the one gate binary.
//!
//! Rust rather than shell scripts or a pipeline-only step: cross-platform,
//! identical on a laptop and in continuous integration. A rule that lives only
//! in the pipeline is a rule half the team never sees.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vleo_sheet::{emit, gate, load_all, page, Tree};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");
    let root = repo_root();
    let rest: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    // The commands in the authoring loop. A person running one of these is
    // about to commit; a person running `status` or `graph` is reading.
    if matches!(cmd, "gate" | "ready" | "fill" | "declare" | "new" | "docs") {
        warn_if_hooks_are_not_wired(&root);
    }

    let r = match cmd {
        "docs" => cmd_docs(&root, &rest),
        "assemble" => cmd_assemble(&root, &rest),
        "gate" => cmd_gate(&root, &rest),
        "status" => cmd_status(&root),
        "active" => cmd_active(&root, &rest),
        "reach" => cmd_reach(&root, &rest),
        "gap" => cmd_gap(&root),
        "graph" => cmd_graph(&root),
        "new" => cmd_new(&root, &rest),
        "declare" => cmd_declare(&root, &rest),
        "fill" => cmd_fill(&root, &rest),
        "ready" => cmd_ready(&root, &rest),
        "codeowners" => cmd_codeowners(&root),
        "bundle" => cmd_bundle(&root, &rest),
        "variables" => cmd_variables(&root),
        "setup" => cmd_setup(&root),
        "mutate" => cmd_mutate(&root, &rest),
        "differential" => cmd_differential(&root, &rest),
        "confirm" => cmd_confirm(&root, &rest),
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
  active [<subsystem>]
                     which rows answer and which do not, and for each one that
                     does not, whether it is its own derivation that is missing
                     or a row it reads. A function is defined by its
                     derivation; an input is defined by carrying a value.
  reach [<subsystem>]
                     where each answer GOES: how many reach a KPI closure, and
                     which answer and are read by nothing. A subsystem can
                     answer on every row it has and be wired to nothing.
  gap                what every sheet promised and nothing yet covers.
  graph              the three graphs, their sizes, and the crate direction check.
  new <id> --like <sibling>
                     clone the shape of a sibling and blank what must be
                     re-decided. Not a copy: a real copy drags a stale source
                     citation through thirty nodes.
  declare <node>     the completion questions, in order, with what each one is
                     for. A gap left open is not a warning: generation refuses
                     until every one is answered. Add --source <path> to record
                     where the drafting started.
  fill <node> --hole <n> --body <file|->
                     splice one hole body into a generated model.rs. The hole
                     filler is never handed the file: it returns the few typed
                     lines as text and this puts them where they go. An agent
                     given the file and told not to stray is not constrained.
  differential <node>
                     re-run the node against every other recorded body for the
                     same hole. A significant node is filled twice by different
                     models and the two are compared; this is the comparison.
                     Recorded by `fill --by`, which refuses a second body from
                     the model that wrote the first.
  confirm --list [<subsystem>]
                     the relations with nobody's name against them, grouped by
                     the owner who has to supply one.
  confirm <node> --by \"<name>\"
                     put a person's name against one relation, after printing
                     the relation and its source so the act is informed. There
                     is no flag that does many at once, and that is deliberate.
  ready [<node>]     whether a person should be asked to look yet: the gate,
                     then the gap pass, then what criticality demands. A node
                     with an open gap does not enter H2 — the reviewer accepts,
                     they do not hunt for defects a machine finds free.
  codeowners         regenerate CODEOWNERS from the layer files.
  bundle publish <dir>
                     hash every payload file and write the result into the
                     manifest. Publishing twice from the same input gives the
                     same hash, which is what makes verification mean anything.
                     Publication is irreversible by design.
  bundle verify      re-check every hash in bundles/.
  mutate [<node>]    perturb the answer by a tenth of a percent and require the
                     node's own tests to notice. A test that passes against a
                     wrong number proves nothing, and nothing else in the gate
                     can tell the difference between evidence and decoration.
  setup              point git at tools/githooks, so the commit-message hook
                     runs on this clone. One command per person per clone, and
                     the commands that matter say so until it is done.
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


/// Today, as the sheets write it.
fn today() -> String {
    // The sheets carry a plain ISO date and nothing reads it as a timestamp, so
    // a date is all this needs. Taken from the system clock through `date`
    // rather than a crate, because adding a dependency to stamp a date is how a
    // dependency list stops meaning anything.
    std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// The relations with nobody's name against them, and who owes each one.
///
/// Two thirds of the tree is declared values and every one of those already
/// carries a confirmation. What is missing is the other third: the relations,
/// where the name matters most, because a declared value that nobody confirmed
/// is a number somebody has to defend and a relation that nobody confirmed
/// might not have come from a person at all.
fn cmd_confirm(root: &Path, args: &[&str]) -> Result<(), String> {
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

    // An agent may never supply mathematics. Stated as a sentence it is a hope;
    // this makes it a fact about what can be written to the file.
    let lower = who.to_lowercase();
    for bad in vleo_sheet::form::agent_identities(root) {
        if lower == bad
            || lower.starts_with(&format!("{bad} "))
            || lower.contains(&format!("{bad}/"))
        {
            return Err(format!(
                "refused: '{who}' is an agent. An agent may never supply mathematics, and this \
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
struct Fill {
    hole: u32,
    by: String,
    model: String,
    body: String,
}

fn fills_path(dir: &Path) -> PathBuf {
    dir.join("fills.toml")
}

fn read_fills(dir: &Path) -> Vec<Fill> {
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

/// What model an agent runs, from the one file that records it.
fn agent_model(root: &Path, who: &str) -> Result<String, String> {
    let text = fs::read_to_string(root.join("agents/provenance.toml"))
        .map_err(|e| format!("agents/provenance.toml: {e}"))?;
    let v: toml::Value = text
        .parse()
        .map_err(|e| format!("agents/provenance.toml: {e}"))?;
    let agents = v
        .get("agent")
        .and_then(|a| a.as_array())
        .ok_or("agents/provenance.toml declares no agents")?;
    let known: Vec<String> = agents
        .iter()
        .filter_map(|a| a.get("name")?.as_str().map(str::to_string))
        .collect();
    agents
        .iter()
        .find(|a| {
            a.get("name").and_then(|n| n.as_str()) == Some(who)
                || a.get("id").and_then(|n| n.as_str()) == Some(who)
        })
        .and_then(|a| a.get("model")?.as_str().map(str::to_string))
        .ok_or_else(|| {
            format!(
                "no agent '{who}' with a model in agents/provenance.toml. Known: {}",
                known.join(", ")
            )
        })
}

/// Append a fill record.
///
/// The model-family rule, enforced as a fact about the file rather than as a
/// sentence in a prompt: a second body for the same hole from the same model is
/// refused. A model handed its own reasoning to check approves it, so two
/// bodies from one model are one body written twice.
fn check_fill_attribution(
    root: &Path,
    sh: &vleo_sheet::model::Sheet,
    hole: u32,
    who: &str,
    body: &str,
) -> Result<(), String> {
    let model = agent_model(root, who)?;
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

fn record_fill(
    root: &Path,
    sh: &vleo_sheet::model::Sheet,
    hole: u32,
    who: &str,
    body: &str,
) -> Result<(), String> {
    let model = agent_model(root, who)?;
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
             # Written by `cargo xtask fill --by`. Editing it by hand defeats the\n\
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
/// What this cannot do here is produce the second body: every agent in this
/// repository runs one vendor's models, so the rule separates model tiers and
/// not training. That is recorded in agents/provenance.toml and it is a
/// decision with a cost attached, not something this command can close.
fn cmd_differential(root: &Path, args: &[&str]) -> Result<(), String> {
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
            "no fills recorded for '{id}'. `cargo xtask fill --by <agent>` records one; \
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
const MUTANT_FLOOR: f64 = 1e-3;

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
fn mutant_scale(sh: &vleo_sheet::model::Sheet) -> f64 {
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
fn cmd_mutate(root: &Path, args: &[&str]) -> Result<(), String> {
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
fn mutate_answer(src: &str, scale: f64) -> Option<String> {
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

/// Where the hooks live, relative to the repository root.
const HOOKS_PATH: &str = "tools/githooks";

/// What `core.hooksPath` is set to on this clone, if anything.
///
/// Read through git rather than by parsing `.git/config`: the setting can come
/// from the repository, the user or the system, and only git knows which one
/// won.
fn configured_hooks_path(root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .current_dir(root)
        .args(["config", "--get", "core.hooksPath"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// Whether the commit-message hook will actually run here.
fn hooks_are_wired(root: &Path) -> bool {
    configured_hooks_path(root).is_some_and(|p| p == HOOKS_PATH)
}

/// Say so, once, on the commands a person runs by hand.
///
/// A warning rather than a refusal: the pipeline has no hooks and does not need
/// them — it re-checks every rule a hook checks, which is the point of a hook
/// being a convenience and not a control. But a person whose hook never ran
/// finds out at review, and that is the expensive place to find out.
fn warn_if_hooks_are_not_wired(root: &Path) {
    if std::env::var_os("CI").is_some() || hooks_are_wired(root) {
        return;
    }
    eprintln!(
        "\x1b[33mnote: the commit-message hook is not installed on this clone.\n      \
         Run `cargo xtask setup` once. Without it a bad commit subject is\n      \
         caught in the pipeline instead of before the commit.\x1b[0m"
    );
}

/// Install the hooks on this clone.
///
/// Git will not follow a path committed to the repository on its own: a hook
/// that ran because it was cloned would be arbitrary code from a pull request.
/// So it is one command per person, and this is that command rather than a
/// line of prose someone has to find.
fn cmd_setup(root: &Path) -> Result<(), String> {
    let hooks = root.join(HOOKS_PATH);
    if !hooks.is_dir() {
        return Err(format!("{} is not a directory", hooks.display()));
    }

    match configured_hooks_path(root) {
        Some(p) if p == HOOKS_PATH => {
            println!("core.hooksPath is already {HOOKS_PATH}");
        }
        other => {
            if let Some(p) = &other {
                println!("core.hooksPath was {p}");
            }
            let st = std::process::Command::new("git")
                .current_dir(root)
                .args(["config", "core.hooksPath", HOOKS_PATH])
                .status()
                .map_err(|e| format!("running git: {e}"))?;
            if !st.success() {
                return Err("git config core.hooksPath failed".into());
            }
            // Read it back. Setting a value and reporting success without
            // looking is how a setup command comes to be trusted wrongly.
            if !hooks_are_wired(root) {
                return Err(format!(
                    "git config accepted core.hooksPath={HOOKS_PATH} and reading it back \
                     gave something else — a user or system setting is overriding it"
                ));
            }
            println!("core.hooksPath is now {HOOKS_PATH}");
        }
    }

    let mut listed = 0usize;
    for e in fs::read_dir(&hooks).map_err(|e| format!("{}: {e}", hooks.display()))? {
        let e = e.map_err(|e| format!("{e}"))?;
        let name = e.file_name().to_string_lossy().to_string();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let m = e.metadata().map_err(|e| format!("{e}"))?;
            if m.permissions().mode() & 0o111 == 0 {
                return Err(format!(
                    "{name} is not executable — git runs hooks by executing them, so this \
                     one would be skipped silently"
                ));
            }
        }
        println!("  hook: {name}");
        listed += 1;
    }
    if listed == 0 {
        return Err(format!("{} is empty", hooks.display()));
    }
    Ok(())
}

// ---------------------------------------------------------------------------


fn cmd_docs(root: &Path, args: &[&str]) -> Result<(), String> {
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






fn cmd_declare(root: &Path, args: &[&str]) -> Result<(), String> {
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

    let asks = vleo_sheet::form::asks(sh)?;
    let mut open = 0usize;
    for a in &asks {
        if a.open {
            open += 1;
            println!("  \x1b[33m?\x1b[0m  {}", a.ask);
            println!("     {} — without it: {}", a.field, a.why);
        } else {
            println!(
                "  \x1b[32m·\x1b[0m  {} = {}",
                a.field,
                truncate(vleo_sheet::form::value(sh, a.field), 68)
            );
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
        println!("     a relation an agent invented carries a citation just as convincingly, and");
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
        println!("     [maths] confirmed_by — an agent may never supply mathematics, and without");
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

fn truncate(s: &str, n: usize) -> String {
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
fn cmd_ready(root: &Path, args: &[&str]) -> Result<(), String> {
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

    // Everything that can refuse, refuses before the file is touched. A splice
    // that lands and then reports "nothing was written" is worse than either
    // outcome on its own.
    let attribution = args
        .iter()
        .position(|a| *a == "--by")
        .and_then(|i| args.get(i + 1))
        .copied();
    match attribution {
        Some(who) => check_fill_attribution(root, sh, n, who, &body)?,
        None if sh.criticality == "significant" => {
            return Err(format!(
                "'{id}' is significant, so its holes are filled twice by different models and \
                 the two compared. An unattributed body cannot be compared to anything: pass \
                 --by <agent>. Nothing was written."
            ))
        }
        None => {}
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
    if let Some(who) = attribution {
        record_fill(root, sh, n, who, &body)?;
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
fn clone_sheet(sheet: &str, id: &str, folder: &str, src_order: u32) -> (String, Vec<String>) {
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
    for line in sheet.lines() {
        let l = line.trim_start();
        if l.starts_with('[') {
            section = l.to_string();
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
        let f = sh.dir.join("node.toml");
        let t = fs::read_to_string(&f).map_err(|e| e.to_string())?;
        let mut w = String::new();
        for line in t.lines() {
            if line.trim_start().starts_with("order = ") {
                w.push_str(&format!("order = {}\n", sh.order + 1));
            } else {
                w.push_str(line);
                w.push('\n');
            }
        }
        fs::write(&f, w).map_err(|e| e.to_string())?;
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
enum Verdict {
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
fn producer_of(var: &str) -> &str {
    var.split('.').next().unwrap_or(var)
}

/// Every row's verdict, computed once from the sheets.
fn active_verdicts(tree: &Tree) -> BTreeMap<&str, Verdict> {
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
fn reaching_kpi<'a>(
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
fn work_behind(
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
fn cmd_reach(root: &Path, args: &[&str]) -> Result<(), String> {
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

fn cmd_active(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;
    let only = args.iter().find(|a| !a.starts_with("--")).copied();
    let verdict = active_verdicts(&tree);

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
mod tests {
    use super::clone_sheet;

    /// A sibling sheet with one of every field a clone has carried wrongly.
    const SIBLING: &str = r#"# A long comment block that is prose about the sibling row and its own scale.
id = "sw_sibling"
label = "The sibling"
folder = "sw_sibling"
kind = "declared"
order = 40
state = "published"

[question]
text = "What does the sibling answer?"
note = "a note about the sibling"

[maths]
confirmed_by = "A. Person / 2026-01-01"
expression = "X = 3"
source = "some_source"

[theory]
why = """
Three paragraphs about why the SIBLING's relation is that relation.
"""
reading = """
What the sibling's answer is and is not.
"""

[[assumption]]
text = "an assumption about the sibling"
fails_when = "the sibling's own failure mode, which is not this row's"

[output]
symbol = "X_sib"
type = "Ratio"
unit = "One"
lower = 1.0
upper = 3.0
reason_lower = "the sibling's lower reason"
reason_upper = "the sibling's upper reason"

[[algorithm.step]]
number = 1
text = "the sibling's one step"

[value]
number = 3.0
confirmed_by = "A. Person / 2026-01-01"
"#;

    fn clone() -> String {
        clone_sheet(SIBLING, "sw_new", "sw_new", 40).0
    }

    /// A NEW ROW IS SEEDED, whatever the sibling is. Inheriting `published` gave a
    /// folder with every field blank a state that means "specified": it counted as
    /// published in `xtask status`, in the index the face reads and in
    /// /v1/branches' idea of an active branch.
    #[test]
    fn a_clone_is_seeded() {
        let out = clone();
        assert!(out.contains("state = \"empty\""), "{out}");
        assert!(!out.contains("state = \"published\""), "{out}");
    }

    /// The identifiers are the new row's, and the place is the one after the
    /// sibling — copying `order` guarantees the collision the assembly check
    /// catches.
    #[test]
    fn identity_and_place_are_the_new_rows() {
        let out = clone();
        assert!(out.contains("id = \"sw_new\""), "{out}");
        assert!(out.contains("folder = \"sw_new\""), "{out}");
        assert!(out.contains("order = 41"), "{out}");
    }

    /// Everything a person must re-decide comes back blank, and the sibling's
    /// answers do not survive anywhere in the file.
    #[test]
    fn what_must_be_re_decided_is_blank() {
        let out = clone();
        for gone in [
            "The sibling",                   // label
            "What does the sibling answer?", // question text
            "a note about the sibling",      // note
            "X = 3",                         // expression
            "some_source",                   // source
            "A. Person / 2026-01-01",        // confirmed_by, twice
            "X_sib",                         // symbol
            "the sibling's lower reason",
            "the sibling's upper reason",
            // A `fails_when` is the other half of an assumption whose `text` is
            // blanked, so inheriting it left the sheet stating how a claim it no
            // longer makes would fail.
            "the sibling's own failure mode",
            // The theory tab is a derivation of the SIBLING's relation, sitting
            // beside a blanked expression.
            "why the SIBLING's relation",
            "What the sibling's answer is and is not",
        ] {
            assert!(
                !out.contains(gone),
                "a clone still carries {gone:?}:\n{out}"
            );
        }
    }

    /// `number` is a value under [value] and a step index under
    /// [[algorithm.step]]. Blanking by key alone renumbers the algorithm; not
    /// blanking at all leaves a value nobody picked beside a blanked signature.
    #[test]
    fn a_value_is_blanked_and_a_step_index_is_not() {
        let out = clone();
        assert!(
            out.contains("number = 0.0   # REQUIRED"),
            "the value was not blanked:\n{out}"
        );
        assert!(
            out.contains("number = 1\n"),
            "the step index was blanked:\n{out}"
        );
    }

    /// And it is 0.0 rather than "": the sheet must stay TOML that the tool's own
    /// next command can read.
    #[test]
    fn the_clone_is_still_toml() {
        let out = clone();
        let parsed: Result<toml::Value, _> = out.parse();
        assert!(
            parsed.is_ok(),
            "a clone does not parse: {:?}\n{out}",
            parsed.err()
        );
    }

    /// The comment blocks cannot be blanked — nothing marks which sentences are
    /// about the sibling — so they are reported instead of carried silently.
    #[test]
    fn carried_comments_are_reported() {
        let (_, carried) = clone_sheet(SIBLING, "sw_new", "sw_new", 40);
        assert!(
            carried
                .iter()
                .any(|c| c.contains("prose about the sibling")),
            "the sibling's comment block was carried without being named: {carried:?}"
        );
    }
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
