//! The change loop: a node form in, a preview out, an approval back, a release.
//!
//!     take <form.html> --for <author>   the form onto its own branch, applied,
//!                                       tested, committed and pushed
//!     preview                           where this branch's preview build is
//!     approve <approval.toml>           the author's approval, checked against
//!                                       this exact build, recorded and pushed
//!     approve --verify <branch>         the same check, for the pipeline
//!     queue                             every form branch and where it stands
//!     ship <version>                    a release branch, stamped and proved
//!
//! THE MAINTAINER'S JOB IS THE SAME EVERY TIME, SO IT IS COMMANDS. A person
//! who takes a form in by hand forgets a step on the day they are busy — the
//! branch made from a stale main, the tests not run, the author not named —
//! and the result looks exactly like a careful one. Each command here does its
//! whole step or nothing, and says what to do next.
//!
//! WHAT CROSSES BETWEEN PEOPLE IS FILES. Authors do not use GitHub: they send a
//! filled form, receive a preview package, and send back the approval file the
//! preview saves. How those files travel is the maintainer's choice and nothing
//! here depends on it.
//!
//! AN APPROVAL IS A RECORD, NOT A PASSWORD. It names the branch, the commit and
//! the preview build it was given for, so it cannot be carried to anything
//! else: a change pushed after it needs a new preview and a new approval. It
//! does not prove who pressed the button — the maintainer who received it does.

use crate::pipeline::{OnStop, Run};
use std::fs;
use std::path::{Path, PathBuf};
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

/// Lower case, letters and digits, words joined by `-`: what a branch or a
/// file name may carry of a person's name or a node's id.
pub fn slug(s: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in s.trim().chars() {
        if c.is_ascii_alphanumeric() {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.push(c.to_ascii_lowercase());
        } else {
            gap = true;
        }
    }
    out
}

/// The branch a form is taken onto: `form/<author>/<node>`.
pub fn form_branch(author: &str, node: &str) -> String {
    format!("form/{}/{}", slug(author), slug(node))
}

/// The author a form branch belongs to, and its node, from the branch name.
pub fn parse_form_branch(branch: &str) -> Option<(String, String)> {
    let rest = branch.strip_prefix("form/")?;
    let (author, node) = rest.split_once('/')?;
    (!author.is_empty() && !node.is_empty() && !node.contains('/'))
        .then(|| (author.to_string(), node.to_string()))
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

/// Where the maintainers' work gathers: every form branch starts from it and
/// its pull request goes back into it. `main` takes only what has been
/// gathered here or on `developer`, through a pull request, and a release is
/// cut from `main` alone (CONTRIBUTING.md, "The three branches").
pub const FORMS_BASE: &str = "maintainer";

/// The base every form branch starts from: `origin/maintainer` when it can be
/// fetched, so a branch never starts from a stale copy.
fn fresh_base(root: &Path) -> String {
    match git(root, &["fetch", "-q", "origin", FORMS_BASE]) {
        Ok(_) => format!("origin/{FORMS_BASE}"),
        Err(e) => {
            eprintln!(
                "  (could not fetch origin/{FORMS_BASE} — {e}; starting from the local {FORMS_BASE})"
            );
            FORMS_BASE.to_string()
        }
    }
}

fn after<'a>(args: &'a [&str], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| *a == flag)
        .and_then(|i| args.get(i + 1))
        .copied()
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
// take

/// Everything the form could not have applied, as a note to send back.
fn returned_note(p: &vleo_sheet::template::Plan, file: &str) -> String {
    use vleo_sheet::template::Verdict;
    let mut out = format!(
        "Your node form ({file}) was not taken in.\n\
         Nothing was changed. Fix what is listed below in the same form, save a filled\n\
         copy, and send it again.\n\n"
    );
    for i in &p.items {
        match &i.verdict {
            Verdict::Refused(w) => out.push_str(&format!("  • {} — refused: {w}\n", i.what)),
            Verdict::Conflict(w) => out.push_str(&format!(
                "  • {} — conflicts with a change made since your form was downloaded: {w}\n",
                i.what
            )),
            _ => {}
        }
    }
    for i in p.interfaces.iter().filter(|i| !i.ok()) {
        out.push_str(&format!(
            "  • input {} ← {} — {}\n",
            i.binding, i.var, i.why
        ));
    }
    if p.items.is_empty() {
        out.push_str("  • the form changes nothing in the node\n");
    }
    out
}

/// `take <form.html> --for <author> [--again] [--no-push] [--no-test]`.
pub fn cmd_take(root: &Path, args: &[&str]) -> Result<(), String> {
    use vleo_sheet::template;
    let usage = "usage: cargo run -p xtask -- take <form.html> --for <author> \
                 [--again] [--no-push] [--no-test]";
    let file = args
        .iter()
        .find(|a| !a.starts_with("--") && Some(**a) != after(args, "--for"))
        .copied()
        .ok_or(usage)?;
    let author = after(args, "--for")
        .filter(|a| !slug(a).is_empty())
        .ok_or(usage)?;
    let html = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;

    // 1 — check, and send it back if it cannot be taken. Nothing is touched.
    let p = template::plan(root, &html)?;
    let node = match &p.new {
        Some(n) => n.id.clone(),
        None => p.form.node.clone(),
    };
    if node.is_empty() {
        return Err("the form names no node — it cannot be taken".into());
    }
    let blocked = p.blocked() > 0 || p.interfaces.iter().any(|i| !i.ok()) || p.applicable() == 0;
    if blocked {
        let note = returned_note(&p, file);
        let out = PathBuf::from(format!("{}.returned.txt", file.trim_end_matches(".html")));
        fs::write(&out, &note).map_err(|e| format!("{}: {e}", out.display()))?;
        print!("{note}");
        return Err(format!(
            "the form was not taken — send {} back to {author}. Nothing was changed.",
            out.display()
        ));
    }
    clean(root)?;

    // 2 — its own branch, from a fresh maintainer branch.
    let branch = form_branch(author, &node);
    let exists = git(root, &["rev-parse", "--verify", "--quiet", &branch]).is_ok();
    let mut run = Run::start(root, "take", args, 7);
    let again = format!("cargo run -p xtask -- take {file} --for {author} --again");
    // From the branch on, a stop leaves the branch holding a half-done change:
    // every stop says so, and how to put everything back.
    let half = || {
        OnStop::new(
            format!("{branch} holds a half-done change, uncommitted"),
            format!(
                "fix it and `{again}`; or abandon it: git reset -q --hard && git switch main && \
                 git branch -D {branch}"
            ),
        )
    };
    step(
        &mut run,
        "the branch",
        OnStop::new("unchanged — nothing was written", &again),
        || {
            if exists {
                if !args.contains(&"--again") {
                    return Err(format!(
                        "{branch} already exists. To apply a corrected form onto it, run again \
                     with --again; to start over, delete it first (git branch -D {branch})"
                    ));
                }
                git(root, &["switch", "-q", &branch]).map(|_| ())
            } else {
                let base = fresh_base(root);
                git(root, &["switch", "-q", "-c", &branch, &base]).map(|_| ())
            }
        },
    )?;

    println!("      {branch}");
    // From here a failure leaves the branch holding a half-done change, so
    // every error says how to put everything back.
    let rest = |run: &mut Run| -> Result<(), String> {
        // 3 — apply, and regenerate everything the pipeline regenerates, so the
        // branch never fails its regeneration diff for want of a command.
        step(run, "apply the form", half(), || {
            crate::cmd_intake(root, &[file, "--apply"])
        })?;
        step(run, "regenerate", half(), || {
            crate::cmd_codeowners(root)?;
            crate::cmd_variables(root)?;
            crate::cmd_derisk(root, &[])
        })?;
        step(run, "gate", half(), || crate::cmd_gate(root, &[]))?;
        // A NODE WITH A METHOD IS BUILT FROM IT, stage by stage: translated,
        // tested against its author's cases, their code rerun, the tests shown
        // to test — and only then connected. Stops here if any stage fails.
        let has_method = vleo_sheet::load_all(root)
            .ok()
            .and_then(|t| {
                t.sheets
                    .get(&node)
                    .map(|s| !s.method.text.trim().is_empty())
            })
            .unwrap_or(false);
        if args.contains(&"--no-test") {
            run.skip("build the node from its method", "--no-test");
        } else if !has_method {
            run.skip(
                "build the node from its method",
                "the node has no method: its code is its holes",
            );
        } else {
            step(run, "build the node from its method", half(), || {
                crate::method::cmd_build_node(root, &[node.as_str()])
            })?;
        }
        if args.contains(&"--no-test") {
            run.skip("tests (cargo test --workspace)", "--no-test");
        } else {
            step(run, "tests (cargo test --workspace)", half(), || {
                let ok = Command::new("cargo")
                    .args(["test", "--workspace", "-q"])
                    .current_dir(root)
                    .status()
                    .map_err(|e| e.to_string())?
                    .success();
                if ok {
                    Ok(())
                } else {
                    Err(
                        "tests failed — nothing is committed. A fixture disagreement is a \
                         physics question for the node's owner, never a tolerance to change"
                            .into(),
                    )
                }
            })?;
        }

        // 4 — one commit, naming the author; then push.
        let tree = crate::load(root)?;
        let scope = tree
            .sheets
            .get(&node)
            .map(|s| s.crate_name.trim_start_matches("vleo-").to_string())
            .unwrap_or_else(|| "tree".to_string());
        let f = &p.form;
        let who = if f.name.is_empty() {
            author.to_string()
        } else {
            f.name.clone()
        };
        let mut body = vec![format!(
            "Taken from {who}'s node form{}: {} change(s) applied with `xtask take`.",
            if f.team.is_empty() {
                String::new()
            } else {
                format!(" ({})", f.team)
            },
            p.applicable()
        )];
        if !p.about.is_empty() {
            body.push(format!("It moves the decisions: {}.", p.about.join(", ")));
        }
        body.push(format!("Form-by: {who}"));
        let subject = if p.new.is_some() {
            format!("{node}, a new node from a node form")
        } else {
            format!("{node} from a node form")
        };
        let message = commit_message("feat", &scope, &subject, &body);
        step(run, "commit and push", half(), || {
            // The form is the author's, attached to the pull request — never
            // committed. Received forms live in `forms/`, which git ignores; one
            // saved anywhere else is taken back out of the commit here.
            git(root, &["add", "-A"])?;
            if let Some(rel) = fs::canonicalize(file).ok().and_then(|p| {
                fs::canonicalize(root)
                    .ok()
                    .and_then(|r| p.strip_prefix(r).ok().map(Path::to_path_buf))
            }) {
                let _ = git(root, &["reset", "-q", "--", &rel.to_string_lossy()]);
            }
            commit(root, &message)?;
            push(root, &branch, args)
        })?;

        let repo = repo_slug(root).unwrap_or_else(|| "<owner>/<repo>".into());
        println!(
            "\n\x1b[1mtaken.\x1b[0m {node} from {who} is on {branch}.\n\n\
             next:\n  \
             1 · the preview builds by itself (about 10 minutes):\n      \
             https://github.com/{repo}/actions/workflows/preview.yml?query=branch%3A{}\n  \
             2 · send its package to {who}; they try it and send back the approval file\n  \
             3 · open the pull request:\n      \
             https://github.com/{repo}/compare/{FORMS_BASE}...{branch}?expand=1",
            branch.replace('/', "%2F")
        );
        Ok(())
    };
    let r = rest(&mut run);
    if r.is_ok() {
        run.done(&format!("take: {node} is on {branch}"));
    }
    r.map_err(|e| {
        format!("{e}\n  to abandon it: git reset -q --hard && git switch main && git branch -D {branch}")
    })
}

// ---------------------------------------------------------------------------
// preview

/// `preview` — where the current form branch's preview build is.
pub fn cmd_preview(root: &Path, _args: &[&str]) -> Result<(), String> {
    let branch = current_branch(root)?;
    let repo = repo_slug(root).unwrap_or_else(|| "<owner>/<repo>".into());
    let Some((author, node)) = parse_form_branch(&branch) else {
        return Err(format!(
            "{branch} is not a form branch (form/<author>/<node>) — previews are built for \
             those. `cargo run -p xtask -- queue` lists them"
        ));
    };
    let pushed = git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("origin/{branch}"),
        ],
    )
    .ok();
    let local = head(root)?;
    println!(
        "{branch} — {author}'s change to {node}\n\n\
         Every push to this branch builds a preview by itself (about 10 minutes):\n  \
         https://github.com/{repo}/actions/workflows/preview.yml?query=branch%3A{}\n\n\
         Open the newest green run, download vleo-preview-{author}-<build> at the bottom,\n\
         unzip it once, and send the .whl inside to {author}. They install it with\n\
         `python -m pip install <file>`, start it with `python -m vleo`, try the changed\n\
         nodes, press Approve, and send back the approval file it saves. Then:\n  \
         cargo run -p xtask -- approve <that file>\n\n\
         The macOS engines are built only on request: Actions → preview → Run workflow →\n\
         this branch → \"also build the macOS engines\".",
        branch.replace('/', "%2F")
    );
    match pushed {
        Some(remote) if remote == local => {}
        Some(_) => println!(
            "\n\x1b[33mthis branch has commits that are not pushed — the preview is of the pushed \
             ones. git push\x1b[0m"
        ),
        None => println!(
            "\n\x1b[33mthis branch is not pushed yet, so it has no preview: git push -u origin \
             {branch}\x1b[0m"
        ),
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// approve

/// Where a form branch's approval is kept: `approvals/<author>--<node>.toml`.
pub fn approval_path(branch: &str) -> Option<String> {
    let (author, node) = parse_form_branch(branch)?;
    Some(format!("approvals/{author}--{node}.toml"))
}

/// An approval file, as the preview saves it.
#[derive(Debug)]
pub struct Approval {
    pub branch: String,
    pub commit: String,
    pub run: String,
    pub nodes: Vec<String>,
    pub by: String,
    pub at: String,
    pub checked: bool,
    pub note: String,
}

pub fn read_approval(text: &str) -> Result<Approval, String> {
    let v: toml::Value = text
        .parse()
        .map_err(|e| format!("it is not an approval file: {e}"))?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    if s("kind") != "vleo-approval" {
        return Err("it is not an approval file (no `kind = \"vleo-approval\"`)".into());
    }
    if v.get("version").and_then(|x| x.as_integer()) != Some(1) {
        return Err("it is an approval file of a version this tool does not read".into());
    }
    let a = Approval {
        branch: s("branch"),
        commit: s("commit"),
        run: s("run"),
        nodes: v
            .get("nodes")
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|n| n.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        by: s("approved_by"),
        at: s("approved_at"),
        checked: v.get("checked").and_then(|x| x.as_bool()).unwrap_or(false),
        note: s("note"),
    };
    for (k, val) in [
        ("branch", &a.branch),
        ("commit", &a.commit),
        ("run", &a.run),
        ("approved_by", &a.by),
    ] {
        if val.trim().is_empty() {
            return Err(format!("the approval file has no {k}"));
        }
    }
    if !a.checked {
        return Err(format!(
            "{} did not confirm they ran the changed nodes and got what they expect — that is \
             what an approval is. Ask them to try it and approve again",
            a.by
        ));
    }
    Ok(a)
}

/// Whether `commit` is what `at` is, apart from approval records: the approval
/// binds only while nothing else has changed since the build it was given for.
fn binds(root: &Path, commit: &str, at: &str, again: &str) -> Result<(), String> {
    git(root, &["cat-file", "-e", &format!("{commit}^{{commit}}")]).map_err(|_| {
        format!("the approved commit {commit} is not in this repository — git fetch, or the branch was rewritten")
    })?;
    git(root, &["merge-base", "--is-ancestor", commit, at])
        .map_err(|_| format!("the approved commit {commit} is not part of this branch any more"))?;
    let changed = git(root, &["diff", "--name-only", commit, at])?;
    let other: Vec<&str> = changed
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with("approvals/") && !l.starts_with("acceptances/"))
        .collect();
    if other.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "the branch has changed since the build that was approved ({} file(s), e.g. {}) — {again}",
            other.len(),
            other[0]
        ))
    }
}

/// `approve <file> [--no-push]` · `approve --verify <branch>`.
pub fn cmd_approve(root: &Path, args: &[&str]) -> Result<(), String> {
    if let Some(branch) = after(args, "--verify") {
        return verify_approval(root, branch, "HEAD");
    }
    let file = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .copied()
        .ok_or("usage: cargo run -p xtask -- approve <approval.toml> [--no-push] | approve --verify <branch>")?;
    let text = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let a = read_approval(&text)?;
    let store =
        approval_path(&a.branch).ok_or_else(|| format!("{} is not a form branch", a.branch))?;
    clean(root)?;
    if current_branch(root)? != a.branch {
        git(root, &["switch", "-q", &a.branch])
            .map_err(|e| format!("the approval is for {}, which is not here: {e}", a.branch))?;
        println!("  switched to {}", a.branch);
    }
    binds(
        root,
        &a.commit,
        "HEAD",
        "send the author the new preview and ask them to approve that",
    )?;
    fs::create_dir_all(root.join("approvals")).map_err(|e| e.to_string())?;
    fs::write(root.join(&store), &text).map_err(|e| format!("{store}: {e}"))?;
    let message = commit_message(
        "chore",
        "tree",
        // The subject starts with the change, not the name: the commit rule
        // refuses a capital, and most names start with one.
        &format!("preview build {} approved by {}", a.run, a.by),
        &[
            format!(
                "{} tried preview build {} of {} ({}) and approved it{}.",
                a.by,
                a.run,
                if a.nodes.is_empty() {
                    "the change".to_string()
                } else {
                    a.nodes.join(", ")
                },
                &a.commit.chars().take(10).collect::<String>(),
                if a.at.is_empty() {
                    String::new()
                } else {
                    format!(" on {}", a.at)
                }
            ),
            if a.note.is_empty() {
                String::new()
            } else {
                format!("Their note: {}", a.note)
            },
            format!("Tested-by: {}", a.by),
        ]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>(),
    );
    git(root, &["add", &store])?;
    commit(root, &message)?;
    push(root, &a.branch, args)?;
    let repo = repo_slug(root).unwrap_or_else(|| "<owner>/<repo>".into());
    println!(
        "\n\x1b[1mapproved.\x1b[0m {} approved build {} — recorded in {store}.\n\
         The pull request can now be reviewed and merged:\n  \
         https://github.com/{repo}/compare/{FORMS_BASE}...{}?expand=1",
        a.by, a.run, a.branch
    );
    Ok(())
}

/// The pipeline's check: a form branch carries an approval of its own current
/// content. Everything after the approved commit may be approval records only.
fn verify_approval(root: &Path, branch: &str, at: &str) -> Result<(), String> {
    if branch.starts_with("group/") {
        return verify_acceptance(root, branch, at);
    }
    let Some(store) = approval_path(branch) else {
        println!("{branch} is not a form branch — no approval is asked of it");
        return Ok(());
    };
    let text = git(root, &["show", &format!("{at}:{store}")]).map_err(|_| {
        format!(
            "{branch} has no approval yet ({store}). Send its author the preview; when they \
             approve, run `cargo run -p xtask -- approve <their file>`"
        )
    })?;
    let a = read_approval(&text)?;
    if a.branch != branch {
        return Err(format!("{store} approves {}, not {branch}", a.branch));
    }
    binds(
        root,
        &a.commit,
        at,
        "send the author the new preview and ask them to approve that",
    )?;
    println!(
        "{branch}: approved by {} (preview build {}, commit {}) and unchanged since",
        a.by,
        a.run,
        a.commit.chars().take(10).collect::<String>()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// group-accept

/// The branch a group's release is taken in, built and delivered on.
pub fn group_branch(group: &str, version: &str) -> String {
    format!("group/{group}-{version}")
}

/// Where the group's acceptance of a release is kept on its branch.
pub fn acceptance_path(group: &str, version: &str) -> String {
    format!("acceptances/{group}-{version}.toml")
}

/// What the group application writes when the lead answers a test application.
pub struct Acceptance {
    pub group: String,
    pub version: String,
    pub fingerprint: String,
    pub commit: String,
    pub delivery: String,
    pub verdict: String,
    pub by: String,
    pub at: String,
    pub tried: String,
    pub note: String,
}

pub fn read_acceptance(text: &str) -> Result<Acceptance, String> {
    let v: toml::Value = text
        .parse()
        .map_err(|e| format!("it is not an acceptance file: {e}"))?;
    let s = |k: &str| {
        v.get(k)
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let a = Acceptance {
        group: s("group"),
        version: s("version"),
        fingerprint: s("fingerprint"),
        commit: s("commit"),
        delivery: s("delivery"),
        verdict: s("verdict"),
        by: s("by"),
        at: s("at"),
        tried: s("tried"),
        note: s("note"),
    };
    for (k, val) in [
        ("group", &a.group),
        ("version", &a.version),
        ("fingerprint", &a.fingerprint),
        ("commit", &a.commit),
        ("verdict", &a.verdict),
        ("by", &a.by),
    ] {
        if val.is_empty() {
            return Err(format!("the acceptance file has no {k}"));
        }
    }
    Ok(a)
}

/// `group-accept <file.accept.toml> [--delivery <DELIVERY.toml>] [--no-push]` ·
/// `group-accept --verify <branch>`.
///
/// The group's answer to a test application, recorded on the branch the
/// release was built on. An answer of `changes` is never recorded: its note
/// goes back into the group's work, and their next release comes back as a new
/// test application. An acceptance binds the build it was given for — the
/// commit the delivery names — and nothing changed since but these records.
pub fn cmd_group_accept(root: &Path, args: &[&str]) -> Result<(), String> {
    if let Some(branch) = after(args, "--verify") {
        return verify_acceptance(root, branch, "HEAD");
    }
    let file = args
        .iter()
        .enumerate()
        .find(|(i, a)| !a.starts_with("--") && (*i == 0 || args[i - 1] != "--delivery"))
        .map(|(_, a)| *a)
        .ok_or("usage: cargo run -p xtask -- group-accept <file.accept.toml> [--delivery <DELIVERY.toml>] [--no-push] | group-accept --verify <branch>")?;
    let text = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let a = read_acceptance(&text)?;
    match a.verdict.as_str() {
        "accepted" => {}
        "changes" => {
            return Err(format!(
                "{} answered CHANGES for {} {}, so nothing is recorded. Their note:\n\n  {}\n{}\n\
                 It goes back into the group's work; their next sealed release comes back through the loop.",
                a.by,
                a.group,
                a.version,
                if a.note.is_empty() { "(none)" } else { &a.note },
                if a.tried.is_empty() {
                    String::new()
                } else {
                    format!("  What they tried: {}\n", a.tried)
                }
            ))
        }
        other => return Err(format!("the verdict is «{other}»; it is accepted or changes")),
    }
    if a.tried.is_empty() {
        return Err(format!(
            "{} accepted without saying what they tried. An acceptance says what it rests on: ask them to answer again",
            a.by
        ));
    }
    vleo_sheet::form::refuse_agent_attribution(root, &a.by).map_err(|e| e.to_string())?;
    // With the delivery record beside it, the answer must be to that record.
    if let Some(d) = after(args, "--delivery") {
        let bytes = fs::read(d).map_err(|e| format!("{d}: {e}"))?;
        let have = crate::group::sha256_hex(&bytes);
        if have != a.delivery {
            return Err(format!(
                "the answer is to another delivery record: {d} hashes to {}…, the answer names {}…",
                &have[..16],
                a.delivery.chars().take(16).collect::<String>()
            ));
        }
    }
    let branch = group_branch(&a.group, &a.version);
    let store = acceptance_path(&a.group, &a.version);
    clean(root)?;
    if current_branch(root)? != branch {
        git(root, &["switch", "-q", &branch])
            .map_err(|e| format!("the acceptance is for {branch}, which is not here: {e}"))?;
        println!("  switched to {branch}");
    }
    binds(
        root,
        &a.commit,
        "HEAD",
        "deliver it again (`xtask group-deliver`) and ask the group to answer that build",
    )?;
    fs::create_dir_all(root.join("acceptances")).map_err(|e| e.to_string())?;
    fs::write(root.join(&store), &text).map_err(|e| format!("{store}: {e}"))?;
    let message = commit_message(
        "chore",
        "tree",
        &format!("{} {} accepted by {}", a.group, a.version, a.by),
        &[
            format!(
                "{} tried the test application of {} {} (commit {}, release fingerprint {}…, delivery record {}…) and accepted it{}.",
                a.by,
                a.group,
                a.version,
                a.commit.chars().take(10).collect::<String>(),
                a.fingerprint.chars().take(16).collect::<String>(),
                a.delivery.chars().take(16).collect::<String>(),
                if a.at.is_empty() {
                    String::new()
                } else {
                    format!(" on {}", a.at)
                }
            ),
            format!("What they tried: {}", a.tried),
            if a.note.is_empty() {
                String::new()
            } else {
                format!("Their note: {}", a.note)
            },
            format!("Tested-by: {}", a.by),
        ]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>(),
    );
    git(root, &["add", &store])?;
    commit(root, &message)?;
    push(root, &branch, args)?;
    println!(
        "\n\x1b[1maccepted.\x1b[0m {} accepted {} {} — recorded in {store}.\n\
         The pull request from {branch} can now be reviewed and merged.",
        a.by, a.group, a.version
    );
    Ok(())
}

/// The pipeline's check on a group branch: it carries the group's acceptance
/// of its own current content.
fn verify_acceptance(root: &Path, branch: &str, at: &str) -> Result<(), String> {
    let Some((group, version)) = branch
        .strip_prefix("group/")
        .and_then(|r| r.rsplit_once('-'))
    else {
        return Err(format!(
            "{branch} is not a group branch (group/<group>-<version>)"
        ));
    };
    let store = acceptance_path(group, version);
    let text = git(root, &["show", &format!("{at}:{store}")]).map_err(|_| {
        format!(
            "{branch} has no acceptance yet ({store}). Send the group its test application \
             (`xtask group-deliver`); when they accept, run `cargo run -p xtask -- group-accept <their file>`"
        )
    })?;
    let a = read_acceptance(&text)?;
    if a.group != group || a.version != version || a.verdict != "accepted" {
        return Err(format!(
            "{store} is {} of {} {}, not an acceptance of {group} {version}",
            a.verdict, a.group, a.version
        ));
    }
    binds(
        root,
        &a.commit,
        at,
        "deliver it again (`xtask group-deliver`) and ask the group to answer that build",
    )?;
    println!(
        "{branch}: accepted by {} (commit {}) and unchanged since",
        a.by,
        a.commit.chars().take(10).collect::<String>()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// queue

/// `queue` — every form branch and where it stands.
pub fn cmd_queue(root: &Path, _args: &[&str]) -> Result<(), String> {
    if let Err(e) = git(root, &["fetch", "-q", "--prune", "origin"]) {
        eprintln!("  (could not fetch — showing what this copy last saw: {e})");
    }
    let refs = git(
        root,
        &[
            "for-each-ref",
            "--format=%(refname:short)|%(committerdate:short)",
            "refs/remotes/origin/form/",
            "refs/heads/form/",
        ],
    )?;
    let mut seen = std::collections::BTreeMap::new();
    for line in refs.lines().filter(|l| !l.is_empty()) {
        let (r, date) = line.split_once('|').unwrap_or((line, ""));
        let branch = r.trim_start_matches("origin/").to_string();
        let remote = r.starts_with("origin/");
        let e = seen
            .entry(branch)
            .or_insert((String::new(), false, false, String::new()));
        e.3 = date.to_string();
        if remote {
            e.0 = r.to_string();
            e.1 = true;
        } else {
            e.2 = true;
            if e.0.is_empty() {
                e.0 = r.to_string();
            }
        }
    }
    if seen.is_empty() {
        println!(
            "no form branches — nothing is waiting. `xtask take <form> --for <author>` starts one."
        );
        return Ok(());
    }
    println!("{:<44} {:<11} stage", "branch", "last change");
    for (branch, (r, remote, _local, date)) in &seen {
        let store = approval_path(branch).unwrap_or_default();
        let on_main = git(root, &["show", &format!("origin/{FORMS_BASE}:{store}")]).ok();
        let on_branch = git(root, &["show", &format!("{r}:{store}")]).ok();
        let stage = if on_main.is_some() && on_main == on_branch {
            "merged — delete the branch: git push origin --delete ".to_string() + branch
        } else if !remote {
            "not pushed — git push -u origin ".to_string() + branch
        } else {
            match verify_approval_quiet(root, branch, r) {
                Ok(by) => format!("approved by {by} — review and merge the pull request"),
                Err(e) if on_branch.is_some() => format!("approval out of date — {e}"),
                Err(_) => "waiting for the author's approval of the preview".to_string(),
            }
        };
        println!("{branch:<44} {date:<11} {stage}");
    }
    Ok(())
}

fn verify_approval_quiet(root: &Path, branch: &str, at: &str) -> Result<String, String> {
    let store = approval_path(branch).ok_or("not a form branch")?;
    let text = git(root, &["show", &format!("{at}:{store}")])?;
    let a = read_approval(&text)?;
    binds(
        root,
        &a.commit,
        at,
        "send the author the new preview and ask them to approve that",
    )?;
    Ok(a.by)
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
    let last = git(root, &["describe", "--tags", "--abbrev=0", "origin/main"]).unwrap_or_default();
    let range = if last.is_empty() {
        "HEAD".to_string()
    } else {
        format!("{last}..HEAD")
    };
    let forms = git(
        root,
        &[
            "log",
            &range,
            "--format=%(trailers:key=Form-by,valueonly,separator=%x2C)",
        ],
    )
    .unwrap_or_default();
    let mut authors: Vec<&str> = forms
        .split([',', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    authors.sort();
    authors.dedup();
    let message = commit_message(
        "chore",
        "tree",
        &format!("release {version}"),
        &[
            format!("`xtask ship {version}`: every node version still `next` is stamped with {version}, and the workspace and every lock file move to it."),
            if authors.is_empty() {
                "No node form since the last release.".to_string()
            } else {
                format!("Node forms since {}: {}.", if last.is_empty() { "the start" } else { &last }, authors.join(", "))
            },
        ],
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
    fn a_form_branch_names_its_author_and_node_safely() {
        assert_eq!(
            form_branch("Ana Sharma", "pay_radiance"),
            "form/ana-sharma/pay-radiance"
        );
        assert_eq!(form_branch("../x", "a b"), "form/x/a-b");
        assert_eq!(
            parse_form_branch("form/ana-sharma/pay-radiance"),
            Some(("ana-sharma".into(), "pay-radiance".into()))
        );
        assert_eq!(parse_form_branch("main"), None);
        assert_eq!(parse_form_branch("form/ana"), None);
        assert_eq!(parse_form_branch("form/ana/x/y"), None);
    }

    #[test]
    fn an_approval_is_read_strictly() {
        let good = r#"kind = "vleo-approval"
version = 1
branch = "form/ana/x"
commit = "abc"
run = "12"
nodes = ["x"]
approved_by = "Ana"
approved_at = "2026-09-28T00:00:00Z"
checked = true
note = ""
"#;
        let a = read_approval(good).unwrap();
        assert_eq!(
            (a.branch.as_str(), a.run.as_str(), a.by.as_str()),
            ("form/ana/x", "12", "Ana")
        );
        assert!(
            read_approval(&good.replace("checked = true", "checked = false"))
                .unwrap_err()
                .contains("did not confirm")
        );
        assert!(read_approval(&good.replace("vleo-approval", "other")).is_err());
        assert!(read_approval(&good.replace("version = 1", "version = 2")).is_err());
        assert!(
            read_approval(&good.replace("approved_by = \"Ana\"", "approved_by = \"\"")).is_err()
        );
        assert!(read_approval("not toml at all [").is_err());
        assert_eq!(
            approval_path("form/ana/x").as_deref(),
            Some("approvals/ana--x.toml")
        );
        assert_eq!(approval_path("main"), None);
    }

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
