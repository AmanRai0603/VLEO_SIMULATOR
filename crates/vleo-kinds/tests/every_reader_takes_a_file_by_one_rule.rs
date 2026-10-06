//! Every reader takes a file by one rule: the table holds together, today's
//! design is never taken for a released one, and Python's reader carries the
//! same table and gives the same answer for every file.
//!
//!     VLEO_KINDS=write cargo test -p vleo-kinds
//!
//! writes the table into crates/vleo-py/python/vleo/files.py again, after a
//! deliberate change to it.

use std::path::{Path, PathBuf};
use std::process::Command;

use vleo_kinds::{identify, python_table, Reader, Refusal, Store, Want, APPLICATION_ID, KINDS};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const FILES_PY: &str = "crates/vleo-py/python/vleo/files.py";
const START: &str = "# --- kinds: GENERATED";
const END: &str = "# --- end kinds ---\n";

#[test]
fn the_table_holds_together() {
    for (i, a) in KINDS.iter().enumerate() {
        for b in &KINDS[i + 1..] {
            assert!(
                !(a.name == b.name && a.store == b.store),
                "{} is listed twice",
                a.name
            );
        }
        if let Some((name, format)) = a.becomes {
            assert!(
                KINDS
                    .iter()
                    .any(|k| k.name == name && k.format() == Some(format) && k.reader == a.reader),
                "{} becomes {name} {format}, which the table does not list",
                a.name
            );
        }
        if let Store::Database { format } = a.store {
            assert!(format >= 1, "{}", a.name);
        }
    }
}

fn want<'a>(reader: Reader, names: &'a [&'a str], called: &'a str) -> Want<'a> {
    Want {
        reader,
        names,
        called,
    }
}

#[test]
fn today_s_design_is_never_taken_for_a_released_one() {
    // design.vleo, opened as a released design by the library …
    let e = identify(
        APPLICATION_ID,
        1,
        Some("design"),
        want(Reader::Files, &["released design"], "a released design"),
    )
    .unwrap_err();
    assert!(matches!(e, Refusal::OtherKind { .. }));
    assert!(e
        .says("x")
        .contains("a design file, not a released design — today's design"));
    // … and a released design opened as today's design by the tool.
    let e = identify(
        APPLICATION_ID,
        2,
        Some("released design"),
        want(Reader::Design, &["design"], "today's design (design.vleo)"),
    )
    .unwrap_err();
    assert!(e
        .says("x")
        .contains("a released design file, not today's design (design.vleo)"));
    // Each is taken by its own reader.
    assert!(identify(
        APPLICATION_ID,
        1,
        Some("design"),
        want(Reader::Design, &["design"], "")
    )
    .is_ok());
    assert!(identify(
        APPLICATION_ID,
        2,
        Some("released design"),
        want(Reader::Files, &["released design"], "")
    )
    .is_ok());
}

#[test]
fn a_group_s_file_from_before_1_0_is_taken_as_what_it_becomes() {
    let k = identify(
        APPLICATION_ID,
        1,
        Some("release"),
        want(Reader::Files, &["group release"], "a group release"),
    )
    .unwrap();
    assert_eq!(k.becomes, Some(("group release", 2)));
}

/// One file as a reader meets it: (application id, format, kind, reader,
/// names it takes, what it calls them).
type Case = (
    i64,
    i64,
    Option<&'static str>,
    Reader,
    Vec<&'static str>,
    &'static str,
);

/// Every case the parity runs: (application id, format, kind, reader, names
/// it takes, what it calls them).
fn cases() -> Vec<Case> {
    let design = (
        Reader::Design,
        vec!["design"],
        "today's design (design.vleo)",
    );
    let results = (Reader::Design, vec!["results"], "a results file");
    let files: Vec<&'static str> = {
        let mut n: Vec<&'static str> = Vec::new();
        for k in KINDS.iter().filter(|k| k.reader == Reader::Files) {
            if !n.contains(&k.name) {
                n.push(k.name);
            }
        }
        n
    };
    let library = (Reader::Files, files, "a file in the one schema");
    let mut out = Vec::new();
    for (reader, names, called) in [design, results, library] {
        for app in [APPLICATION_ID, 12345] {
            for format in [0, 1, 2, 3, 99] {
                let mut kinds: Vec<Option<&'static str>> = vec![None, Some("spreadsheet")];
                for k in KINDS.iter().filter(|k| k.format().is_some()) {
                    if !kinds.contains(&Some(k.name)) {
                        kinds.push(Some(k.name));
                    }
                }
                for kind in kinds {
                    out.push((app, format, kind, reader, names.clone(), called));
                }
            }
        }
    }
    out
}

fn rust_answer(c: &Case) -> String {
    match identify(c.0, c.1, c.2, want(c.3, &c.4, c.5)) {
        Ok(k) => format!("took {} {}", k.name, k.format().unwrap()),
        Err(r) => r.says("{place}"),
    }
}

#[test]
fn python_carries_the_table() {
    let path = root().join(FILES_PY);
    let text = std::fs::read_to_string(&path).unwrap();
    let a = text.find(START).expect("the generated block's start");
    let b = text[a..].find(END).expect("the generated block's end") + a + END.len();
    let want = python_table();
    if std::env::var("VLEO_KINDS").as_deref() == Ok("write") {
        std::fs::write(&path, format!("{}{want}{}", &text[..a], &text[b..])).unwrap();
        return;
    }
    assert_eq!(
        &text[a..b],
        want,
        "{FILES_PY} does not carry the table as it is: VLEO_KINDS=write cargo test -p vleo-kinds"
    );
}

#[test]
fn python_takes_and_refuses_every_file_as_rust_does() {
    let cases = cases();
    let mut lines = String::new();
    for c in &cases {
        let names =
            c.4.iter()
                .map(|n| format!("{n:?}"))
                .collect::<Vec<_>>()
                .join(",");
        let kind = c.2.map_or("None".to_string(), |k| format!("{k:?}"));
        lines.push_str(&format!(
            "{}\t{}\t{kind}\t[{names}]\t{:?}\t{:?}\n",
            c.0,
            c.1,
            c.5,
            c.3.name()
        ));
    }
    let script = r#"
import importlib.util, sys
spec = importlib.util.spec_from_file_location("files", sys.argv[1])
files = importlib.util.module_from_spec(spec); spec.loader.exec_module(files)
for line in sys.stdin.read().splitlines():
    app, fmt, kind, names, called, reader = line.split("\t")
    k, refused = files.identify(int(app), int(fmt), eval(kind), eval(names), eval(called), eval(reader))
    print(refused if refused else "took %s %d" % (k[0], k[1]))
"#;
    let mut py = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(root().join(FILES_PY))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("python3 is needed to check the Python reader");
    use std::io::Write;
    py.stdin
        .take()
        .unwrap()
        .write_all(lines.as_bytes())
        .unwrap();
    let out = py.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let said = String::from_utf8(out.stdout).unwrap();
    let said: Vec<&str> = said.lines().collect();
    assert_eq!(said.len(), cases.len());
    let mut took = 0;
    for (c, py) in cases.iter().zip(&said) {
        let rust = rust_answer(c);
        took += usize::from(rust.starts_with("took"));
        assert_eq!(*py, rust, "{c:?}");
    }
    // The matrix is not all refusals: each reader takes its own.
    assert!(took >= 3, "only {took} taken");
}
