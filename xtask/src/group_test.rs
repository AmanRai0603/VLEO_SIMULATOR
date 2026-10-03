//! A group's release, built and tested in the design.
//!
//! After `group-intake --apply` has taken a sealed release into the design, the
//! developer builds every computed node from its method and tests the group
//! against its own results — never against anything the code under test
//! produced:
//!
//! - `group-build` runs `build-node` on each computed node of the release:
//!   translated, tested on the author's cases, the tests proved to test, the
//!   interface checked. A node whose method in the design is not the
//!   release's is refused: the release is taken in first.
//! - `group-test` asks four things, and writes what it found to a report:
//!   1. the design holds the group's isolation results as the release has
//!      them, case for case;
//!   2. each node's own tests — its author's cases — pass;
//!   3. the group as a whole, through the engine, gives `results/group.csv`;
//!   4. at both ends of every range the group declares, each answer is a
//!      number or a refusal by name — never NaN, never a crash.

use super::group_intake::{cases, parse_csv, read_csv, Release};
use super::*;
use std::process::Command;

fn usage(cmd: &str) -> String {
    format!("usage: {cmd} <unpacked release folder> [--node <id>]")
}

fn flag<'a>(args: &[&'a str], f: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| *a == f)
        .and_then(|i| args.get(i + 1))
        .copied()
}

fn folder(args: &[&str], cmd: &str) -> Result<PathBuf, String> {
    // The folder is the one argument that is neither a flag nor a flag's value.
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
            continue;
        }
        if matches!(*a, "--node" | "--out" | "--bin") {
            skip = true;
            continue;
        }
        if !a.starts_with("--") {
            return Ok(PathBuf::from(a));
        }
    }
    Err(usage(cmd))
}

/// The release's computed nodes, by id, in the order nodes.csv lists them.
fn computed(dir: &Path) -> Result<Vec<String>, String> {
    Ok(read_csv(&dir.join("nodes.csv"))?
        .into_iter()
        .filter(|n| n.get("kind").map(String::as_str) == Some("computed"))
        .filter_map(|n| n.get("id").cloned())
        .collect())
}

// ---------------------------------------------------------------------------
// group-build

/// `group-build <folder> [--node <id>]` — every computed node of a sealed
/// release, built from its method by `build-node`.
pub(super) fn cmd_group_build(root: &Path, args: &[&str]) -> Result<(), String> {
    let dir = folder(args, "group-build")?;
    let only = flag(args, "--node");
    let rel = Release::open(&dir, false)?;
    let tree = load(root)?;
    let (mut built, mut failed, mut skipped) = (Vec::new(), Vec::new(), Vec::new());
    for id in computed(&dir)? {
        if only.is_some_and(|o| o != id) {
            continue;
        }
        let Some(sh) = tree.sheets.get(&id) else {
            skipped.push(format!(
                "{id} — not in the design; a new node arrives by its own form"
            ));
            continue;
        };
        let theirs = fs::read_to_string(dir.join("nodes").join(&id).join("pseudocode.txt"))
            .unwrap_or_default();
        if sh.method.text.trim().is_empty() || sh.method.text.trim() != theirs.trim() {
            failed.push(format!(
                "{id} — the design's method is not this release's: take it in first, \
                 `cargo run -p xtask -- group-intake {} --apply`",
                dir.display()
            ));
            continue;
        }
        println!("\n\x1b[1m━━ {id} ━━\x1b[0m");
        match method::cmd_build_node(root, &[id.as_str()]) {
            Ok(()) => built.push(id),
            Err(e) => failed.push(format!("{id} — {e}")),
        }
    }
    println!(
        "\n\x1b[1mgroup-build: {} {}\x1b[0m — {} built from its method, {} not",
        rel.group,
        rel.version,
        built.len(),
        failed.len()
    );
    for s in &skipped {
        println!("  skipped  {s}");
    }
    for f in &failed {
        println!("  \x1b[31mFAIL\x1b[0m     {f}");
    }
    if !failed.is_empty() {
        return Err(format!("{} node(s) not built", failed.len()));
    }
    if built.is_empty() {
        return Err("nothing was built: no computed node of this release is in the design".into());
    }
    println!("next: cargo run -p xtask -- group-test {}", dir.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// group-test

/// One line of the report.
struct Finding {
    stage: &'static str,
    subject: String,
    verdict: &'static str,
    detail: String,
}

/// What the engine said about one node, run once.
enum Ran {
    Answer(f64),
    /// Refused or blocked, by name: the reason.
    Refused(String),
    /// A crash, or a run that did not end in an answer or a refusal.
    Broke(String),
}

/// The engine, run on one node with some declared values set (SI).
fn run(vleo: &Path, root: &Path, id: &str, set: &[(String, f64)], tmp: &Path) -> Ran {
    let save = tmp.join(format!("{id}.csv"));
    let _ = fs::remove_file(&save);
    let mut c = Command::new(vleo);
    c.args(["run", id, "--defaults", "--again", "--save"])
        .arg(&save)
        .current_dir(root);
    for (k, v) in set {
        c.args(["--set", &format!("{k}={v:?}")]);
    }
    let out = match c.output() {
        Ok(o) => o,
        Err(e) => return Ran::Broke(format!("the engine did not start: {e}")),
    };
    let said = |b: &[u8]| {
        String::from_utf8_lossy(b)
            .lines()
            .map(|l| l.replace("\x1b[31m", "").replace("\x1b[0m", ""))
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
    };
    if !out.status.success() {
        let err = said(&out.stderr).join(" ");
        // 101 is a Rust panic: a crash, never a refusal.
        return if out.status.code() == Some(101) || err.is_empty() {
            Ran::Broke(format!("exit {:?}: {err}", out.status.code()))
        } else {
            Ran::Refused(err)
        };
    }
    let text = fs::read_to_string(&save).unwrap_or_default();
    let body: String = text
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| format!("{l}\n"))
        .collect();
    let (head, rows) = parse_csv(&body);
    let col = |n: &str| head.iter().position(|h| h == n);
    let (Some(sec), Some(idc), Some(si)) = (col("section"), col("id"), col("si")) else {
        return Ran::Broke("the saved result has no section, id and si columns".into());
    };
    match rows.iter().find(|r| {
        r.get(sec).map(String::as_str) == Some("output")
            && r.get(idc).map(String::as_str) == Some(id)
    }) {
        Some(r) => match r.get(si).and_then(|v| v.trim().parse::<f64>().ok()) {
            Some(x) if x.is_finite() => Ran::Answer(x),
            Some(x) => Ran::Broke(format!("the answer is {x}")),
            None => Ran::Broke(format!(
                "the answer is «{}»",
                r.get(si).cloned().unwrap_or_default()
            )),
        },
        None => {
            let blocked: Vec<String> = said(&out.stdout)
                .into_iter()
                .skip_while(|l| !l.trim_start().starts_with("blocked:"))
                .skip(1)
                .take_while(|l| l.starts_with("    "))
                .map(|l| l.trim().to_string())
                .collect();
            Ran::Refused(if blocked.is_empty() {
                "it did not answer".into()
            } else {
                blocked.join("; ")
            })
        }
    }
}

/// `{ a = 1.0, b = 2.0 }`, the shape a form carries a case's inputs in.
fn pairs(s: &str) -> Vec<(String, f64)> {
    s.trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .split(',')
        .filter_map(|p| {
            let (k, v) = p.split_once('=')?;
            let v = match v.trim() {
                "nan" => f64::NAN,
                "inf" => f64::INFINITY,
                "-inf" => f64::NEG_INFINITY,
                t => t.parse().ok()?,
            };
            Some((k.trim().to_string(), v))
        })
        .collect()
}

fn same(a: f64, b: f64) -> bool {
    a == b || (a.is_nan() && b.is_nan()) || (a - b).abs() <= 1e-12 * a.abs().max(b.abs())
}

/// A value in a group's unit, in SI.
fn si(value: &str, unit: &str) -> Option<f64> {
    let x: f64 = value.trim().parse().ok()?;
    let f = if unit.trim().is_empty() {
        1.0
    } else {
        vleo_sheet::method::parse_unit(unit).ok()?.0
    };
    Some(x * f)
}

/// `group-test <folder> [--out <dir>]` — the group, tested against its own
/// results, with a report written.
pub(super) fn cmd_group_test(root: &Path, args: &[&str]) -> Result<(), String> {
    let dir = folder(args, "group-test")?;
    let rel = Release::open(&dir, false)?;
    let tree = load(root)?;
    let out = flag(args, "--out").map(PathBuf::from).unwrap_or_else(|| {
        root.join("target/group")
            .join(format!("{}-{}", rel.group, rel.version))
    });
    fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let mut found: Vec<Finding> = Vec::new();
    let mut note = |stage, subject: String, verdict, detail: String| {
        let mark = match verdict {
            "ok" => "\x1b[32mok\x1b[0m  ",
            "FAIL" => "\x1b[31mFAIL\x1b[0m",
            _ => "note",
        };
        println!(
            "  {mark}  {subject}{}",
            if detail.is_empty() {
                String::new()
            } else {
                format!(" — {detail}")
            }
        );
        found.push(Finding {
            stage,
            subject,
            verdict,
            detail,
        });
    };
    let ids: Vec<String> = computed(&dir)?
        .into_iter()
        .filter(|id| tree.sheets.contains_key(id))
        .collect();
    if ids.is_empty() {
        return Err("no computed node of this release is in the design: take it in first with group-intake --apply".into());
    }

    // 1 · the design holds the group's isolation results, case for case
    println!("\n\x1b[1m1 · the design holds the group's results\x1b[0m");
    for id in &ids {
        let sh = &tree.sheets[id];
        let want =
            cases(&dir.join("nodes").join(id).join("results/isolation.csv")).unwrap_or_default();
        let have = &sh.cases;
        let mut differs = Vec::new();
        if want.len() != have.len() {
            differs.push(format!(
                "{} case(s) in the release, {} in the design",
                want.len(),
                have.len()
            ));
        } else {
            for (k, (w, h)) in want.iter().zip(have.iter()).enumerate() {
                let refuse = w.get("refuse").map(String::as_str) == Some("yes");
                let expect = w.get("expect").and_then(|e| e.parse::<f64>().ok());
                let inputs = pairs(w.get("inputs").map(String::as_str).unwrap_or(""));
                let same_inputs = inputs.len() == h.inputs.len()
                    && inputs
                        .iter()
                        .all(|(n, v)| h.inputs.iter().any(|(m, u)| m == n && same(*u, *v)));
                let same_answer = match (refuse, expect, h.expect) {
                    (true, _, None) => true,
                    (false, Some(a), Some(b)) => same(a, b),
                    _ => false,
                };
                if !same_inputs || !same_answer {
                    differs.push(format!("case {} is not the release's", k + 1));
                }
            }
        }
        if differs.is_empty() {
            note(
                "design",
                id.clone(),
                "ok",
                format!("{} case(s), as sealed", have.len()),
            );
        } else {
            note(
                "design",
                id.clone(),
                "FAIL",
                format!("{} — take the release in again", differs.join("; ")),
            );
        }
    }

    // 2 · each node's own tests: its author's cases, and the translation
    println!("\n\x1b[1m2 · each node on its own: its author's cases\x1b[0m");
    for id in &ids {
        let sh = &tree.sheets[id];
        let ok = Command::new("cargo")
            .args(["test", "-q", "-p", &sh.crate_name, "--", &format!("{id}::")])
            .current_dir(root)
            .output()
            .map_err(|e| e.to_string())?;
        if ok.status.success() {
            note(
                "isolation",
                id.clone(),
                "ok",
                format!("its tests pass in {}", sh.crate_name),
            );
        } else {
            note(
                "isolation",
                id.clone(),
                "FAIL",
                format!("`cargo test -p {} -- {id}::` fails", sh.crate_name),
            );
        }
    }

    // The engine, built once, for the group as a whole and its edges.
    let built = Command::new("cargo")
        .args(["build", "-q", "-p", "vleo-cli", "--bin", "vleo"])
        .current_dir(root)
        .status()
        .map_err(|e| e.to_string())?;
    if !built.success() {
        return Err("the engine (vleo-cli) does not build".into());
    }
    let target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root.join("target"));
    let vleo = target
        .join("debug")
        .join(if cfg!(windows) { "vleo.exe" } else { "vleo" });
    let tmp = out.join("runs");
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

    // 3 · the group as a whole: results/group.csv, through the engine
    println!("\n\x1b[1m3 · the group as a whole, through the engine\x1b[0m");
    let gpath = dir.join("results/group.csv");
    let mut answers: Vec<String> = Vec::new();
    match fs::read_to_string(&gpath) {
        Err(_) => note(
            "group",
            "results/group.csv".into(),
            "note",
            "the release has no group results: only its nodes were tested on their own".into(),
        ),
        Ok(text) => {
            let (head, rows) = parse_csv(&text);
            let split = |h: &str| match h.split_once('[') {
                Some((n, u)) => (
                    n.trim().to_string(),
                    u.trim_end_matches(']').trim().to_string(),
                ),
                None => (h.trim().to_string(), String::new()),
            };
            let cols: Vec<(String, String)> = head.iter().map(|h| split(h)).collect();
            let other = ["tolerance", "refuses", "origin", "says", "note", "label"];
            let at = |n: &str| cols.iter().position(|(c, _)| c == n);
            for (c, _) in &cols {
                if let Some(a) = c.strip_prefix("answer.") {
                    answers.push(a.to_string());
                }
            }
            for (k, r) in rows.iter().enumerate() {
                let mut set = Vec::new();
                for (i, (c, u)) in cols.iter().enumerate() {
                    if c.starts_with("answer") || other.contains(&c.as_str()) {
                        continue;
                    }
                    match r.get(i).and_then(|v| si(v, u)) {
                        Some(x) => set.push((c.clone(), x)),
                        None => note(
                            "group",
                            format!("row {}", k + 1),
                            "FAIL",
                            format!("{c} is not a number in [{u}]"),
                        ),
                    }
                }
                let refuses =
                    at("refuses").and_then(|i| r.get(i)).map(String::as_str) == Some("yes");
                let tol: f64 = at("tolerance")
                    .and_then(|i| r.get(i))
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(0.0);
                let given = set
                    .iter()
                    .map(|(c, x)| format!("{c} = {x:?}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                for (i, (c, u)) in cols.iter().enumerate() {
                    let Some(a) = c.strip_prefix("answer.") else {
                        continue;
                    };
                    let subject = format!("row {} · {a}", k + 1);
                    if !tree.sheets.contains_key(a) {
                        note(
                            "group",
                            subject,
                            "note",
                            format!("{a} is not in the design yet"),
                        );
                        continue;
                    }
                    let ran = run(&vleo, root, a, &set, &tmp);
                    match (refuses, ran) {
                        (true, Ran::Refused(why)) => note(
                            "group",
                            subject,
                            "ok",
                            format!("refused at {given}, as the group said: {why}"),
                        ),
                        (true, Ran::Answer(x)) => note(
                            "group",
                            subject,
                            "FAIL",
                            format!("answered {x:?} at {given}; the group says it refuses"),
                        ),
                        (_, Ran::Broke(why)) => {
                            note("group", subject, "FAIL", format!("at {given}: {why}"))
                        }
                        (false, Ran::Refused(why)) => note(
                            "group",
                            subject,
                            "FAIL",
                            format!("refused at {given}: {why}"),
                        ),
                        (false, Ran::Answer(x)) => {
                            let Some(e) = r.get(i).and_then(|v| si(v, u)) else {
                                note(
                                    "group",
                                    subject,
                                    "FAIL",
                                    format!("the expected {a} is not a number in [{u}]"),
                                );
                                continue;
                            };
                            let err = if e == 0.0 {
                                x.abs()
                            } else {
                                (x - e).abs() / e.abs()
                            };
                            if err <= tol {
                                note(
                                    "group",
                                    subject,
                                    "ok",
                                    format!("{x:?} at {given}, within {tol:e}"),
                                );
                            } else {
                                note("group", subject, "FAIL", format!("{x:?} at {given}; the group's {e:?} — off by {err:.3e}, tolerance {tol:e}"));
                            }
                        }
                    }
                }
            }
        }
    }

    // 4 · the edges: every declared range, at both ends, answers or refuses
    println!("\n\x1b[1m4 · at both ends of every range the group declares\x1b[0m");
    if answers.is_empty() {
        answers = ids.clone();
    }
    answers.retain(|a| tree.sheets.contains_key(a));
    for n in read_csv(&dir.join("nodes.csv"))? {
        let g = |k: &str| n.get(k).cloned().unwrap_or_default();
        if g("kind") != "declared" || !tree.sheets.contains_key(&g("id")) {
            continue;
        }
        for (end, v) in [("lower", g("lower")), ("upper", g("upper"))] {
            let Some(x) = si(&v, &g("unit")) else {
                continue;
            };
            for a in &answers {
                let subject = format!("{} at its {end} end, {v} {} · {a}", g("id"), g("unit"));
                match run(&vleo, root, a, &[(g("id"), x)], &tmp) {
                    Ran::Answer(y) => note("edges", subject, "ok", format!("{y:?}")),
                    Ran::Refused(why) => {
                        note("edges", subject, "note", format!("refused by name: {why}"))
                    }
                    Ran::Broke(why) => note("edges", subject, "FAIL", why),
                }
            }
        }
    }

    // The report, beside the runs it rests on.
    let esc = |s: &str| {
        if s.contains([',', '"', '\n']) {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    };
    let mut csv = String::from("stage,subject,verdict,detail\n");
    for f in &found {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            f.stage,
            esc(&f.subject),
            f.verdict,
            esc(&f.detail)
        ));
    }
    let report = out.join("group-test.csv");
    fs::write(&report, csv).map_err(|e| format!("{}: {e}", report.display()))?;
    let fails = found.iter().filter(|f| f.verdict == "FAIL").count();
    let oks = found.iter().filter(|f| f.verdict == "ok").count();
    println!(
        "\n\x1b[1mgroup-test: {} {}\x1b[0m — {oks} held, {fails} did not, {} noted. Report: {}",
        rel.group,
        rel.version,
        found.len() - oks - fails,
        report.strip_prefix(root).unwrap_or(&report).display()
    );
    if fails > 0 {
        return Err(format!(
            "{fails} check(s) did not hold. A disagreement with the group's results goes back to the group with the report; \
             the results are their evidence, and never the thing changed to make a test pass"
        ));
    }
    println!(
        "next: cargo run -p xtask -- group-deliver {}",
        dir.display()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// group-deliver

/// `group-deliver <folder> [--out <dir>] [--bin <dir>] [--uncommitted]` — the
/// test application for the group: the tool, built from this checkout with
/// their release in it, and a delivery record saying exactly what it holds.
///
/// Refused until `group-test` has passed on this release, and refused from a
/// checkout with uncommitted changes unless `--uncommitted` says it is a
/// throwaway: a delivery is a build someone can make again from its commit.
pub(super) fn cmd_group_deliver(root: &Path, args: &[&str]) -> Result<(), String> {
    let dir = folder(args, "group-deliver")?;
    let rel = Release::open(&dir, false)?;
    let tree = load(root)?;
    let key = format!("{}-{}", rel.group, rel.version);
    let tested = root.join("target/group").join(&key).join("group-test.csv");
    let report = fs::read_to_string(&tested).map_err(|_| {
        format!(
            "{key} has not been tested here: `cargo run -p xtask -- group-test {}` first",
            dir.display()
        )
    })?;
    let (head, rows) = parse_csv(&report);
    let vcol = head.iter().position(|h| h == "verdict").unwrap_or(2);
    let fails = rows
        .iter()
        .filter(|r| r.get(vcol).map(String::as_str) == Some("FAIL"))
        .count();
    let held = rows
        .iter()
        .filter(|r| r.get(vcol).map(String::as_str) == Some("ok"))
        .count();
    if fails > 0 {
        return Err(format!(
            "{key}'s group test did not pass ({fails} check(s) failed, {}): nothing is delivered until it does",
            tested.display()
        ));
    }
    let git = |a: &[&str]| {
        Command::new("git")
            .args(a)
            .current_dir(root)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    };
    let commit = git(&["rev-parse", "HEAD"]);
    let dirty = !git(&["status", "--porcelain"]).is_empty();
    if dirty && !args.contains(&"--uncommitted") {
        return Err(
            "the checkout has uncommitted changes: commit the release's intake first, so the \
             delivery can be built again from its commit (or pass --uncommitted on a throwaway)"
                .into(),
        );
    }
    // The nodes this delivery built from the group's methods, from the design itself.
    let built: Vec<String> = computed(&dir)?
        .into_iter()
        .filter(|id| {
            tree.sheets.get(id).is_some_and(|sh| {
                let theirs = fs::read_to_string(dir.join("nodes").join(id).join("pseudocode.txt"))
                    .unwrap_or_default();
                !sh.method.text.trim().is_empty() && sh.method.text.trim() == theirs.trim()
            })
        })
        .collect();

    let version = release::workspace_version(root)?;
    let out = flag(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("dist").join(format!("vleo-{version}-{key}-test")));
    let bin = match flag(args, "--bin") {
        Some(b) => PathBuf::from(b),
        None => {
            println!("building the programs (release profile) …");
            let ok = Command::new("cargo")
                .args([
                    "build",
                    "--release",
                    "-q",
                    "-p",
                    "vleo-daemon",
                    "-p",
                    "vleo-cli",
                ])
                .current_dir(root)
                .status()
                .map_err(|e| e.to_string())?
                .success();
            if !ok {
                return Err("the programs do not build".into());
            }
            std::env::var("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| root.join("target"))
                .join("release")
        }
    };
    let out_s = out.to_string_lossy().to_string();
    let bin_s = bin.to_string_lossy().to_string();
    release::cmd_kit(root, &["--out", &out_s, "--bin", &bin_s])?;

    let fingerprint = fs::read_to_string(dir.join("RELEASE.toml"))
        .ok()
        .and_then(|t| t.parse::<toml::Value>().ok())
        .and_then(|v| {
            v.get("fingerprint")
                .and_then(|f| f.as_str())
                .map(str::to_string)
        })
        .unwrap_or_default();
    let today = git(&["log", "-1", "--format=%cs"]);
    let list = |v: &[String]| {
        v.iter()
            .map(|s| format!("{s:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let record = format!(
        "# What this test application holds — written by `xtask group-deliver`.\n\
         # The group checks it, then tells the developer: accepted, or what to change.\n\
         group = {:?}\nversion = {:?}\nsealed = {:?}\nfingerprint = {:?}\n\
         tool = {version:?}\ncommit = {commit:?}\nuncommitted = {dirty}\n\
         built = [{}]\nchecks_held = {held}\nchecks_failed = 0\n",
        rel.group,
        rel.version,
        rel.sealed,
        fingerprint,
        list(&built),
    );
    fs::write(out.join("DELIVERY.toml"), record).map_err(|e| e.to_string())?;
    fs::copy(&tested, out.join("group-test.csv")).map_err(|e| e.to_string())?;
    let nodes = if built.is_empty() {
        "(none — this release changed no method)".to_string()
    } else {
        built
            .iter()
            .map(|b| {
                format!("- `{b}` — open it, and read the answer, its derivation and its cases")
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let notes = format!(
        "# Test application — {group} {v}\n\n\
         This is the tool, built with your release **{group} {v}** in it, for your group to try \
         before anything reaches everyone. It changes nothing on your drive.\n\n\
         ## What is in it\n\n\
         Built from your pseudocode, and tested against your own results:\n\n{nodes}\n\n\
         `group-test.csv` is every check the developer ran: {held} held, none failed. \
         `DELIVERY.toml` says exactly which release, which seal and which commit this is.\n\n\
         ## What to try\n\n\
         1. Start it (`start.sh`, or `Start VLEO.exe` on Windows) and open each node above.\n\
         2. Run it at the values you know the answers to — your own results, your own \
         spreadsheet. Every number must be one you can account for.\n\
         3. Read each node's page as someone who has never seen it: the words, the \
         derivation, the pictures.\n\n\
         ## Then\n\n\
         Tell the developer **accepted**, or **changes** with what you saw. A change goes back \
         into your group folder and a new sealed release; nothing in this folder is edited.\n\n\
         Built {today} from commit `{short}`{dirty_note}.\n",
        group = rel.group,
        v = rel.version,
        short = &commit[..commit.len().min(10)],
        dirty_note = if dirty {
            " with uncommitted changes — a throwaway build, not one to accept"
        } else {
            ""
        },
    );
    fs::write(out.join("DELIVERY.md"), notes).map_err(|e| e.to_string())?;
    println!(
        "\ngroup-deliver: {} — the test application for {key}: {} node(s) built from the group's methods, \
         {held} check(s) held. Zip the folder and send it to the group; they answer accepted, or changes.",
        out.display(),
        built.len()
    );
    Ok(())
}
