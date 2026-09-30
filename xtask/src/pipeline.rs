//! Scripts you can follow: what each command does, before and after it runs.
//!
//! One table says it — the `steps`, `writes` and `runs` of every `[[command]]`
//! in docs/manual.toml — and everything here reads that table: `explain` says
//! one command in full, `--dry-run` prints that and runs nothing, `why` names
//! the commands that write a file, and `pipeline` writes docs/PIPELINE.md.
//! Every run leaves one line in target/xtask-trace.log: when, what, how long,
//! and how it ended.

use std::io::Write;
use std::path::Path;
use vleo_sheet::manual;

fn the_manual(root: &Path) -> Result<manual::Manual, String> {
    manual::load(root).map_err(|e| format!("docs/manual.toml: {e}"))
}

/// `explain <command>`: what it is for, its steps, what it writes and starts.
pub fn cmd_explain(root: &Path, args: &[&str]) -> Result<(), String> {
    let m = the_manual(root)?;
    let Some(name) = args.first() else {
        for c in m.commands.iter().filter(|c| c.tool == "xtask") {
            println!("  {:<14} {:<12} {}", c.name, c.effect, c.what);
        }
        println!("\n`cargo xtask explain <command>` for one in full.");
        return Ok(());
    };
    let c = m
        .commands
        .iter()
        .find(|c| c.tool == "xtask" && c.name == *name)
        .ok_or_else(|| format!("no command '{name}'. `cargo xtask explain` lists them."))?;
    print!("{}", manual::explain(c));
    Ok(())
}

/// `--dry-run` on any command: print what it would do, and run nothing.
pub fn dry_run(root: &Path, cmd: &str) -> Result<(), String> {
    cmd_explain(root, &[cmd])?;
    println!("\n--dry-run: nothing was run and nothing was written.");
    Ok(())
}

/// `why <path>`: which commands write this file, and so whether to edit it.
pub fn cmd_why(root: &Path, args: &[&str]) -> Result<(), String> {
    let path = args.first().ok_or("usage: cargo xtask why <path>")?;
    let rel = Path::new(path)
        .canonicalize()
        .ok()
        .and_then(|p| {
            root.canonicalize()
                .ok()
                .and_then(|r| p.strip_prefix(r).ok().map(|x| x.to_path_buf()))
        })
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|| path.to_string());
    let m = the_manual(root)?;
    let by = manual::writers(&m, &rel);
    if by.is_empty() {
        println!("{rel}: no command writes it — it is a source, edited by a person.");
        return Ok(());
    }
    println!("{rel} is written by:");
    for c in &by {
        let pat = c
            .writes
            .iter()
            .find(|w| manual::path_matches(w, &rel))
            .map(String::as_str)
            .unwrap_or("");
        println!("  cargo xtask {:<14} ({pat}) — {}", c.name, c.what);
    }
    if !rel.ends_with("node.toml") && !rel.ends_with("fixtures.toml") && !rel.ends_with(".lock") {
        println!("Generated: change what it is generated from and run the command, not the file.");
    }
    Ok(())
}

/// `pipeline [--check]`: write docs/PIPELINE.md, or only say whether it is current.
pub fn cmd_pipeline(root: &Path, args: &[&str]) -> Result<(), String> {
    let text = manual::pipeline_md(&the_manual(root)?);
    let path = root.join("docs").join("PIPELINE.md");
    let now = std::fs::read_to_string(&path).unwrap_or_default();
    if args.contains(&"--check") {
        return if now == text {
            println!("docs/PIPELINE.md is current");
            Ok(())
        } else {
            Err(
                "docs/PIPELINE.md is not what docs/manual.toml says — run `cargo xtask pipeline`"
                    .into(),
            )
        };
    }
    if now != text {
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("wrote docs/PIPELINE.md");
    } else {
        println!("docs/PIPELINE.md was already current");
    }
    Ok(())
}

/// One line per run in target/xtask-trace.log. A trace that cannot be written
/// costs nothing: the command's own result is what matters.
pub fn trace(
    root: &Path,
    args: &[String],
    started: std::time::Instant,
    outcome: &Result<(), String>,
) {
    let dir = root.join("target");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("xtask-trace.log"))
    else {
        return;
    };
    let how = match outcome {
        Ok(()) => "ok".to_string(),
        Err(e) => format!("failed: {}", e.lines().next().unwrap_or("")),
    };
    let _ = writeln!(
        f,
        "{} {:>8.1}s xtask {} — {how}",
        vleo_data::clock::now_utc(),
        started.elapsed().as_secs_f64(),
        args.join(" ")
    );
}
