//! The method language, from the terminal: a node's method checked against its
//! node engineer's cases, and the checker the pages carry rebuilt.
//!
//! Both read `vleo_sheet::method`, the one implementation — the pages run the
//! same function compiled to WebAssembly, so what `xtask method` says about a
//! node is what the node engineer saw before they sent it.

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
    let tree = crate::read(root)?;
    let sh = tree
        .sheets
        .get(*id)
        .ok_or_else(|| format!("no node '{id}'"))?;
    let text = &sh.text;
    let r = vleo_sheet::method::report_toml(text)?;
    if sh.method.text.trim().is_empty() {
        println!("{id}: no method yet — the node answers nothing until its owner sends one.");
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

/// `method-wasm [--check]` — rebuild the checker the pages carry, or
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
            "{WASM} was built from different sources ({recorded:?}, now {now}). Every page that \
             checks a method carries it, so a page would check methods by the old rules. Run \
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
            "# The pages' method checker: web/method.wasm.gz, built by\n\
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

/// `rerun <node>|--all [--require]` — run each node engineer's own code again on
/// their cases. `--require` refuses a node whose code could not be run here,
/// for a pipeline that has installed the runners and means to use them.
pub fn cmd_rerun(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = crate::read(root)?;
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
        match vleo_sheet::rerun::rerun(sh, &work_root.join(&sh.id))? {
            vleo_sheet::rerun::Rerun::Agrees(n) => println!(
                "  ok   {}: the node engineer's code gives all {n} recorded case(s)",
                sh.id
            ),
            vleo_sheet::rerun::Rerun::NotRun(why) => {
                not_run += 1;
                println!("  note {}: not rerun — {why}", sh.id)
            }
            vleo_sheet::rerun::Rerun::Differs(lines) => {
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
