//! `cargo xtask` — the one gate binary.
//!
//! Rust rather than shell scripts or a pipeline-only step: cross-platform,
//! identical on a laptop and in continuous integration. A rule that lives only
//! in the pipeline is a rule half the people working here never see.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vleo_sheet::{emit, gate, page, Tree};

mod catalogue;
mod files;
mod flow;
mod forms;
mod graph;
mod group;
mod hooks;
mod method;
mod pipeline;
mod readers;
mod release;
mod report;
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
    if matches!(cmd, "gate" | "ready" | "docs") {
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
        "docs" => cmd_docs(&root),
        "assemble" => cmd_assemble(&root, &rest),
        "gate" => cmd_gate(&root, &rest),
        "status" => cmd_status(&root),
        "active" => cmd_active(&root, &rest),
        "reach" => cmd_reach(&root, &rest),
        "catalogue" => catalogue::cmd_catalogue(&root, &rest),
        "impact" => catalogue::cmd_impact(&root, &rest),
        "gap" => cmd_gap(&root),
        "graph" => cmd_graph(&root),
        "ready" => cmd_ready(&root, &rest),
        "codeowners" => cmd_codeowners(&root),
        "bundle" => cmd_bundle(&root, &rest),
        "variables" => cmd_variables(&root),
        "setup" => cmd_setup(&root),
        "lesson" => cmd_lesson(&root, &rest),
        "readers" => readers::cmd_readers(&root, &rest),
        "derisk" => cmd_derisk(&root, &rest),
        "release" => cmd_release(&root, &rest),
        "kit" => cmd_kit(&root, &rest),
        "guides" => cmd_guides(&root),
        "ship" => flow::cmd_ship(&root, &rest),
        "method" => method::cmd_method(&root, &rest),
        "method-wasm" => method::cmd_method_wasm(&root, &rest),
        "files-wasm" => files::cmd_files_wasm(&root, &rest),
        "group-app" => group::cmd_group_app(&root, &rest),
        "group-export" => group::cmd_group_export(&root, &rest),
        "rerun" => method::cmd_rerun(&root, &rest),
        "explain" => pipeline::cmd_explain(&root, &rest),
        "why" => pipeline::cmd_why(&root, &rest),
        "sheet" => cmd_sheet(&root, &rest),
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

  docs               write docs/PSEUDOCODE.md, the method language's reference
                     page, from the tables the checker reads. A node's page is
                     rendered when it is opened, and is never written here.
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
  lesson form <node> [--out <file.html>] [--check]
                     a row's lesson form: one HTML file the node engineer who knows
                     the row fills anywhere, checked as they type by the
                     gate's own lesson check, saved as a filled copy.
                     --check (what --dry-run runs) writes nothing.
  lesson check <file> [--for <node>]
                     what a filled lesson form (or a bare lesson.toml, with
                     --for) holds, and every reason it would be refused.
                     Writes nothing.
  derisk             write docs/DERISK_NARRATIVE.md and docs/derisking.csv — every
                     recorded change, in the columns of the de-risking narrative,
                     and every registered risk as it stands. Generated from the
                     sheets' [[version]] and [[risk]] records, never edited.
  kit [--bin <dir>] [--out <dir>] [--files-only]
                     the tool as each person gets it: the two programs and
                     the files they read (the web face, the design's files,
                     design/, and the reference data) in one folder, with
                     START_HERE.md — on Windows the daemon is `Start VLEO.exe`,
                     elsewhere start.sh starts it. No git, no Rust source.
                     Zip the folder and share it. --bin is where the
                     release-built programs are (default target/release);
                     --files-only leaves the programs out, for the Python
                     package (tools/build_wheel.py).
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
  ship <version> [--no-push] [--no-test]
                     the release branch release/<version> from main: the
                     de-risking narrative, the version, regenerate, gate, test,
                     commit, push — and the tag commands for after the merge.
  release <version> [--check]
                     set the workspace version, and regenerate. It stamps
                     nothing in the design: a node's record is its node
                     engineer's. --check refuses while the workspace says
                     another version.
  variables          write docs/VARIABLES.md — every variable in the tree, its
                     unit, its range, the reason for each bound, and what reads
                     it. Generated, because a register maintained by hand is a
                     register that is wrong.
  explain [<command>]
                     where a command sits in a node's journey, what it reads,
                     writes and checks, how to undo it, its steps and where its
                     code is — from the one table docs/PIPELINE.md is written
                     from. With no command, the whole journey.
  sheet <node> ... | --all
                     each node's sheet as the design holds it, as JSON by id:
                     the text every reader of a sheet reads, for a tool
                     outside this program to read through the one reader.
  why <node>         a node's history in one place: every recorded version and
                     who made it, the commits that changed it, how its code
                     came to be, and its gate, run now.
  trace [<command>] [--list]
                     the last run's trace — every command that writes leaves
                     one in target/xtask-trace/, the newest 50 kept — or the
                     last of one command, or --list every one kept.
  pipeline [--check] write docs/PIPELINE.md from the pipeline table; --check
                     only says whether it is current.

Every command that writes also takes --dry-run: its check mode where it has
one, otherwise the plan — its steps, what it would write, how to undo it —
with nothing touched.";

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
        if p.join("Cargo.toml").is_file() && p.join("crates").is_dir() && p.join("design").is_dir()
        {
            return p;
        }
        if !p.pop() {
            panic!("not inside the VLEO repository");
        }
    }
}

/// The design as its files state it (`design/`), read by the one reader
/// (`vleo_files::convert::open`): what every command that reads the design
/// reads. A sheet left in a node folder is not read.
fn read(root: &Path) -> Result<Tree, String> {
    vleo_files::convert::open(root)
        .map(|(tree, _)| tree)
        .map_err(|e| e.to_string())
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

/// The method language's reference page, from the tables the checker reads,
/// so the page and the checker cannot describe two languages.
fn cmd_docs(root: &Path) -> Result<(), String> {
    let md = vleo_sheet::method::reference_md();
    let written = write_if_changed(&root.join("docs/PSEUDOCODE.md"), &md)?;
    println!(
        "docs: docs/PSEUDOCODE.md {}",
        if written { "written" } else { "unchanged" }
    );
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
        let t = page::fragment(sh, &tree);
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

/// The producing row behind a variable id.
///
/// A variable id is a node id or `<node id>.<extra>` — a node id never contains
/// a dot — so the producer is everything before the first one.
use vleo_sheet::text::producer_of;

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
