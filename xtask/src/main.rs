//! `cargo xtask` — the one gate binary.
//!
//! Rust rather than shell scripts or a pipeline-only step: cross-platform,
//! identical on a laptop and in continuous integration. A rule that lives only
//! in the pipeline is a rule half the people working here never see.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vleo_sheet::{emit, gate, load_all, page, Tree};

mod catalogue;
mod convert;
mod design;
mod files;
mod fills;
mod flow;
mod forms;
mod graph;
mod group;
mod group_intake;
mod group_test;
mod hooks;
mod method;
mod pipeline;
mod readers;
mod release;
mod report;
use fills::*;
use forms::*;
use graph::*;
use hooks::*;
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");
    let root = repo_root();
    let rest: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    // The commands in the authoring loop. A person running one of these is
    // about to commit; a person running `status` or `graph` is reading.
    if matches!(cmd, "gate" | "ready" | "fill" | "declare" | "new" | "docs") {
        warn_if_hooks_are_not_wired(&root);
    }

    let checked: Vec<&str> = rest.iter().copied().filter(|a| *a != "--dry-run").collect();
    if let Err(e) = known_flags(cmd, &checked) {
        eprintln!("xtask {cmd}: {e}");
        return ExitCode::FAILURE;
    }

    // --dry-run, on any command: what it would do, doing nothing — its check
    // mode when it has one, the plan from the pipeline table when it does not.
    let r = if rest.contains(&"--dry-run") {
        pipeline::dry_run(&root, cmd, &rest)
    } else {
        dispatch(&root, cmd, &rest)
    };

    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("\x1b[31mxtask: {e}\x1b[0m");
            ExitCode::FAILURE
        }
    }
}

/// Run one command. Every command the program has is here, and in `help`, and
/// in the pipeline table — a test holds the three to each other.
pub(crate) fn dispatch(root: &Path, cmd: &str, rest: &[&str]) -> Result<(), String> {
    let root = root.to_path_buf();
    let rest = rest.to_vec();
    match cmd {
        "docs" => cmd_docs(&root, &rest),
        "assemble" => cmd_assemble(&root, &rest),
        "gate" => cmd_gate(&root, &rest),
        "status" => cmd_status(&root),
        "active" => cmd_active(&root, &rest),
        "reach" => cmd_reach(&root, &rest),
        "catalogue" => catalogue::cmd_catalogue(&root, &rest),
        "impact" => catalogue::cmd_impact(&root, &rest),
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
        "differential" => cmd_differential(&root, &rest),
        "confirm" => cmd_confirm(&root, &rest),
        "lesson" => cmd_lesson(&root, &rest),
        "readers" => readers::cmd_readers(&root, &rest),
        "publish" => cmd_publish(&root, &rest),
        "derisk" => cmd_derisk(&root, &rest),
        "release" => cmd_release(&root, &rest),
        "kit" => cmd_kit(&root, &rest),
        "design" => design::cmd_design(&root, &rest),
        "convert" => convert::cmd_convert(&root, &rest),
        "guides" => cmd_guides(&root),
        "ship" => flow::cmd_ship(&root, &rest),
        "method" => method::cmd_method(&root, &rest),
        "method-wasm" => method::cmd_method_wasm(&root, &rest),
        "files-wasm" => files::cmd_files_wasm(&root, &rest),
        "group-app" => group::cmd_group_app(&root, &rest),
        "group-export" => group::cmd_group_export(&root, &rest),
        "group-intake" => group_intake::cmd_group_intake(&root, &rest),
        "group-build" => group_test::cmd_group_build(&root, &rest),
        "group-test" => group_test::cmd_group_test(&root, &rest),
        "group-deliver" => group_test::cmd_group_deliver(&root, &rest),
        "group-accept" => flow::cmd_group_accept(&root, &rest),
        "rerun" => method::cmd_rerun(&root, &rest),
        "build-node" => method::cmd_build_node(&root, &rest),
        "explain" => pipeline::cmd_explain(&root, &rest),
        "why" => pipeline::cmd_why(&root, &rest),
        "trace" => pipeline::cmd_trace(&root, &rest),
        "pipeline" => pipeline::cmd_pipeline(&root, &rest),
        "help" | "--help" | "-h" => {
            help();
            Ok(())
        }
        other => Err(format!(
            "unknown command '{other}'. Try `cargo xtask help`."
        )),
    }
}

/// Every command and every flag it takes. `help` prints it, and the
/// dispatcher reads each command's flags from it, so a flag this does not
/// list is refused rather than ignored — and a flag that works is one a
/// person can find.
const HELP: &str = "\
cargo xtask <command>

  docs [<node>]      the per-node generators that write files — model,
                     contract, module, evidence and metadata, each from the
                     node's own sheet. A node's page is rendered from its
                     sheet when it is opened, and is never written here.
  assemble           the three assembly generators — the index, the document
                     and the graph tables. They combine and refuse; they never
                     decide, because a decision taken during assembly is a
                     decision nobody reviewed.
  gate [<node>]      the checks, in order, stopping at the first failure.
                     Called by the authoring hook, by the pipeline and by hand.
  status             counts by state and by subsystem, and what is blocking.
  active [<subsystem>] [--names] [--defined]
                     which rows answer and which do not, and for each one that
                     does not, whether it is its own derivation that is missing
                     or a row it reads. A function is defined by its
                     derivation; an input is defined by carrying a value.
  reach [<subsystem>]
                     where each answer GOES: how many reach a KPI closure, and
                     which answer and are read by nothing. A subsystem can
                     answer on every row it has and be wired to nothing.
  catalogue [<group>] [--csv <file>]
                     what each group publishes to the others: every row another
                     group reads, or that crosses a layer, with its version and
                     every row that reads it. Taken from the inputs the sheets
                     declare; --csv writes it as a table.
  impact <node|group> ...
                     which other groups a change to these rows reaches: the
                     rows that read them, theirs, and so on, by group, nearest
                     first. A group named stands for all its rows.
  gap                what every sheet promised and nothing yet covers.
  graph              the three graphs, their sizes, and the crate direction check.
  new <id> --like <sibling>
                     clone the shape of a sibling and blank what must be
                     re-decided. Not a copy: a real copy drags a stale source
                     citation through thirty nodes.
  declare <node>     the completion questions, in order, with what each one is
                     for. A gap left open is not a warning: generation refuses
                     until every one is answered. Add --source <path> to record
                     where the drafting started; --json prints the questions
                     for a tool to read.
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
  codeowners         regenerate CODEOWNERS from areas/teams.toml and the owner
                     each sheet names.
  bundle publish <dir>
                     hash every payload file and write the result into the
                     manifest. Publishing twice from the same input gives the
                     same hash, which is what makes verification mean anything.
                     Publication is irreversible by design.
  bundle verify      re-check every hash in bundles/.
  setup              point git at tools/githooks, so the commit-message hook
                     runs on this clone. One command per person per clone, and
                     the commands that matter say so until it is done.
  lesson form <node> [--out <file.html>]
                     a row's lesson form: one HTML file the node engineer who knows
                     the row fills anywhere, checked as they type by the
                     gate's own lesson check, saved as a filled copy.
  lesson check <file> [--for <node>]
                     what a filled lesson form (or a bare lesson.toml, with
                     --for) holds, and every reason it would be refused.
                     Writes nothing.
  lesson apply <file> [--for <node>] [--check]
                     check it, write it as lesson.toml beside the row's
                     node.toml, and gate the row — or put the row back.
                     --check (what --dry-run runs) only checks.
  publish <node>     move a filled, seeded row to published, so its model,
                     contract and evidence are generated and its holes can be
                     written. Refuses, naming every reason, while it is not ready.
  derisk             write docs/DERISK_NARRATIVE.md and docs/derisking.csv — every
                     recorded change, in the columns of the de-risking narrative,
                     and every registered risk as it stands. Generated from the
                     sheets' [[version]] and [[risk]] records, never edited.
  kit [--bin <dir>] [--out <dir>] [--files-only]
                     the tool as each person gets it: the two programs and
                     the files they read (the web face, the design as one file,
                     design.vleo, and the reference data) in one folder, with
                     START_HERE.md — on Windows the daemon is `Start VLEO.exe`,
                     elsewhere start.sh starts it. No git, no Rust source.
                     Zip the folder and share it. --bin is where the
                     release-built programs are (default target/release);
                     --files-only leaves the programs out, for the Python
                     package (tools/build_wheel.py).
  design [--out <file>]
                     design.vleo: the tree the tool reads — every node folder,
                     the layers, the cases and the source list — written into
                     one SQLite file, which the kit carries in their place and
                     the daemon reads as it reads the folders. Default
                     target/design.vleo.
  design --check <file>
                     the file held to the tree: each file against its SHA-256,
                     the fingerprint, and every file against the folders.
  convert [--out <dir>]
                     the design as its files (docs/PLAN_1_0.md, phase E): each
                     branch's group file and node files, each case, as the
                     shared drive holds them, read back to show they are the
                     tree. To target/converted/, or an empty --out.
  readers [--out <dir>]
                     the docs folder for readers: every row's page and every
                     lesson, read with no tool running — from a shared drive or
                     an internal web server. A lesson's widgets are answered by
                     the engine compiled for the browser. Default target/readers.
  guides             the three role guides, docs/roles/user.html,
                     maintainer.html and developer.html, rendered from
                     docs/manual.toml. Never
                     edited by hand; the pipeline regenerates and compares.
  method <node>      the node's method, checked, and each of its node engineer's test
                     cases run through it — the check the group's
                     application runs as the node engineer types, and the
                     one the gate refuses on.
  method-wasm [--check]
                     rebuild web/method.wasm.gz, the checker the pages
                     carry, from vleo_sheet::method; --check only says
                     whether the committed one is current.
  files-wasm [--check]
                     rebuild web/files.wasm.gz, the design-file library the
                     pages carry, from vleo_files; --check only says whether
                     the committed one is current.
  group-app [--check] the group and node applications, web/group.html and
                     web/node.html — offline pages a group keeps its database
                     files in (docs/GROUP_APPS.md) — and docs/GROUP_FOLDER.md
                     and groups/skill/vleo-group-folder/SKILL.md, all from groups/SPEC.toml;
                     --check only says whether the committed four are current.
  group-intake <folder> [--node <id>] [--apply [--partial]] [--draft]
                     a group's sealed release, written out with `node
                     tools/group_db.mjs --unpack`, taken into the design: the
                     seal checked against every file, then each computed
                     node's pseudocode and results planned against the
                     design — conflicts, an assistant's method or results
                     refused —
                     and with --apply written and gated. --draft looks at an
                     unsealed release and never applies.
  group-build <folder> [--node <id>]
                     every computed node of a sealed release, taken in with
                     group-intake --apply, built from its method: build-node
                     on each — translated, tested on the node engineer's cases, the
                     tests proved to test, the interface checked.
  group-test <folder> [--out <dir>]
                     the group tested against its own results: the design
                     holds the release's cases; each node's tests pass; the
                     group, through the engine, gives results/group.csv; and
                     both ends of every declared range answer or refuse by
                     name. The report goes to target/group/<group>-<version>/.
  group-deliver <folder> [--out <dir>] [--bin <dir>] [--uncommitted]
                     the test application for the group: the kit, built from
                     this commit with their release in it, with DELIVERY.toml
                     (which release, seal, commit, nodes) and DELIVERY.md (what
                     to try). Refused until group-test has passed, and from
                     uncommitted changes unless --uncommitted says throwaway.
  group-accept <file.accept.toml> [--delivery <DELIVERY.toml>] [--no-push]
                     the group's answer to its test application, written by the
                     group application, recorded in acceptances/ on the branch
                     group/<group>-<version> it was built on. An answer of
                     changes is never recorded: its note is printed to take back.
  group-accept --verify <branch>
                     the pipeline's check on a group branch: it carries the
                     group's acceptance of exactly what is on it.
  group-export <group> [--out <dir>]
                     a group's folder in the pattern, written from every sheet
                     in the group, for the group to start from. It invents
                     nothing: what the tree lacks is left for the group, and
                     the group application lists it. Default target/groups/.
  group-export --all [--out <dir>]
                     every group that owns a node, each in its own folder, and
                     GROUPS.csv: whose each is and how far the design carries
                     it. Default target/groups/all/.
  rerun <node>|--all [--require]
                     the node engineer's own code run again on their cases: Python
                     directly, MATLAB and Octave through Octave; anything else
                     is kept and read, not rerun.
  build-node <node>  from a node's method to a connected node, in order: the
                     method on its cases, the translation into the kernel, the
                     node's tests, the node engineer's code rerun — and only then
                     the interface.
  ship <version> [--no-push] [--no-test]
                     the release branch release/<version> from main: the
                     de-risking narrative, the stamp, regenerate, gate, test,
                     commit, push — and the tag commands for after the merge.
  release <version> [--check]
                     stamp every node version still marked `next` with this
                     release, set the workspace version, and regenerate. The
                     node's record then says which release carried each belief.
                     --check refuses while anything is unstamped or newer.
  variables          write docs/VARIABLES.md — every variable in the tree, its
                     unit, its range, the reason for each bound, and what reads
                     it. Generated, because a register maintained by hand is a
                     register that is wrong.
  explain [<command>]
                     where a command sits in a node's journey, what it reads,
                     writes and checks, how to undo it, its steps and where its
                     code is — from the one table docs/PIPELINE.md is written
                     from. With no command, the whole journey.
  why <node>         a node's history in one place: every recorded version and
                     who made it, the commits that changed it, its approvals,
                     how its code came to be, and its gate, run now.
  trace [<command>] [--list]
                     the last run's trace — every command that writes leaves
                     one in target/xtask-trace/, the newest 50 kept — or the
                     last of one command, or --list every one kept.
  pipeline [--check] write docs/PIPELINE.md from the pipeline table; --check
                     only says whether it is current.

Every command that writes also takes --dry-run: its check mode where it has
one, otherwise the plan — its steps, what it would write, how to undo it —
with nothing touched. The tree is seeded once, ever, by tools/seed_tree.py.";

fn help() {
    println!("{HELP}");
}

/// The flags `help` lists for one command: every `--flag` in each of its
/// entries (a command may have more than one, as `approve` does).
fn flags_in_help(cmd: &str) -> Option<Vec<&'static str>> {
    let lines: Vec<&str> = HELP.lines().collect();
    let names = |l: &str| {
        l.starts_with("  ") && !l.starts_with("   ") && l.split_whitespace().next() == Some(cmd)
    };
    let mut flags: Vec<&str> = Vec::new();
    let mut found = false;
    let mut i = 0;
    while i < lines.len() {
        if names(lines[i]) {
            found = true;
            let mut j = i;
            loop {
                flags.extend(
                    lines[j]
                        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
                        .filter(|w| w.starts_with("--") && w.len() > 2),
                );
                j += 1;
                if j >= lines.len() || !lines[j].starts_with("   ") {
                    break;
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    flags.sort();
    flags.dedup();
    found.then_some(flags)
}

/// A flag the command does not take is refused, by name, before it runs.
///
/// Flags were read by searching the arguments for the ones a command knew,
/// so `--aply` or `--dry-run` was passed over and the command ran as though
/// it had not been given — the one mistake a person cannot see.
fn known_flags(cmd: &str, rest: &[&str]) -> Result<(), String> {
    let Some(allowed) = flags_in_help(cmd) else {
        return Ok(()); // not a command `help` lists: the dispatcher says so
    };
    for a in rest {
        let flag = a.split('=').next().unwrap_or(a);
        if flag.starts_with("--") && !allowed.contains(&flag) {
            return Err(if allowed.is_empty() {
                format!("{flag} is not a flag it takes; it takes none")
            } else {
                format!(
                    "{flag} is not a flag it takes; it takes {}",
                    allowed.join(", ")
                )
            });
        }
    }
    Ok(())
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
    Ok(load_all(root)?)
}

/// The design as its files state it (`design/`), read as the folders they
/// were converted from: what every command that reads the design, and does
/// not write it, reads.
fn read(root: &Path) -> Result<Tree, String> {
    let dir = root.join("design");
    let (files, _) = vleo_files::convert::read_folder(&dir)
        .map_err(|e| format!("the design folder {} does not open: {e}", dir.display()))?;
    let served = vleo_files::convert::Served::new(
        root,
        &files,
        std::sync::Arc::new(vleo_sheet::files::Disk),
    )
    .map_err(|e| format!("the design folder {} does not read: {e}", dir.display()))?;
    vleo_sheet::load::load_all_from(&served, root)
        .map_err(|e| format!("the design folder {} does not load: {e}", dir.display()))
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

/// Move a seeded row to published, from a terminal.
fn cmd_publish(root: &Path, args: &[&str]) -> Result<(), String> {
    use pipeline::{OnStop, Run};
    let id = *args.first().ok_or("usage: cargo xtask publish <node>")?;
    let mut run = Run::start(root, "publish", args, 2);
    let again = format!("cargo run -p xtask -- publish {id}");
    let base = run.step(
        "read the sheet",
        OnStop::new("unchanged — nothing was written", &again),
        || {
            let tree = load(root)?;
            let sh = tree
                .sheets
                .get(id)
                .ok_or_else(|| format!("no node '{id}'"))?;
            let base = fs::read_to_string(sh.dir.join("node.toml"))
                .map(|t| vleo_sheet::form::file_hash(&t))
                .map_err(|e| e.to_string())?;
            Ok((base, format!("{id} — {}", sh.label)))
        },
    )?;
    run.step(
        "publish, generate and gate",
        OnStop::new(
            "as they were — a publish that does not gate is put back whole",
            format!("`cargo run -p xtask -- declare {id}` says what is still open; then {again}"),
        ),
        || match vleo_sheet::form::publish(root, id, &base) {
            vleo_sheet::form::Saved::Ok { regenerated, .. } => Ok((
                (),
                format!("{regenerated} artefact(s) generated, the whole tree gated"),
            )),
            vleo_sheet::form::Saved::Stale { .. } => {
                Err("the sheet changed while publishing".into())
            }
            vleo_sheet::form::Saved::Refused(e) => Err(e),
        },
    )?;
    run.done(&format!(
        "published {id}. Its holes are next — `cargo run -p xtask -- fill {id} --hole <n> --body <file>`."
    ));
    Ok(())
}

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
            vec![("meta.json", emit::meta_json(sh, &gaps))]
        } else {
            vec![
                ("model.rs", emit::model_rs(sh, &holes)),
                ("contract.rs", emit::contract_rs(sh)),
                ("mod.rs", emit::mod_rs(sh)),
                ("evidence.rs", emit::evidence_rs(sh)),
                ("meta.json", emit::meta_json(sh, &gaps)),
            ]
        };
        // A node's page is rendered from its sheet when it is opened; a copy
        // left in the folder from before would be read by nothing and believed
        // by whoever opened it.
        let stale = sh.dir.join("page.html");
        if stale.is_file() {
            fs::remove_file(&stale).map_err(|e| format!("{}: {e}", stale.display()))?;
            written += 1;
        }
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

fn cmd_assemble(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = read(root)?;
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
        let t = page::fragment(sh, &vleo_sheet::load::read_holes(&sh.dir), &tree);
        bytes += t.len();
        fs::write(frag_dir.join(format!("{}.html", sh.id)), t)
            .map_err(|e| format!("fragment {}: {e}", sh.id))?;
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
    let tree = read(root)?;
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

/// The producing row behind a variable id.
///
/// A variable id is a node id or `<node id>.<extra>` — a node id never contains
/// a dot — so the producer is everything before the first one.
use vleo_sheet::text::producer_of;

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
