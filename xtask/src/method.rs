//! The method language, from the terminal: a node's method checked against its
//! node engineer's cases, and the checker every node form carries rebuilt.
//!
//! Both read `vleo_sheet::method`, the one implementation — the form runs the
//! same function compiled to WebAssembly, so what `xtask method` says about a
//! node is what the node engineer saw in their form before they sent it.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Where the checker lives, and the fingerprint of what it was built from.
pub const WASM: &str = "web/method.wasm.gz";
pub const STAMP: &str = "web/method.wasm.stamp";

/// `method <node>` — the node's method, checked, and each of its node engineer's
/// cases run through it.
pub fn cmd_method(root: &Path, args: &[&str]) -> Result<(), String> {
    let id = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .ok_or("usage: cargo run -p xtask -- method <node>")?;
    let tree = vleo_sheet::load_all(root)?;
    let sh = tree
        .sheets
        .get(*id)
        .ok_or_else(|| format!("no node '{id}'"))?;
    let text = fs::read_to_string(sh.dir.join("node.toml")).map_err(|e| e.to_string())?;
    let r = vleo_sheet::method::report_toml(&text)?;
    if sh.method.text.trim().is_empty() {
        println!("{id}: no method yet — the node keeps its hand-written holes until its owner sends one.");
        return Ok(());
    }
    println!(
        "{id}: the method (language {})",
        vleo_sheet::method::LANGUAGE_VERSION
    );
    for d in &r.diags {
        println!("  {d}");
    }
    for (i, (c, v)) in r.cases.iter().enumerate() {
        let mark = if v.agrees() { "ok  " } else { "FAIL" };
        let got = r.got.get(i).copied().flatten();
        let shown = match (c.expect, got) {
            (Some(e), Some(g)) => format!(" — your code {e:e}, the method {g:e}"),
            _ => String::new(),
        };
        println!("  {mark} {}: {}{shown}", c.label, v.text(c));
    }
    // A method copied from the code it replaced is held, as the gate holds
    // it, to what that code answered (baseline/transcribed.csv), not to
    // author cases it never had: cases made now would take their expected
    // values from the code under test (AGENTS.md, rule 4).
    let transcribed = !sh.method.transcribed_from.is_empty();
    if transcribed {
        println!(
            "  transcribed from {}: held to what that code answered, in baseline/transcribed.csv, \
             not to cases of its own; read against its source by {}",
            sh.method.transcribed_from,
            if sh.method.checked_by.is_empty() {
                "nobody yet"
            } else {
                sh.method.checked_by.as_str()
            }
        );
    } else {
        for s in &r.shortfall {
            println!("  missing: {s}");
        }
    }
    let sound = if transcribed {
        !r.diags
            .iter()
            .any(|d| d.severity == vleo_sheet::method::Severity::Error)
            && r.cases.iter().all(|(_, v)| v.agrees())
    } else {
        r.sound()
    };
    if sound && transcribed {
        println!(
            "sound: the method checks{}.",
            if r.cases.is_empty() {
                ", and carries no case of its own"
            } else {
                ", and agrees with every case it carries"
            }
        );
        Ok(())
    } else if sound {
        println!(
            "sound: the method checks and agrees with every one of its node engineer's cases."
        );
        Ok(())
    } else {
        Err(format!("{id}: the method is not sound yet — see above"))
    }
}

/// `method-wasm [--check]` — rebuild the checker the node form carries, or
/// only say whether it is current.
pub fn cmd_method_wasm(root: &Path, args: &[&str]) -> Result<(), String> {
    let now = vleo_sheet::method::checker_fingerprint(root)?;
    let stamp = fs::read_to_string(root.join(STAMP)).unwrap_or_default();
    let recorded = stamp
        .lines()
        .find_map(|l| l.strip_prefix("fingerprint = "))
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();
    if args.contains(&"--check") {
        if recorded == now {
            println!("method-wasm: {WASM} is built from the sources as they are ({now})");
            return Ok(());
        }
        return Err(format!(
            "{WASM} was built from different sources ({recorded:?}, now {now}). Every node form \
             carries it, so a form would check methods by the old rules. Run \
             `cargo run -p xtask -- method-wasm` and commit both files."
        ));
    }
    let manifest = root.join("crates/vleo-method-wasm/Cargo.toml");
    let mut cmd = Command::new("cargo");
    cmd.args([
        "build",
        "--release",
        "--target",
        "wasm32-unknown-unknown",
        "--manifest-path",
    ])
    .arg(&manifest)
    .current_dir(root);
    if root.join("crates/vleo-method-wasm/Cargo.lock").exists() {
        cmd.arg("--locked");
    }
    let ok = cmd.status().map_err(|e| e.to_string())?.success();
    if !ok {
        return Err(
            "the checker did not build — is the wasm32-unknown-unknown target installed? \
                    `rustup target add wasm32-unknown-unknown`"
                .into(),
        );
    }
    let built = root.join(
        "crates/vleo-method-wasm/target/wasm32-unknown-unknown/release/vleo_method_wasm.wasm",
    );
    // Gzipped, with no name or time in the header, so the same build gives the
    // same bytes: every form carries this, and the browser unpacks it.
    let gz = Command::new("gzip")
        .args(["-9", "-n", "-c"])
        .arg(&built)
        .output()
        .map_err(|e| format!("gzip could not be run: {e}"))?;
    if !gz.status.success() {
        return Err("gzip failed on the built checker".into());
    }
    fs::write(root.join(WASM), &gz.stdout).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(root.join("web/method.wasm"));
    fs::write(
        root.join(STAMP),
        format!(
            "# The node form's method checker: web/method.wasm.gz, built by\n\
             # `cargo run -p xtask -- method-wasm` from vleo_sheet::method::CHECKER_SOURCES.\n\
             # A test refuses a checkout whose sources have moved on from it.\n\
             fingerprint = \"{now}\"\nlanguage = {}\n",
            vleo_sheet::method::LANGUAGE_VERSION
        ),
    )
    .map_err(|e| e.to_string())?;
    let size = fs::metadata(root.join(WASM)).map(|m| m.len()).unwrap_or(0);
    println!(
        "method-wasm: {WASM} — {} KB, fingerprint {now}",
        size / 1024
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// rerun: the node engineer's own code, run again on their cases

/// What running the node engineer's code again said about one node.
enum Rerun {
    /// Every case gave what the sheet records.
    Agrees(usize),
    /// Some did not; each line says which and how.
    Differs(Vec<String>),
    /// It could not be run here, and why — a language with no free runner, or
    /// a runner that is not installed. Kept and read, not rerun.
    NotRun(String),
}

fn which(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run the node engineer's code on each of their cases and compare with what they
/// recorded. Python is called with each input as a keyword argument named by
/// its binding; MATLAB and Octave code with the inputs in the node's declared
/// order. A raised error or exception is a refusal.
fn rerun_one(sh: &vleo_sheet::model::Sheet, work: &Path) -> Result<Rerun, String> {
    let lang = sh.author.language.trim();
    let entry = sh.author.entry.trim();
    if sh.cases.is_empty() || sh.author.code.trim().is_empty() {
        return Ok(Rerun::NotRun("no cases or no code".into()));
    }
    if entry.is_empty() {
        return Ok(Rerun::NotRun("the entry function is not named".into()));
    }
    fs::create_dir_all(work).map_err(|e| e.to_string())?;
    let order: Vec<&str> = sh.inputs.iter().map(|i| i.binding.as_str()).collect();
    let value = |c: &vleo_sheet::method::Case, k: &str| {
        c.inputs
            .iter()
            .find(|(n, _)| n == k)
            .map(|(_, v)| *v)
            .unwrap_or(0.0)
    };
    let output = match lang {
        "Python" => {
            if !which("python3") {
                return Ok(Rerun::NotRun("python3 is not installed here".into()));
            }
            fs::write(work.join("author_code.py"), &sh.author.code).map_err(|e| e.to_string())?;
            let mut h = String::from(
                "import sys\nsys.path.insert(0, '.')\nimport author_code as m\nf = getattr(m, ",
            );
            h.push_str(&format!("{entry:?})\n"));
            for c in &sh.cases {
                let kw = order
                    .iter()
                    .map(|k| format!("{k}={:?}", value(c, k)))
                    .collect::<Vec<_>>()
                    .join(", ");
                h.push_str(&format!(
                    "try:\n    print(repr(float(f({kw}))))\nexcept Exception:\n    print('refused')\n"
                ));
            }
            fs::write(work.join("harness.py"), h).map_err(|e| e.to_string())?;
            Command::new("python3")
                .arg("harness.py")
                .current_dir(work)
                .output()
                .map_err(|e| e.to_string())?
        }
        "MATLAB" | "Octave" => {
            if !which("octave") {
                return Ok(Rerun::NotRun(format!(
                    "{lang} code is run with Octave, and Octave is not installed here"
                )));
            }
            fs::write(work.join(format!("{entry}.m")), &sh.author.code)
                .map_err(|e| e.to_string())?;
            let mut h = String::new();
            for c in &sh.cases {
                let args = order
                    .iter()
                    .map(|k| format!("{:?}", value(c, k)))
                    .collect::<Vec<_>>()
                    .join(", ");
                h.push_str(&format!(
                    "try\n  printf('%.17g\\n', {entry}({args}));\ncatch\n  printf('refused\\n');\nend\n"
                ));
            }
            fs::write(work.join("harness.m"), h).map_err(|e| e.to_string())?;
            Command::new("octave")
                .args(["--no-gui", "--quiet", "--eval", "harness"])
                .current_dir(work)
                .output()
                .map_err(|e| e.to_string())?
        }
        other => {
            return Ok(Rerun::NotRun(format!(
                "{} code is kept and read, not rerun — the pipeline runs Python, and MATLAB or \
                 Octave through Octave",
                if other.is_empty() { "unnamed" } else { other }
            )))
        }
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() != sh.cases.len() {
        return Ok(Rerun::Differs(vec![format!(
            "the code printed {} line(s) for {} case(s): {}",
            lines.len(),
            sh.cases.len(),
            String::from_utf8_lossy(&output.stderr)
                .lines()
                .last()
                .unwrap_or("")
        )]));
    }
    let mut bad = Vec::new();
    for (c, got) in sh.cases.iter().zip(lines) {
        match (c.expect, got) {
            (None, "refused") => {}
            (None, g) => bad.push(format!(
                "«{}»: recorded as refused, the code gives {g}",
                c.label
            )),
            (Some(e), "refused") => bad.push(format!(
                "«{}»: recorded as {e}, the code refuses it",
                c.label
            )),
            (Some(e), g) => {
                let v: f64 = g.parse().unwrap_or(f64::NAN);
                let rel = if e == 0.0 {
                    v.abs()
                } else {
                    ((v - e) / e).abs()
                };
                // NaN — output that was not a number — is a disagreement too.
                if rel.is_nan() || rel > c.tolerance {
                    bad.push(format!(
                        "«{}»: recorded as {e}, the code now gives {v} ({rel:.2e} apart)",
                        c.label
                    ));
                }
            }
        }
    }
    Ok(if bad.is_empty() {
        Rerun::Agrees(sh.cases.len())
    } else {
        Rerun::Differs(bad)
    })
}

/// `rerun <node>|--all [--require]` — run each node engineer's own code again on
/// their cases. `--require` refuses a node whose code could not be run here,
/// for a pipeline that has installed the runners and means to use them.
pub fn cmd_rerun(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = vleo_sheet::load_all(root)?;
    let all = args.contains(&"--all");
    let require = args.contains(&"--require");
    let mut not_run = 0usize;
    let only = args.iter().find(|a| !a.starts_with("--")).copied();
    if !all && only.is_none() {
        return Err("usage: cargo run -p xtask -- rerun <node>|--all [--require]".into());
    }
    let work_root = std::env::temp_dir().join("vleo-rerun");
    let mut differs = 0usize;
    let mut seen = 0usize;
    for sh in tree.ordered() {
        if only.is_some_and(|o| sh.id != o) || sh.cases.is_empty() {
            continue;
        }
        seen += 1;
        match rerun_one(sh, &work_root.join(&sh.id))? {
            Rerun::Agrees(n) => println!(
                "  ok   {}: the node engineer's code gives all {n} recorded case(s)",
                sh.id
            ),
            Rerun::NotRun(why) => {
                not_run += 1;
                println!("  note {}: not rerun — {why}", sh.id)
            }
            Rerun::Differs(lines) => {
                differs += 1;
                println!("  FAIL {}:", sh.id);
                for l in lines {
                    println!("         {l}");
                }
            }
        }
    }
    let _ = fs::remove_dir_all(&work_root);
    if seen == 0 {
        println!(
            "rerun: no node{} has its node engineer's cases yet",
            only.map(|o| format!(" '{o}'")).unwrap_or_default()
        );
    }
    if differs > 0 {
        return Err(format!(
            "{differs} node(s): the node engineer's code no longer gives the cases recorded from it. \
             The cases are the node engineer's evidence — take it to them."
        ));
    }
    if require && not_run > 0 {
        return Err(format!(
            "{not_run} node(s) could not be rerun here, and --require asks that every one is"
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// build-node: every stage, in order, stopping at the first that fails

/// `build-node <node>` — from the node's method to a node that may be
/// connected: translate, test against the node engineer's cases, rerun their code,
/// prove the tests test, and only then check the interface.
pub fn cmd_build_node(root: &Path, args: &[&str]) -> Result<(), String> {
    use crate::pipeline::{OnStop, Run};
    let id = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .copied()
        .ok_or("usage: cargo run -p xtask -- build-node <node>")?;
    let tree = vleo_sheet::load_all(root)?;
    let sh = tree
        .sheets
        .get(id)
        .ok_or_else(|| format!("no node '{id}'"))?;
    let mut run = Run::start(root, "build-node", args, 6);
    let again = format!("cargo run -p xtask -- build-node {id}");
    let built = "the kernel's translation and the node's generated files are written; the node \
                 is NOT connected — `git diff` shows them";

    run.step(
        "the method, against the node engineer's cases",
        OnStop::new(
            "unchanged — nothing was written",
            format!("fix the method on the node's form, then {again}"),
        ),
        || {
            if sh.method.text.trim().is_empty() {
                return Err(format!(
                    "{id} has no method. Its code is its hand-written holes, built and tested by \
                     `gate` and `cargo test` as before; build-node starts from a method."
                ));
            }
            cmd_method(root, &[id])?;
            Ok((
                (),
                format!(
                    "{} case(s) come out as the node engineer said",
                    sh.cases.len()
                ),
            ))
        },
    )?;

    let path = root
        .join("crates/vleo-core/src/physics/methods")
        .join(format!("{}.rs", sh.rust_ident()));
    run.step(
        "translate the method into the kernel, and regenerate the node",
        OnStop::new("part written — `git diff` shows what", &again),
        || {
            crate::cmd_docs(root, &[id])?;
            Ok((
                (),
                format!(
                    "{} — translated by rule",
                    path.strip_prefix(root).unwrap_or(&path).display()
                ),
            ))
        },
    )?;

    run.step(
        "the node's tests: the node engineer's cases, and the translation against the method",
        OnStop::new(
            built,
            format!(
                "a case that disagrees goes back to the node engineer; a translation test that fails is a \
                 translator defect for a developer. Then {again}"
            ),
        ),
        || {
            let ok = Command::new("cargo")
                .args(["test", "-q", "-p", &sh.crate_name, "--", &format!("{id}::")])
                .current_dir(root)
                .status()
                .map_err(|e| e.to_string())?
                .success();
            if !ok {
                return Err(format!("{id}: its tests fail"));
            }
            Ok(((), format!("{} — its tests pass", sh.crate_name)))
        },
    )?;

    run.step(
        "the node engineer's own code, run again on their cases",
        OnStop::new(
            built,
            format!("take the disagreement to the node engineer, then {again}"),
        ),
        || {
            cmd_rerun(root, &[id])?;
            Ok(((), String::new()))
        },
    )?;

    run.step(
        "the tests really test: the answer is moved and the tests must notice",
        OnStop::new(
            built,
            format!("add a case the moved answer fails, on the node's form; then {again}"),
        ),
        || {
            crate::cmd_mutate(root, &[id])?;
            Ok(((), String::new()))
        },
    )?;

    run.step(
        "only now, the interface: the node in the tree",
        OnStop::new(
            built,
            format!("`cargo run -p xtask -- gate {id}` says which check; then {again}"),
        ),
        || {
            crate::cmd_gate(root, &[id])?;
            let tree = vleo_sheet::load_all(root)?;
            let checks = vleo_sheet::gate::validate_tree(&tree);
            let failed: Vec<String> = checks
                .iter()
                .filter(|c| c.failed())
                .map(|c| format!("{} — {:?}", c.name, c.verdict))
                .collect();
            if !failed.is_empty() {
                return Err(format!(
                    "the tree does not assemble with {id}: {}",
                    failed.join("; ")
                ));
            }
            let readers: Vec<&str> = tree
                .ordered()
                .into_iter()
                .filter(|s| s.inputs.iter().any(|i| i.var == id))
                .map(|s| s.id.as_str())
                .collect();
            Ok((
                (),
                format!(
                    "the tree assembles. {} row(s) read {id}{}",
                    readers.len(),
                    if readers.is_empty() {
                        String::new()
                    } else {
                        format!(
                            ": {} — their answers move with it; `cargo test` checks them",
                            readers.join(", ")
                        )
                    }
                ),
            ))
        },
    )?;
    run.done(&format!(
        "build-node: {id} is built from its method, tested against its node engineer's cases, and connected."
    ));
    Ok(())
}

// ---------------------------------------------------------------------------
// migration: the tree moved over to methods, a batch at a time

/// `migration [--owner <o>] [--subsystem <s>] [--forms <dir>]` — which nodes
/// still need a method, by owner, and their forms written ready to send.
///
/// Nothing here writes a method. Each one comes from the node's owner, on the
/// node's form, like any other change: this only says whose they are and puts
/// the forms in one place.
pub fn cmd_migration(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = vleo_sheet::load_all(root)?;
    let flag = |f: &str| {
        args.iter()
            .position(|a| *a == f)
            .and_then(|i| args.get(i + 1))
            .copied()
    };
    let (owner, subsystem, forms) = (flag("--owner"), flag("--subsystem"), flag("--forms"));
    let mut by_owner: std::collections::BTreeMap<String, Vec<&vleo_sheet::model::Sheet>> =
        Default::default();
    let (mut asked, mut done) = (0usize, 0usize);
    for sh in tree.ordered() {
        if sh.is_seeded() || sh.is_declared() {
            continue;
        }
        if owner.is_some_and(|o| sh.owner != o) || subsystem.is_some_and(|s| sh.subsystem != s) {
            continue;
        }
        asked += 1;
        if !sh.method.text.trim().is_empty() {
            done += 1;
            continue;
        }
        by_owner.entry(sh.owner.clone()).or_default().push(sh);
    }
    println!(
        "migration: {done} of {asked} computed row(s) have a method; {} still to come.",
        asked - done
    );
    for (o, rows) in &by_owner {
        let names: Vec<&str> = rows.iter().take(6).map(|s| s.id.as_str()).collect();
        println!(
            "  {:<14} {:>4}   {}{}",
            if o.is_empty() { "(no owner)" } else { o },
            rows.len(),
            names.join(", "),
            if rows.len() > names.len() {
                ", …"
            } else {
                ""
            }
        );
    }
    if let Some(dir) = forms {
        let mut n = 0usize;
        for (o, rows) in &by_owner {
            let d = Path::new(dir).join(if o.is_empty() { "no-owner" } else { o });
            fs::create_dir_all(&d).map_err(|e| format!("{}: {e}", d.display()))?;
            for sh in rows {
                let html = vleo_sheet::template::document(sh, &tree);
                let p = d.join(format!("{}.node-form.html", sh.id));
                fs::write(&p, html).map_err(|e| format!("{}: {e}", p.display()))?;
                n += 1;
            }
        }
        println!(
            "\nwrote {n} form(s) under {dir}/<owner>/ — send each owner theirs. Each comes back \
             through `take`, one form per branch, like any other change."
        );
    }
    Ok(())
}
