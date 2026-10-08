//! The release.
//!
//!     ship <version>                    a release branch, stamped and proved
//!
//! THE DEVELOPER'S JOB IS THE SAME EVERY TIME, SO IT IS COMMANDS. A person
//! who does a step by hand forgets part of it on the day they are busy — the
//! branch made from a stale main, the tests not run — and the result looks
//! exactly like a careful one. Each command here does its whole step or
//! nothing, and says what to do next.

use crate::pipeline::{OnStop, Run};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Run git in the repository and return what it printed, or what it refused.
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// `owner/repo` from the origin remote, whatever form its URL takes.
fn repo_slug(root: &Path) -> Option<String> {
    let url = git(root, &["remote", "get-url", "origin"]).ok()?;
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    let parts: Vec<&str> = url.split(['/', ':']).filter(|p| !p.is_empty()).collect();
    (parts.len() >= 2).then(|| format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1]))
}

fn current_branch(root: &Path) -> Result<String, String> {
    git(root, &["rev-parse", "--abbrev-ref", "HEAD"])
}

fn head(root: &Path) -> Result<String, String> {
    git(root, &["rev-parse", "HEAD"])
}

/// Refuse to start on a working tree with uncommitted changes to tracked files:
/// they would be swept into a commit that names somebody else.
fn clean(root: &Path) -> Result<(), String> {
    let dirty = git(root, &["status", "--porcelain", "--untracked-files=no"])?;
    if dirty.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "the working tree has uncommitted changes — commit or stash them first, \
             so they are not swept into this commit:\n{dirty}"
        ))
    }
}

/// One of this program's steps, run as the pipeline runs every step: numbered,
/// and on a stop saying what state the branch is in and how to get out.
fn step<F: FnOnce() -> Result<(), String>>(
    run: &mut Run,
    what: &str,
    stop: OnStop,
    f: F,
) -> Result<(), String> {
    run.step(what, stop, || f().map(|()| ((), String::new())))
}

/// Today's answers, recorded again after a deliberate change to the design.
///
/// `baseline/today.csv` holds what the engine answers, and `cargo test` fails
/// when an answer moves. A node built from its method moves answers on
/// purpose, so the record is written again here, before the tests,
/// and the difference goes into the same commit as the change, where it is
/// reviewed with it (`baseline/README.md`). What it says is how many lines
/// moved, so the person reading the steps sees it too.
pub(crate) fn record_today(root: &Path) -> Result<String, String> {
    let ok = Command::new("cargo")
        .args([
            "test",
            "-q",
            "-p",
            "vleo-cli",
            "--test",
            "today_s_answers_are_on_record",
        ])
        .env("VLEO_BASELINE", "write")
        .current_dir(root)
        .status()
        .map_err(|e| e.to_string())?
        .success();
    if !ok {
        return Err(
            "today's answers could not be recorded: the engine did not build or run".into(),
        );
    }
    let moved = git(root, &["diff", "--numstat", "--", "baseline/today.csv"]).unwrap_or_default();
    let mut n = moved
        .split_whitespace()
        .map(|n| n.parse::<usize>().unwrap_or(0));
    let (added, removed) = (n.next().unwrap_or(0), n.next().unwrap_or(0));
    Ok(if added + removed == 0 {
        "no answer moved".to_string()
    } else {
        format!("{added} line(s) of baseline/today.csv written, {removed} replaced: review them with the change")
    })
}

/// A commit message that passes tools/commit_message.py: `type(scope):
/// subject` within 72 characters, a blank line, a body wrapped at 72.
pub fn commit_message(kind: &str, scope: &str, subject: &str, body: &[String]) -> String {
    let mut head = format!("{kind}({scope}): {subject}");
    if head.chars().count() > 72 {
        head = head
            .chars()
            .take(71)
            .collect::<String>()
            .trim_end()
            .to_string();
    }
    let mut out = head;
    out.push_str("\n\n");
    for para in body {
        let mut line = String::new();
        for w in para.split_whitespace() {
            if !line.is_empty() && line.chars().count() + 1 + w.chars().count() > 72 {
                out.push_str(&line);
                out.push('\n');
                line.clear();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(w);
        }
        out.push_str(&line);
        out.push_str("\n\n");
    }
    out.trim_end().to_string() + "\n"
}

fn commit(root: &Path, message: &str) -> Result<(), String> {
    let file = std::env::temp_dir().join(format!("vleo-commit-{}.txt", std::process::id()));
    fs::write(&file, message).map_err(|e| e.to_string())?;
    let r = git(root, &["commit", "-q", "-F", &file.to_string_lossy()]);
    let _ = fs::remove_file(&file);
    r.map(|_| ())
}

fn push(root: &Path, branch: &str, args: &[&str]) -> Result<(), String> {
    if args.contains(&"--no-push") {
        println!("  not pushed (--no-push). When ready: git push -u origin {branch}");
        return Ok(());
    }
    git(root, &["push", "-q", "-u", "origin", branch]).map(|_| {
        println!("  pushed {branch}");
    })
}

// ---------------------------------------------------------------------------
// ship

/// Bring every lockfile level with the workspace version. `cargo metadata`
/// rewrites only the entries that changed — the workspace's own crates — and
/// upgrades nothing else.
fn refresh_locks(root: &Path) -> Result<(), String> {
    for manifest in [
        "Cargo.toml",
        "crates/vleo-py/Cargo.toml",
        "crates/vleo-wasm/Cargo.toml",
        "crates/vleo-kernel-wasm/Cargo.toml",
    ] {
        let ok = Command::new("cargo")
            .args([
                "metadata",
                "-q",
                "--format-version",
                "1",
                "--manifest-path",
                manifest,
            ])
            .current_dir(root)
            .stdout(std::process::Stdio::null())
            .status()
            .map_err(|e| e.to_string())?
            .success();
        if !ok {
            return Err(format!(
                "the lockfile beside {manifest} could not be brought level"
            ));
        }
    }
    Ok(())
}

/// `ship <version> [--no-push] [--no-test]` — the release branch.
pub fn cmd_ship(root: &Path, args: &[&str]) -> Result<(), String> {
    let version = args.iter().find(|a| !a.starts_with("--")).copied().ok_or(
        "usage: cargo run -p xtask -- ship <version> [--no-push] [--no-test] — e.g. ship 0.3.0",
    )?;
    clean(root)?;
    if current_branch(root)? != "main" {
        return Err("a release is cut from main: git switch main && git pull origin main".into());
    }
    if git(root, &["fetch", "-q", "origin", "main"]).is_ok() {
        let local = head(root)?;
        let remote = git(root, &["rev-parse", "origin/main"])?;
        if local != remote {
            return Err("this main is not origin's main — git pull origin main first".into());
        }
    }
    let branch = format!("release/{version}");
    let mut run = Run::start(root, "ship", args, 7);
    let half = || {
        OnStop::new(
            format!("{branch} holds a half-done release, uncommitted"),
            format!(
                "abandon it: git reset -q --hard && git switch main && git branch -D {branch}; \
                 then fix and `cargo run -p xtask -- ship {version}`"
            ),
        )
    };
    step(
        &mut run,
        "the branch",
        OnStop::new(
            "unchanged — nothing was written",
            format!("if {branch} exists already: git branch -D {branch}, then ship again"),
        ),
        || git(root, &["switch", "-q", "-c", &branch]).map(|_| ()),
    )?;
    println!("      {branch}");
    step(&mut run, "the de-risking narrative", half(), || {
        crate::cmd_derisk(root, &[])
    })?;
    step(&mut run, "stamp the release", half(), || {
        crate::cmd_release(root, &[version])
    })?;
    // The version is written into the role guides and into three lockfiles
    // (the workspace's and the two faces built outside it). A stamp that
    // leaves any of them behind fails the regeneration diff, or the release's
    // `--locked` build of the Python engine.
    step(&mut run, "regenerate", half(), || {
        crate::cmd_variables(root)?;
        crate::cmd_codeowners(root)?;
        crate::cmd_guides(root)?;
        refresh_locks(root)
    })?;
    step(&mut run, "gate", half(), || crate::cmd_gate(root, &[]))?;
    if args.contains(&"--no-test") {
        run.skip("tests (cargo test --workspace)", "--no-test");
    } else {
        step(&mut run, "tests (cargo test --workspace)", half(), || {
            let ok = Command::new("cargo")
                .args(["test", "--workspace", "-q"])
                .current_dir(root)
                .status()
                .map_err(|e| e.to_string())?
                .success();
            if ok {
                Ok(())
            } else {
                Err("tests failed — nothing is committed".into())
            }
        })?;
    }
    let message = commit_message(
        "chore",
        "tree",
        &format!("release {version}"),
        &[format!("`xtask ship {version}`: every node version still `next` is stamped with {version}, and the workspace and every lock file move to it.")],
    );
    step(&mut run, "commit and push", half(), || {
        git(root, &["add", "-A"])?;
        commit(root, &message)?;
        push(root, &branch, args)
    })?;
    run.done(&format!("ship: {branch} is committed"));
    let repo = repo_slug(root).unwrap_or_else(|| "<owner>/<repo>".into());
    println!(
        "\n\x1b[1mready to release {version}.\x1b[0m\n\n\
         1 · open the pull request and merge it once it is green:\n      \
         https://github.com/{repo}/compare/main...{branch}?expand=1\n  \
         2 · then tag the merged main:\n      \
         git switch main && git pull origin main && git tag v{version} && git push origin v{version}\n  \
         3 · Actions → release builds the four files; publish them as the v{version} release\n      \
         (docs/roles/maintainer.html, \"Release a version\")"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_commit_message_keeps_to_the_repositorys_rule() {
        let m = commit_message(
            "feat",
            "mod-payload",
            "a subject long enough that it would run past the seventy-two character limit",
            &["a body paragraph ".repeat(12), "Form-by: Ana".into()],
        );
        let mut lines = m.lines();
        assert!(lines.next().unwrap().chars().count() <= 72);
        assert_eq!(lines.next(), Some(""));
        assert!(m.lines().all(|l| l.chars().count() <= 72));
        assert!(m.contains("Form-by: Ana"));
    }
}
