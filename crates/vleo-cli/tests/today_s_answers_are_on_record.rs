//! Today's answers, on record before the engine changes underneath them.
//!
//! 1.0 moves the graph out of the compiled code and into the design file
//! (`docs/PLAN_1_0.md`, phase D). Its parity gate asks one question: does the
//! new engine give, for every row in every case, the answer this one gives —
//! and refuse where this one refuses, for the same reason? That question has no
//! answer unless this one's answers were written down first, by this engine,
//! before anything it measures moved. This file is that record, and this test
//! holds the engine to it exactly.
//!
//! What is recorded, in `baseline/today.csv`:
//! - every row of the whole design, run on each shared case, with the reference
//!   data and without it: its value as the shortest text that reads back to the
//!   same f64, its status and its credibility; and every refusal, by kind and
//!   message;
//! - every input of the case at both ends of its declared range, the rest at
//!   their defaults, by what it changes from the declared design: each row
//!   that is not a declared value and answers differently there, and each
//!   refusal that can move with an input (whether a row is written or
//!   confirmed cannot, and is recorded once per case above);
//! - every fixture each row carries, as the engine computes it today.
//!
//! A difference is never re-recorded to get green. It is either an engine
//! change that moved an answer, which is a defect, or a deliberate change to
//! the design, whose answers are recorded again on purpose:
//!
//!     VLEO_BASELINE=write cargo test -p vleo-cli --test today_s_answers_are_on_record

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use vleo_bus::{Case, RunMode};
use vleo_modules::core_engine::graph::Kind;
use vleo_modules::{evaluate, Scratch, Vleo, NODES};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn record_path() -> PathBuf {
    root().join("baseline/today.csv")
}

/// A field as CSV writes it: quoted when it holds a comma, a quote or a line.
fn field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// The shared cases: the design as declared, and each example a person can
/// copy, read from the file exactly as the command line reads it.
fn cases() -> Vec<(String, Vec<(String, f64)>)> {
    let mut out = vec![("declared".to_string(), Vec::new())];
    let dir = root().join("cases/examples");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "csv"))
        .collect();
    files.sort();
    for f in files {
        let r = vleo_modules::inputs::read_csv(&std::fs::read_to_string(&f).unwrap());
        assert!(r.ok(), "{} no longer reads: {:?}", f.display(), r.refused);
        out.push((f.file_stem().unwrap().to_string_lossy().into_owned(), r.set));
    }
    out
}

fn whole(supply: Vec<(String, f64)>, data: &[String]) -> Case {
    Case {
        target: NODES[0].id.to_string(),
        mode: RunMode::All,
        supply,
        data: data.to_vec(),
        ..Default::default()
    }
}

/// One run, every row of it, in the engine's own order, as `(what, id)` and
/// the rest of the line.
fn run(case: &Case, only_computed: bool) -> Vec<((String, String), String)> {
    let mut out = Vec::new();
    let r = match evaluate(case, &mut Scratch::new()) {
        Err(f) => {
            let why = field(&vleo_bus::fault_message(&f));
            out.push((("refused".into(), String::new()), format!(",,{why}")));
            return out;
        }
        Ok(r) => r,
    };
    for v in &r.values {
        if only_computed {
            let declared = Vleo::find(&v.id)
                .map(|i| NODES[i as usize].kind == Kind::Declared)
                .unwrap_or(false);
            if declared {
                continue;
            }
        }
        let cred: String = v.cred.0.iter().map(|d| char::from(b'0' + d)).collect();
        out.push((
            ("value".into(), v.id.clone()),
            format!("{:?},{:?},{cred}", v.value, v.status),
        ));
    }
    for b in &r.blocked {
        // Whether a row is written, and confirmed, does not move with the
        // inputs; the whole-design runs record it once per case.
        if only_computed && matches!(b.kind, "not-run" | "undefined") {
            continue;
        }
        out.push((
            ("blocked".into(), b.id.clone()),
            format!("{},,{}", b.kind, field(&b.message)),
        ));
    }
    out.push((
        ("counted".into(), String::new()),
        format!(
            "{} ran,{} blocked,{} iterations",
            r.manifest.ran, r.manifest.blocked_count, r.manifest.iterations
        ),
    ));
    out
}

fn write(out: &mut String, name: &str, (what, id): &(String, String), rest: &str) {
    let _ = writeln!(out, "{},{what},{id},{rest}", field(name));
}

/// The reference data, verified into a store of the test's own.
fn data(scratch: &Path) -> Vec<String> {
    let mut store = vleo_data::Store::open(&scratch.join("data"));
    store
        .sync(&vleo_data::Source::Shipped(root().join("bundles")))
        .expect("the shipped bundles verify");
    let names = store.verified_names();
    assert!(!names.is_empty(), "no bundle verified");
    names
}

fn today() -> String {
    let scratch = std::env::temp_dir().join(format!("vleo-baseline-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    let with = data(&scratch);

    let mut out = String::from("run,what,id,value,status,detail\n");
    for (name, supply) in cases() {
        for (with_data, data) in [("with data", with.as_slice()), ("no data", &[])] {
            let name = format!("{name} · {with_data}");
            for (k, rest) in run(&whole(supply.clone(), data), false) {
                write(&mut out, &name, &k, &rest);
            }
        }
    }
    // A range end is recorded by what it changes from the declared design:
    // every line that differs, and every row that answers there and not at
    // the declared values, or the other way round. Anything a new engine
    // moves at a range end therefore still shows, as a line that was not here.
    let declared: std::collections::BTreeMap<_, _> =
        run(&whole(Vec::new(), &with), true).into_iter().collect();
    for input in vleo_modules::inputs::case_inputs() {
        for (end, v) in [("lower", input.lo), ("upper", input.hi)] {
            let name = format!("{} at its {end} end · with data", input.id);
            let here = run(&whole(vec![(input.id.to_string(), v)], &with), true);
            for (k, rest) in &here {
                if declared.get(k) != Some(rest) {
                    write(&mut out, &name, k, rest);
                }
            }
            for k in declared.keys() {
                if !here.iter().any(|(h, _)| h == k) {
                    write(&mut out, &name, k, "absent,,");
                }
            }
        }
    }
    for (i, def) in NODES.iter().enumerate() {
        for f in vleo_modules::fixture_verdicts(i as u16) {
            let _ = writeln!(
                out,
                "fixtures,fixture,{},{:?},{},{}",
                def.id,
                f.got,
                if f.passed { "passed" } else { "failed" },
                field(&f.label)
            );
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    out
}

#[test]
fn today_s_answers_are_on_record() {
    let now = today();
    let path = record_path();
    if std::env::var("VLEO_BASELINE").as_deref() == Ok("write") {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &now).unwrap();
        return;
    }
    let was = std::fs::read_to_string(&path).unwrap_or_default();
    if was == now {
        return;
    }
    let mut said = String::new();
    let (a, b): (Vec<&str>, Vec<&str>) = (was.lines().collect(), now.lines().collect());
    for (n, (x, y)) in a.iter().zip(&b).enumerate() {
        if x != y {
            let _ = writeln!(
                said,
                "  line {}:\n    on record  {x}\n    today      {y}",
                n + 1
            );
            if said.lines().count() > 30 {
                break;
            }
        }
    }
    if a.len() != b.len() {
        let _ = writeln!(said, "  {} line(s) on record, {} today", a.len(), b.len());
    }
    panic!(
        "the engine no longer gives the answers on record in baseline/today.csv:\n{said}\
         An engine change that moves an answer is a defect. A deliberate change to the \
         design is recorded again on purpose: VLEO_BASELINE=write cargo test -p vleo-cli \
         --test today_s_answers_are_on_record"
    );
}
