//! The record of today's answers, written from any graph the engine is given:
//! the compiled one (`today_s_answers_are_on_record`) and the one read from
//! the design's files (`the_design_read_at_run_time_answers_as_today`). One
//! recording, so the two are compared by the same code.

#![allow(dead_code)]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use vleo_bus::{Case, RunMode};
use vleo_modules::core_engine::graph::{Behaviour, Kind};
use vleo_modules::{Graph, Scratch};

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn record_path() -> PathBuf {
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

fn whole(graph: &Graph, supply: Vec<(String, f64)>, data: &[String]) -> Case {
    Case {
        target: graph.nodes[0].id.to_string(),
        mode: RunMode::All,
        supply,
        data: data.to_vec(),
        ..Default::default()
    }
}

/// One run, every row of it, in the engine's own order, as `(what, id)` and
/// the rest of the line.
fn run(graph: &'static Graph, case: &Case, only_computed: bool) -> Vec<((String, String), String)> {
    let mut out = Vec::new();
    let r = match graph.evaluate(case, &mut Scratch::for_graph(graph)) {
        Err(f) => {
            let why = field(&vleo_bus::fault_message(&f));
            out.push((("refused".into(), String::new()), format!(",,{why}")));
            return out;
        }
        Ok(r) => r,
    };
    for v in &r.values {
        if only_computed {
            let declared = graph
                .find(&v.id)
                .map(|i| graph.nodes[i as usize].kind == Kind::Declared)
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

/// The whole record, as `graph` answers it today.
pub fn today(graph: &'static Graph) -> String {
    let scratch = std::env::temp_dir().join(format!("vleo-baseline-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    let with = data(&scratch);

    let mut out = String::from("run,what,id,value,status,detail\n");
    for (name, supply) in cases() {
        for (with_data, data) in [("with data", with.as_slice()), ("no data", &[])] {
            let name = format!("{name} · {with_data}");
            for (k, rest) in run(graph, &whole(graph, supply.clone(), data), false) {
                write(&mut out, &name, &k, &rest);
            }
        }
    }
    // A range end is recorded by what it changes from the declared design:
    // every line that differs, and every row that answers there and not at
    // the declared values, or the other way round. Anything a new engine
    // moves at a range end therefore still shows, as a line that was not here.
    let declared: std::collections::BTreeMap<_, _> =
        run(graph, &whole(graph, Vec::new(), &with), true)
            .into_iter()
            .collect();
    for input in vleo_modules::inputs::case_inputs() {
        for (end, v) in [("lower", input.lo), ("upper", input.hi)] {
            let name = format!("{} at its {end} end · with data", input.id);
            let here = run(
                graph,
                &whole(graph, vec![(input.id.to_string(), v)], &with),
                true,
            );
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
    for (i, def) in graph.nodes.iter().enumerate() {
        for f in graph.fixture_verdicts(i as u16) {
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

/// The graph as text, one line per node, variable, case and cycle, and its
/// fingerprint: what `baseline/graph.txt` holds.
pub fn the_graph(g: &Graph) -> String {
    let mut s = String::new();
    for n in g.nodes {
        s += &format!("node {n:?}\n");
    }
    for v in g.vars {
        s += &format!("variable {v:?}\n");
    }
    for c in g.cases {
        s += &format!(
            "case {:?}\n",
            (c.id, c.label, c.note, c.supply, c.conditions)
        );
        for y in c.cycles {
            s += &format!(
                "cycle {:?}\n",
                (y.nodes, y.converge_on, y.tolerance, y.max_iter, y.seeds)
            );
        }
    }
    s += &format!("graph {:016x}\n", g.graph_hash());
    s
}

/// The first line two records differ at, said for a person.
pub fn first_difference(on_record: &str, now: &str) -> String {
    on_record
        .lines()
        .zip(now.lines())
        .position(|(a, b)| a != b)
        .map_or("a line count".to_string(), |n| {
            format!(
                "line {}:\n  on record  {}\n  now        {}",
                n + 1,
                on_record.lines().nth(n).unwrap_or(""),
                now.lines().nth(n).unwrap_or("")
            )
        })
}

/// What a method answers at one set of inputs, as `baseline/methods.csv`
/// holds it: the inputs and the answer to the bit, or the fault in its words.
fn method_line(
    id: &str,
    t: &[f64],
    outputs: usize,
    r: Result<[f64; vleo_modules::MAX_OUTPUTS], vleo_modules::core_engine::fault::Fault>,
) -> String {
    let bits = |v: &[f64]| {
        v.iter()
            .map(|x| format!("{:016x}", x.to_bits()))
            .collect::<Vec<_>>()
            .join(";")
    };
    let said = match r {
        Ok(v) => bits(&v[..outputs]),
        Err(f) => format!("{f:?}"),
    };
    format!("{id},{},{}\n", bits(t), said.replace('\n', " "))
}

/// Every method's answer at inputs today's record never reaches, as the
/// translated code answered it: each of its fixtures, and each input in turn
/// not a number, infinite, far outside every range, zero and negative, the
/// rest at its first fixture.
pub fn methods_record(
    g: &Graph,
    answer: impl Fn(
        usize,
        &[f64],
    )
        -> Result<[f64; vleo_modules::MAX_OUTPUTS], vleo_modules::core_engine::fault::Fault>,
) -> String {
    let mut s = String::from("node,inputs,answer\n");
    for (k, def) in g.nodes.iter().enumerate() {
        if def.behaviour != Behaviour::Method {
            continue;
        }
        let n = def.inputs.len();
        let mut tries: Vec<Vec<f64>> = def.fixtures.iter().map(|f| f.inputs.to_vec()).collect();
        let base = tries.first().cloned().unwrap_or_else(|| vec![1.0; n]);
        for i in 0..n {
            for bad in [f64::NAN, f64::INFINITY, -1e12, 1e12, 0.0, -1.0] {
                let mut t = base.clone();
                t[i] = bad;
                tries.push(t);
            }
        }
        for t in &tries {
            s += &method_line(def.id, t, def.outputs.len(), answer(k, t));
        }
    }
    s
}
