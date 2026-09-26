//! The manual says what is true.
//!
//! `docs/manual.toml` is what the browser's Manual tab shows about the tool:
//! what to click, what to paste, what each command is for, which settings exist,
//! what is in the repository. It is prose, and prose drifts — this repository's
//! own README quotes a row count four hundred out of date and an example that
//! stopped answering months ago. So every NAME the manual uses is held here
//! against the thing it names, and in both directions:
//!
//!   the manual → the code   nothing documented has been renamed or removed
//!   the code → the manual   nothing that exists has been left undocumented
//!
//! The second is the one that matters over time. A manual that is merely
//! accurate about the half it covers reads as complete, and the reader has no
//! way to know what is missing.
//!
//! What this cannot check is whether a command DOES what the manual says. That
//! is `tools/manual_check.py`, which runs every one of them, in the pipeline.
//!
//! Each check is a function over the sets it compares, so the last tests here
//! can hand each one a deliberately wrong input and watch it refuse — a check
//! that passes whatever it is given is not checking anything.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use vleo_sheet::manual::{self, Check, Manual};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read(p: &str) -> String {
    std::fs::read_to_string(root().join(p)).unwrap_or_else(|e| panic!("{p}: {e}"))
}

fn the_manual() -> Manual {
    manual::load(&root()).unwrap_or_else(|e| panic!("the manual does not load: {e}"))
}

// ── what the code says exists ────────────────────────────────────────────────

/// The command names a binary dispatches on, read from its `match cmd {`.
///
/// Read from the source because the dispatch IS the list — there is no second
/// table to consult. An arm that cannot be found fails loudly rather than
/// yielding an empty set, which would let every manual entry pass as "not a
/// command, but nothing to compare against either".
fn dispatched(src: &str) -> BTreeSet<String> {
    let start = src
        .find("let r = match cmd {")
        .expect("the dispatch `let r = match cmd {` is where the commands are; it has moved");
    let mut out = BTreeSet::new();
    for line in src[start..].lines().skip(1) {
        let t = line.trim();
        if t.starts_with("other =>") || t.starts_with("_ =>") {
            break;
        }
        // `"name" => …` and `"a" | "b" => …`
        if let Some((lhs, _)) = t.split_once("=>") {
            for part in lhs.split('|') {
                let p = part.trim();
                if p.len() > 2 && p.starts_with('"') && p.ends_with('"') {
                    out.insert(p[1..p.len() - 1].to_string());
                }
            }
        }
    }
    assert!(!out.is_empty(), "no commands found in a dispatch block");
    // `help` is not a command a person needs told about — the manual says how
    // to ask for it in prose.
    for h in ["help", "--help", "-h"] {
        out.remove(h);
    }
    out
}

/// The routes the daemon answers, as `METHOD path`, with a prefix route written
/// the way the manual writes it: `/v1/node/<id>`.
fn routed(src: &str) -> BTreeSet<String> {
    let start = src
        .find("match (method, path) {")
        .expect("the daemon's router has moved");
    let end = start
        + src[start..]
            .find("\"404 Not Found\"")
            .expect("the router's fallback has moved");
    let mut out = BTreeSet::new();
    let body = &src[start..end];
    let mut rest = body;
    // Every `("GET", "/x")` and every `("GET", p) if p.starts_with("/x/")`.
    while let Some(i) = rest.find("(\"") {
        rest = &rest[i + 2..];
        let Some(q) = rest.find('"') else { break };
        let method = &rest[..q];
        if method != "GET" && method != "POST" {
            continue;
        }
        let after = &rest[q + 1..];
        let after = after.trim_start_matches([',', ' ']);
        if let Some(lit) = after.strip_prefix('"') {
            let path = &lit[..lit.find('"').unwrap()];
            out.insert(format!("{method} {path}"));
        } else if let Some(j) = after.find("starts_with(\"") {
            let p = &after[j + "starts_with(\"".len()..];
            let prefix = &p[..p.find('"').unwrap()];
            let tail = if prefix == "/js/" {
                "<module>"
            } else if prefix.contains("bundle") || prefix.contains("parity") {
                "<name>"
            } else {
                "<id>"
            };
            out.insert(format!("{method} {prefix}{tail}"));
        }
    }
    out
}

/// Every `VLEO_*` variable the code actually READS — not every mention: the C
/// binding has a constant called VLEO_DATA that is an error code, not a setting.
fn env_read() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut files = Vec::new();
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                // Node folders are generated and read no settings; target is
                // build output.
                if name != "target" && name != "nodes" && !name.starts_with('.') {
                    walk(&p, files);
                }
            } else if name.ends_with(".rs") || name.ends_with(".py") {
                files.push(p);
            }
        }
    }
    for d in ["crates", "xtask", "tools"] {
        walk(&root().join(d), &mut files);
    }
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        for pat in [
            "env::var(\"",
            "env::var_os(\"",
            "environ.get(\"",
            "environ[\"",
        ] {
            let mut rest = text.as_str();
            while let Some(i) = rest.find(pat) {
                rest = &rest[i + pat.len()..];
                let name = &rest[..rest.find('"').unwrap_or(0)];
                if name.starts_with("VLEO_") {
                    out.insert(name.to_string());
                }
            }
        }
    }
    out
}

/// The entries git tracks at the top of the repository.
fn tracked_top() -> BTreeSet<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root())
        .args(["ls-files"])
        .output()
        .expect("git is needed to know what the repository tracks");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.split('/').next())
        .map(|s| s.to_string())
        .collect()
}

// ── the checks ───────────────────────────────────────────────────────────────

/// Two sets that must be the same, with each difference named from the side it
/// is missing on.
fn same(what: &str, documented: &BTreeSet<String>, real: &BTreeSet<String>) -> Result<(), String> {
    let gone: Vec<_> = documented.difference(real).collect();
    let undocumented: Vec<_> = real.difference(documented).collect();
    if gone.is_empty() && undocumented.is_empty() {
        return Ok(());
    }
    let mut e = format!("the manual and the code disagree about {what}:");
    if !gone.is_empty() {
        e.push_str(&format!("\n  in the manual, not in the code: {gone:?}"));
    }
    if !undocumented.is_empty() {
        e.push_str(&format!(
            "\n  in the code, not in the manual: {undocumented:?}"
        ));
    }
    Err(e)
}

/// The names the manual's commands are held to.
struct Known {
    xtask: BTreeSet<String>,
    vleo: BTreeSet<String>,
    routes: BTreeSet<String>,
    env: BTreeSet<String>,
    crates: BTreeSet<String>,
}

/// Whether one command line names only things that exist.
///
/// Every shape a manual command takes is listed here, and anything else is
/// refused as unknown — so a new kind of command makes this test grow a rule
/// for it rather than pass unchecked.
fn command_ok(line: &str, k: &Known) -> Result<(), String> {
    for part in line.split("&&") {
        let mut words: Vec<&str> = part.split_whitespace().collect();
        // Leading `VAR=value` settings.
        while let Some(w) = words.first() {
            match w.split_once('=') {
                Some((var, _)) if var.chars().all(|c| c.is_ascii_uppercase() || c == '_') => {
                    if !k.env.contains(var) {
                        return Err(format!(
                            "`{line}` sets {var}, which nothing in the code reads"
                        ));
                    }
                    words.remove(0);
                }
                _ => break,
            }
        }
        let sub = |after: &[&str]| after.first().map(|s| s.to_string()).unwrap_or_default();
        match words.as_slice() {
            ["cargo", "run", "-p", "xtask", "--", rest @ ..] | ["cargo", "xtask", rest @ ..] => {
                let c = sub(rest);
                if c != "help" && !k.xtask.contains(&c) {
                    return Err(format!("`{line}`: xtask has no command '{c}'"));
                }
            }
            ["cargo", "run", "-p", "vleo-cli", "--bin", "vleo", "--", rest @ ..]
            | ["cargo", "vleo", rest @ ..] => {
                let c = sub(rest);
                if c != "help" && !k.vleo.contains(&c) {
                    return Err(format!("`{line}`: vleo has no command '{c}'"));
                }
            }
            ["cargo", "run", "--release", "-p", "vleo-daemon"] => {}
            ["cargo", "test"] | ["cargo", "fmt", ..] | ["cargo", "clippy", ..] => {}
            ["cargo", "test", "-p", krate] => {
                if !k.crates.contains(*krate) {
                    return Err(format!("`{line}`: there is no crate '{krate}'"));
                }
            }
            ["python3", tool, ..] => {
                if !root().join(tool).is_file() {
                    return Err(format!("`{line}`: {tool} does not exist"));
                }
            }
            ["curl", .., url] => {
                let url = url.trim_matches('\'');
                let path = url
                    .split_once("://")
                    .and_then(|(_, r)| r.split_once('/'))
                    .map(|(_, p)| format!("/{}", p.split('?').next().unwrap_or("")))
                    .ok_or_else(|| format!("`{line}`: not a URL this test can read"))?;
                let hit = k.routes.iter().any(|r| {
                    let (m, pat) = r.split_once(' ').unwrap();
                    m == "GET"
                        && match pat.split_once('<') {
                            Some((prefix, _)) => {
                                path.starts_with(prefix) && path.len() > prefix.len()
                            }
                            None => path == pat,
                        }
                });
                if !hit {
                    return Err(format!("`{line}`: the daemon answers no GET {path}"));
                }
            }
            ["git", ..] | ["cd", ..] => {}
            _ => {
                return Err(format!(
                    "`{line}` is a kind of command this test does not know how to check. \
                     Teach `command_ok` about it rather than let it through unchecked"
                ))
            }
        }
    }
    Ok(())
}

fn known() -> Known {
    let mut crates: BTreeSet<String> = std::fs::read_dir(root().join("crates"))
        .unwrap()
        .flatten()
        .filter(|e| e.path().join("Cargo.toml").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    crates.insert("xtask".into());
    Known {
        xtask: dispatched(&read("xtask/src/main.rs")),
        vleo: dispatched(&read("crates/vleo-cli/src/main.rs")),
        routes: routed(&read("crates/vleo-daemon/src/main.rs")),
        env: env_read(),
        crates,
    }
}

// ── the tests ────────────────────────────────────────────────────────────────

#[test]
fn the_manual_loads() {
    let m = the_manual();
    assert!(
        m.layers.len() >= 2,
        "a terminal layer and a browser layer, at least"
    );
    for want in ["browser", "terminal"] {
        assert!(
            m.layers.iter().any(|l| l.id == want),
            "the manual has no '{want}' layer, and a person is told to look for one"
        );
    }
}

#[test]
fn every_command_of_both_programs_is_documented_and_nothing_else() {
    let m = the_manual();
    let k = known();
    for (tool, real) in [("xtask", &k.xtask), ("vleo", &k.vleo)] {
        let documented: BTreeSet<String> = m
            .commands
            .iter()
            .filter(|c| c.tool == tool)
            .map(|c| c.name.clone())
            .collect();
        same(&format!("the commands of {tool}"), &documented, real)
            .unwrap_or_else(|e| panic!("{e}"));
    }
}

#[test]
fn every_usage_line_is_the_one_the_program_prints() {
    // The arguments are the part of a command a person copies. Held to the
    // program's own help text, which is a literal in its source.
    let m = the_manual();
    // Unescaped, so the comparison is with what the help PRINTS: `--by "<name>"`
    // is written `--by \"<name>\"` inside the string literal.
    let xtask = read("xtask/src/main.rs").replace("\\\"", "\"");
    let vleo = read("crates/vleo-cli/src/main.rs").replace("\\\"", "\"");
    for c in &m.commands {
        let src = if c.tool == "xtask" { &xtask } else { &vleo };
        assert!(
            src.contains(&c.usage),
            "{} {}: the manual gives the usage as `{}`, and the program's own help does not say that",
            c.tool,
            c.name,
            c.usage
        );
    }
}

#[test]
fn every_command_the_manual_gives_names_things_that_exist() {
    let m = the_manual();
    let k = known();
    let mut bad = Vec::new();
    for l in &m.layers {
        for s in &l.sections {
            for st in &s.steps {
                if let Some(r) = &st.run {
                    if let Err(e) = command_ok(r, &k) {
                        bad.push(format!("{}: {e}", s.id));
                    }
                }
            }
        }
    }
    // Where the browser says a thing is done instead, and it is a command, the
    // command is held to the same rule. "Do it over there" pointing at nothing
    // is worse than no answer.
    for c in &m.cannot {
        if c.instead.starts_with("cargo ") || c.instead.starts_with("python3 ") {
            if let Err(e) = command_ok(&c.instead, &k) {
                bad.push(format!("cannot '{}': {e}", c.what));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn every_route_is_documented_and_nothing_else() {
    let m = the_manual();
    let documented: BTreeSet<String> = m
        .routes
        .iter()
        .map(|r| format!("{} {}", r.method, r.path))
        .collect();
    same("the daemon's routes", &documented, &known().routes).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn every_route_that_writes_says_so() {
    // A route that changes a sheet is refused unless the daemon was started
    // for it. The manual's `writes` flag is what tells a scripter that; held to
    // the router's own write guard.
    let src = read("crates/vleo-daemon/src/main.rs");
    let m = the_manual();
    for r in &m.routes {
        let handler_writes = r.method == "POST"
            && [
                "/v1/sheet/",
                "/v1/block/",
                "/v1/view/",
                "/v1/publish/",
                "/v1/propose",
            ]
            .iter()
            .any(|p| r.path.starts_with(p));
        assert_eq!(
            r.writes, handler_writes,
            "{} {}: the manual says writes = {}, and the router says otherwise",
            r.method, r.path, r.writes
        );
    }
    assert!(
        src.contains("fn may_write"),
        "the write guard the manual describes has moved"
    );
}

#[test]
fn every_setting_the_code_reads_is_documented_and_nothing_else() {
    let m = the_manual();
    let documented: BTreeSet<String> = m.env.iter().map(|e| e.name.clone()).collect();
    same("the environment variables", &documented, &env_read()).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn every_label_the_manual_says_to_click_is_on_the_page() {
    // A button renamed and a manual still telling people to press the old name
    // is the most ordinary drift there is, and the reader blames themselves.
    let mut page = read("web/index.html");
    for e in std::fs::read_dir(root().join("web/js")).unwrap().flatten() {
        page.push_str(&std::fs::read_to_string(e.path()).unwrap_or_default());
    }
    let m = the_manual();
    let mut missing = Vec::new();
    for l in &m.layers {
        for s in &l.sections {
            for st in &s.steps {
                if let Some(ui) = &st.ui {
                    if !page.contains(ui.as_str()) {
                        missing.push(format!("{}: \"{ui}\"", s.id));
                    }
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "labels the manual names and the page does not show:\n{}",
        missing.join("\n")
    );
}

#[test]
fn every_top_level_entry_is_documented_and_nothing_else() {
    let m = the_manual();
    for f in &m.folders {
        assert!(
            root().join(&f.name).exists(),
            "the manual describes '{}', which is not in the repository",
            f.name
        );
    }
    let documented: BTreeSet<String> = m.folders.iter().map(|f| f.name.clone()).collect();
    let tracked = tracked_top();
    let undocumented: Vec<_> = tracked.difference(&documented).collect();
    assert!(
        undocumented.is_empty(),
        "tracked at the top of the repository and not in the manual: {undocumented:?}"
    );
}

#[test]
fn every_document_is_listed_and_nothing_else() {
    let m = the_manual();
    let documented: BTreeSet<String> = m.docs.iter().map(|d| d.path.clone()).collect();
    let real: BTreeSet<String> = std::fs::read_dir(root().join("docs"))
        .unwrap()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| format!("docs/{}", e.file_name().to_string_lossy()))
        .filter(|p| p.ends_with(".md") || p.ends_with(".toml"))
        .collect();
    same("the documents under docs/", &documented, &real).unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn every_command_the_pipeline_is_said_to_run_it_does_run() {
    // `check = "ci"` is a claim that the pipeline runs this command itself. Each
    // part must begin a `run:` line of the workflow — `cargo test` is satisfied
    // by `cargo test --workspace`, which is the same command with more said.
    let wf = read(".github/workflows/gate.yml");
    let runs: Vec<String> = wf
        .lines()
        .map(|l| l.trim().trim_start_matches("run:").trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let m = the_manual();
    for l in &m.layers {
        for s in &l.sections {
            for st in &s.steps {
                if st.check != Some(Check::Ci) {
                    continue;
                }
                for part in st.run.as_deref().unwrap().split("&&").map(str::trim) {
                    assert!(
                        runs.iter().any(|r| r.starts_with(part)),
                        "{}: the manual says the pipeline runs `{part}`, and gate.yml does not",
                        s.id
                    );
                }
            }
        }
    }
}

#[test]
fn the_sections_the_page_fills_from_live_data_exist() {
    // The Manual view draws its tables into these sections by id. Renamed here
    // and not there, the table would simply not appear — nothing would fail.
    let js = read("web/js/manual.js");
    let m = the_manual();
    let ids: BTreeSet<&str> = m
        .layers
        .iter()
        .flat_map(|l| l.sections.iter().map(|s| s.id.as_str()))
        .collect();
    for id in [
        "browser-limits",
        "code-folders",
        "code-docs",
        "ref-xtask",
        "ref-vleo",
        "ref-routes",
        "ref-env",
        "ref-sheet",
        "ref-locked",
    ] {
        assert!(
            ids.contains(id),
            "the manual has no section '{id}', and the page fills one"
        );
        assert!(
            js.contains(&format!("'{id}'")),
            "the page no longer fills '{id}'"
        );
    }
}

#[test]
fn every_locked_field_is_refused_with_a_reason() {
    // The manual lists what cannot change from the browser live, from
    // `form::LOCKED` and `form::structural`. Two lists that must agree.
    for f in vleo_sheet::form::LOCKED {
        assert!(
            vleo_sheet::form::structural(f).is_some(),
            "'{f}' is listed as locked and the form does not refuse it"
        );
        assert!(
            vleo_sheet::form::field(f).is_none(),
            "'{f}' is listed as locked and is also a field the form writes"
        );
    }
}

// ── the checks, proved to refuse ─────────────────────────────────────────────

#[test]
fn the_checks_refuse_what_they_exist_to_refuse() {
    let k = known();
    // A command that does not exist, in each program, in each form.
    for bad in [
        "cargo run -p xtask -- nosuch",
        "cargo xtask nosuch",
        "cargo run -p vleo-cli --bin vleo -- nosuch",
        "cargo vleo nosuch",
        "cargo test -p vleo-mod-nosuch",
        "python3 tools/nosuch.py",
        "curl -s http://127.0.0.1:7777/v1/nosuch",
        "VLEO_NOSUCH=1 cargo run --release -p vleo-daemon",
        "rm -rf /",
    ] {
        assert!(command_ok(bad, &k).is_err(), "`{bad}` was accepted");
    }
    // And the real ones pass, so the refusals above are about the names.
    for good in [
        "cargo run -p xtask -- status",
        "cargo xtask gate && cargo test",
        "cargo run -p vleo-cli --bin vleo -- run sw_ap_design",
        "VLEO_ALLOW_WRITE=1 cargo run --release -p vleo-daemon",
        "curl -s 'http://127.0.0.1:7777/v1/run?node=sw_ap_design'",
        "curl -s http://127.0.0.1:7777/v1/declare/sw_ap_design",
    ] {
        command_ok(good, &k).unwrap_or_else(|e| panic!("`{good}` was refused: {e}"));
    }

    let a: BTreeSet<String> = ["x".into(), "y".into()].into();
    let b: BTreeSet<String> = ["y".into(), "z".into()].into();
    let e = same("letters", &a, &b).unwrap_err();
    assert!(
        e.contains("\"x\"") && e.contains("\"z\""),
        "both sides must be named: {e}"
    );
}

#[test]
fn a_malformed_manual_is_refused_by_name() {
    let ok = "[[layer]]\nid=\"l\"\ntitle=\"L\"\n[[layer.section]]\nid=\"s\"\ntitle=\"S\"\nwho=\"user\"\n";
    manual::parse(ok).expect("the minimal manual loads");
    for (bad, says) in [
        // A command whose check is not stated.
        (format!("{ok}[[layer.section.step]]\nsay=\"x\"\nrun=\"cargo xtask status\"\n"), "no `check`"),
        // A command with a placeholder, claiming to be runnable.
        (
            format!("{ok}[[layer.section.step]]\nsay=\"x\"\nrun=\"cargo xtask declare <row>\"\ncheck=\"exits\"\n"),
            "placeholder",
        ),
        // A command that writes, with nothing said about what.
        (format!("{ok}[[layer.section.step]]\nsay=\"x\"\nrun=\"cargo xtask setup\"\ncheck=\"writes\"\n"), "why"),
        // A "cannot" with nowhere to go instead.
        (format!("{ok}[[cannot]]\nplace=\"browser\"\nwhat=\"x\"\nwhy=\"y\"\nwho=\"user\"\n"), "instead"),
        // Two sections one link cannot tell apart.
        (format!("{ok}[[layer.section]]\nid=\"s\"\ntitle=\"T\"\nwho=\"user\"\n"), "twice"),
        // A reader nobody is.
        (ok.replace("who=\"user\"", "who=\"manager\""), "who"),
    ] {
        match manual::parse(&bad) {
            Ok(_) => panic!("accepted a manual that should refuse ({says})"),
            Err(e) => assert!(e.contains(says), "refused, but not for the reason ({says}): {e}"),
        }
    }
}
