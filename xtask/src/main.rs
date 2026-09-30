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

mod authoring;
mod flow;
mod forms;
mod generate;
mod method;
mod pipeline;
mod release;
mod report;

use authoring::*;
use forms::*;
use generate::*;
use release::*;
use report::*;

/// A reader that stops early — `| head`, `| grep -m1`, a pager quit halfway —
/// closes the pipe, and the next line printed panics with a backtrace that
/// reads like a crash in this program. It is not one: the reader had what it
/// wanted. So that one panic ends the program quietly, and every other panic
/// is reported exactly as before. (Restoring the default SIGPIPE disposition
/// would need `unsafe`, which this crate forbids.)
fn quiet_when_the_reader_stops() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("");
        if msg.starts_with("failed printing to stdout") && msg.contains("Broken pipe") {
            std::process::exit(0);
        }
        default(info);
    }));
}

fn main() -> ExitCode {
    quiet_when_the_reader_stops();
    let started = std::time::Instant::now();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");
    let root = repo_root();
    let rest: Vec<&str> = args
        .iter()
        .skip(1)
        .map(|s| s.as_str())
        .filter(|a| *a != "--dry-run")
        .collect();
    // --dry-run on any command: what it would do, from the same table
    // `explain` reads, and nothing run.
    if args.iter().any(|a| a == "--dry-run") && !matches!(cmd, "help" | "--help" | "-h") {
        let r = pipeline::dry_run(&root, cmd);
        pipeline::trace(&root, &args, started, &r);
        return match r {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("\x1b[31mxtask: {e}\x1b[0m");
                ExitCode::FAILURE
            }
        };
    }

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
        "form" => cmd_form(&root, &rest),
        "intake" => cmd_intake(&root, &rest),
        "publish" => cmd_publish(&root, &rest),
        "derisk" => cmd_derisk(&root, &rest),
        "release" => cmd_release(&root, &rest),
        "kit" => cmd_kit(&root, &rest),
        "take" => flow::cmd_take(&root, &rest),
        "guides" => cmd_guides(&root),
        "preview" => flow::cmd_preview(&root, &rest),
        "approve" => flow::cmd_approve(&root, &rest),
        "queue" => flow::cmd_queue(&root, &rest),
        "ship" => flow::cmd_ship(&root, &rest),
        "method" => method::cmd_method(&root, &rest),
        "method-wasm" => method::cmd_method_wasm(&root, &rest),
        "rerun" => method::cmd_rerun(&root, &rest),
        "build-node" => method::cmd_build_node(&root, &rest),
        "migration" => method::cmd_migration(&root, &rest),
        "explain" => pipeline::cmd_explain(&root, &rest),
        "why" => pipeline::cmd_why(&root, &rest),
        "pipeline" => pipeline::cmd_pipeline(&root, &rest),
        "help" | "--help" | "-h" => {
            help();
            Ok(())
        }
        other => Err(format!(
            "unknown command '{other}'. Try `cargo xtask help`."
        )),
    };

    pipeline::trace(&root, &args, started, &r);
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
  active [<subsystem>] [--names]
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
  fill <node> --hole <n> --body <file|-> [--by <who> --model <model>]
                     splice one hole body into a generated model.rs. Whoever
                     writes the body — a developer, or an assistant a developer
                     runs — returns the few typed lines as text and this puts
                     them where they go: nothing is handed the whole file.
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
  form <node>|--new [--example] [--out <file.html>]
                     a node's form: one HTML file that explains itself, asks
                     every question the sheet answers, lists every row it could
                     read, and saves a filled copy. --new is the form for a node
                     the design does not have yet, which also asks where it goes.
                     Anyone can fill it, offline, by hand or with an assistant;
                     the filled file comes back to a developer. --example fills
                     orbit_velocity's form with the worked example, for the
                     pipeline's end-to-end test of the method path only.
  intake <file.html> [--apply [--partial]]
                     the checker: what a filled form would change, field by
                     field, and every interface it declares — each input a row
                     that exists, of the quantity expected. What it cannot do is
                     named: a conflict with a change made since, a relation an
                     assistant supplied. --apply writes it (a new node is built
                     in its place in the tree), regenerates, gates, and puts
                     everything back on a refusal. Known-good values come out as
                     a request, never written.
  publish <node>     move a filled, seeded row to published, so its model,
                     contract and evidence are generated and its holes can be
                     written. Refuses, naming every reason, while it is not ready.
  derisk             write docs/DERISK_NARRATIVE.md and docs/derisking.csv — every
                     recorded change, in the columns of the de-risking narrative,
                     and every registered risk as it stands. Generated from the
                     sheets' [[version]] and [[risk]] records, never edited.
  kit [--bin <dir>] [--out <dir>] [--files-only]
                     the tool as a team member gets it: the two programs and
                     the files they read (the web face, the tree, its pages,
                     the reference data) in one folder, with START_HERE.md —
                     on Windows the daemon is `Start VLEO.exe`, elsewhere
                     start.sh starts it. No git, no Rust source beyond the node
                     folders. Zip the folder and share it. --bin is where the
                     release-built programs are (default target/release);
                     --files-only leaves the programs out, for the Python
                     package (tools/build_wheel.py).
  guides             the three role guides, docs/roles/user.html,
                     maintainer.html and developer.html, rendered from
                     docs/manual.toml. Never
                     edited by hand; the pipeline regenerates and compares.
  method <node>      the node's method, checked, and each of its author's test
                     cases run through it — the check the form runs as the
                     author types, and the one the gate refuses on.
  method-wasm [--check]
                     rebuild web/method.wasm.gz, the checker every node form
                     carries, from vleo_sheet::method; --check only says
                     whether the committed one is current.
  rerun <node>|--all [--require]
                     the author's own code run again on their cases: Python
                     directly, MATLAB and Octave through Octave; anything else
                     is kept and read, not rerun.
  build-node <node>  from a node's method to a connected node, in order: the
                     method on its cases, the translation into the kernel, the
                     node's tests, the author's code rerun, a mutation the
                     tests must catch — and only then the interface.
  migration [--owner <o>] [--subsystem <s>] [--forms <dir>]
                     which computed rows still need a method, by owner, and
                     with --forms their node forms written ready to send.
                     Nothing here writes a method: each comes from its owner.
  take <form.html> --for <author> [--again] [--no-push] [--no-test]
                     the maintainer's first step: check a filled node form;
                     if it cannot be taken, write <form>.returned.txt to send
                     back and change nothing; otherwise put it on its own
                     branch form/<author>/<node> from a fresh main, apply it,
                     regenerate, gate, test, commit naming the author, push.
  preview            where the current form branch's preview build is — every
                     push to a form branch builds one — and what to do with it.
  approve <approval.toml> [--no-push]
                     the author's approval of a preview, checked against this
                     branch: it must be for the build of what is here now.
                     Recorded in approvals/, committed and pushed.
  approve --verify <branch>
                     the same check, as the pipeline runs it on a form branch's
                     pull request.
  queue              every form branch and where it stands: waiting for the
                     author's approval, approved, merged.
  ship <version> [--no-push] [--no-test]
                     the release branch release/<version> from main: the
                     de-risking narrative, the stamp, regenerate, gate, test,
                     commit, push — and the tag commands for after the merge.
  release <version> [--check]
                     stamp every node version still marked `next` with this
                     release, set the workspace version, and regenerate. The
                     node's record then says which release carried each belief.
                     --check refuses while anything is unstamped or newer.
  explain [<command>] what a command does, in order, what it writes and what it
                     starts — read from docs/manual.toml. `--dry-run` on any
                     command prints the same and runs nothing.
  why <path>         which commands write this file: generated, or a source.
  pipeline [--check] write docs/PIPELINE.md, every command's steps and every
                     generated file's writer; --check only says if it is current.
                     Every run is recorded in target/xtask-trace.log.
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

/// A node's place on the tree, set in its node.toml: the top-level `order`
/// and nothing else, every comment and every other line kept as it was. A
/// TOML edit rather than a line match, so an `order` in some table of the
/// sheet is never the one changed.
fn set_order(path: &Path, order: u32) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    fs::write(
        path,
        with_order(&text, order).map_err(|e| format!("{}: {e}", path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))
}

fn with_order(text: &str, order: u32) -> Result<String, String> {
    let mut doc: toml_edit::DocumentMut = text.parse().map_err(|e| format!("{e}"))?;
    let Some(item) = doc.get_mut("order").filter(|i| i.is_value()) else {
        return Err("no top-level `order` to move".into());
    };
    let decor = item.as_value().map(|v| v.decor().clone());
    *item = toml_edit::value(i64::from(order));
    if let (Some(d), Some(v)) = (decor, item.as_value_mut()) {
        *v.decor_mut() = d;
    }
    Ok(doc.to_string())
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

/// Today, as the sheets write it (vleo_data::clock: no subprocess, so the same
/// on Windows).
fn today() -> String {
    vleo_data::clock::today()
}

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
mod moving_a_node {
    use super::with_order;

    #[test]
    fn only_the_top_level_order_moves_and_every_comment_stays() {
        let text = "# why this row exists\nid = \"x\"\norder = 40   # its place\n\n[[case]]\norder = 3\n# a reason\n";
        let out = with_order(text, 41).unwrap();
        assert_eq!(
            out,
            "# why this row exists\nid = \"x\"\norder = 41   # its place\n\n[[case]]\norder = 3\n# a reason\n"
        );
        assert!(with_order("id = \"x\"\n[view]\norder = 2\n", 5).is_err());
    }

    /// On every sheet in the tree, setting a node's order to what it already
    /// is gives back the file byte for byte: the edit changes the number and
    /// nothing else a person wrote.
    #[test]
    fn every_sheet_in_the_tree_comes_back_unchanged() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let tree = vleo_sheet::load_all(&root).unwrap();
        let mut n = 0;
        for sh in tree.ordered() {
            let text = std::fs::read_to_string(sh.dir.join("node.toml")).unwrap();
            assert_eq!(with_order(&text, sh.order).unwrap(), text, "{}", sh.id);
            n += 1;
        }
        assert!(n > 1000);
    }

    /// The rings point inward, checked by `cargo test` and not only by the
    /// manual's run of `xtask graph`: a crate that reached outward for a
    /// helper once went unseen until that run.
    #[test]
    fn every_crate_dependency_points_inward() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let wrong = crate::crate_direction(&root).unwrap();
        assert!(wrong.is_empty(), "{wrong:#?}");
    }
}
