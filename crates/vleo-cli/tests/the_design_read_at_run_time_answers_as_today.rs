//! The parity gate: the graph read from the design's files when the engine
//! opens answers exactly as the compiled one does.
//!
//! docs/PLAN_1_0.md, phase D: "For every row in every case, the new engine
//! gives today's answer within the row's own tolerance, and refuses where
//! today refuses. There is no switch without it." The tolerance here is none:
//! the record of today's answers (`baseline/today.csv`), written by the same
//! recording from the graph built at run time, must be the record byte for
//! byte — every value, every status and credibility, every refusal and its
//! reason, at both ends of every input's range and on every fixture.
//!
//! And the graph itself is held to the compiled one, node for node, variable
//! for variable and case for case, so a difference shows as the field that
//! differs rather than only as an answer that moved.
//!
//! Every row whose relation is a method runs here in the interpreter, not as
//! the translated code the compiled graph carries: the record of today's
//! answers is the check that the two agree, exactly. (The graph a face runs
//! takes the translation as its fast path for a method the build was made
//! from; this is the gate that holds the two equal.)

mod baseline;

use std::sync::OnceLock;

use baseline::{record_path, root, today};
use vleo_modules::core_engine::graph::Behaviour;
use vleo_modules::{opened, Graph, COMPILED};

fn read() -> &'static Graph {
    static G: OnceLock<&'static Graph> = OnceLock::new();
    G.get_or_init(|| opened::read_interpreting(&root()).expect("the design's files make a graph"))
}

#[test]
fn the_graph_read_at_run_time_is_the_compiled_graph() {
    let g = read();
    assert_eq!(g.nodes.len(), COMPILED.nodes.len());
    for (a, b) in g.nodes.iter().zip(COMPILED.nodes) {
        assert_eq!(format!("{a:?}"), format!("{b:?}"), "node {}", b.id);
    }
    assert_eq!(g.vars.len(), COMPILED.vars.len());
    for (a, b) in g.vars.iter().zip(COMPILED.vars) {
        assert_eq!(format!("{a:?}"), format!("{b:?}"), "variable {}", b.id);
    }
    assert_eq!(g.cases.len(), COMPILED.cases.len());
    for (a, b) in g.cases.iter().zip(COMPILED.cases) {
        assert_eq!(
            (a.id, a.label, a.note, a.supply, a.conditions),
            (b.id, b.label, b.note, b.supply, b.conditions),
            "case {}",
            b.id
        );
        assert_eq!(a.cycles.len(), b.cycles.len(), "case {}", b.id);
        for (x, y) in a.cycles.iter().zip(b.cycles) {
            assert_eq!(
                (x.nodes, x.converge_on, x.tolerance, x.max_iter, x.seeds),
                (y.nodes, y.converge_on, y.tolerance, y.max_iter, y.seeds),
                "a cycle of case {}",
                b.id
            );
        }
    }
    assert_eq!(g.kernel_hash(), COMPILED.kernel_hash());
    assert_eq!(g.graph_hash(), COMPILED.graph_hash());
}

#[test]
fn no_row_runs_compiled_code_but_a_method_s_own_translation() {
    // Counted from the translated methods the compiled engine carries, one
    // file each.
    let translated = std::fs::read_dir(root().join("crates/vleo-core/src/physics/methods"))
        .unwrap()
        .filter(|e| {
            let p = e.as_ref().unwrap().path();
            p.extension().is_some_and(|x| x == "rs") && p.file_stem().is_some_and(|s| s != "mod")
        })
        .count();
    assert!(translated > 0);
    // In the graph the parity gate runs, every method is run by the
    // interpreter. In the graph a face runs, a method this build was made
    // from runs as its translation, the fast path — all of them, here, since
    // this build was made from these very sheets. In both, every stated value
    // is published by the graph itself.
    let faces = opened::read(&root()).unwrap();
    for (g, interpreted) in [(read(), translated), (faces, 0)] {
        let methods = g
            .nodes
            .iter()
            .filter(|d| d.behaviour == Behaviour::Method)
            .count();
        assert_eq!(methods, translated);
        let by_the_graph = |k: usize| g.run.get(k).is_some_and(|r| r.is_some());
        let run_by_the_interpreter = (0..g.nodes.len())
            .filter(|&k| g.nodes[k].behaviour == Behaviour::Method && by_the_graph(k))
            .count();
        assert_eq!(run_by_the_interpreter, interpreted);
        for (k, d) in g.nodes.iter().enumerate() {
            if d.behaviour == Behaviour::Stated && d.inputs.is_empty() {
                assert!(by_the_graph(k), "{} is not published by the graph", d.id);
            }
        }
    }
    assert_eq!(faces.graph_hash(), COMPILED.graph_hash());
}

/// Each stated value, published by the graph, against the row the generator
/// compiled for it: the same value to the bit, or the same fault.
#[test]
fn every_stated_value_is_published_as_the_compiled_row_published_it() {
    let g = read();
    let mut compared = 0;
    for (k, def) in g.nodes.iter().enumerate() {
        if def.behaviour != Behaviour::Stated || !def.inputs.is_empty() {
            continue;
        }
        let c = COMPILED
            .find(def.id)
            .expect("the stated row is compiled in");
        let said = |r: Result<[f64; vleo_modules::MAX_OUTPUTS], _>| match r {
            Ok(v) => format!("{:?}", v[0].to_bits()),
            Err(f) => format!("{f:?}"),
        };
        assert_eq!(
            said(g.probe(k as u16, &[])),
            said(COMPILED.probe(c, &[])),
            "{}",
            def.id
        );
        compared += 1;
    }
    assert!(compared > 100, "only {compared} stated rows compared");
}

#[test]
fn the_graph_read_at_run_time_gives_today_s_answers() {
    let on_record = std::fs::read_to_string(record_path()).expect("baseline/today.csv");
    let now = today(read());
    if now == on_record {
        return;
    }
    let first = on_record
        .lines()
        .zip(now.lines())
        .position(|(a, b)| a != b)
        .map_or("a line count".to_string(), |n| {
            format!(
                "line {}:\n  on record  {}\n  read       {}",
                n + 1,
                on_record.lines().nth(n).unwrap_or(""),
                now.lines().nth(n).unwrap_or("")
            )
        });
    panic!("the graph read from the design's files does not give today's answers — {first}");
}

/// Each method, run by the interpreter, against the translated code the
/// compiled graph carries — at inputs today's record never reaches: each of
/// its fixtures, a value that is not a number in each place, and values far
/// outside every range, so the door and the guards are held as well as the
/// arithmetic. The same answer to the bit, or the same fault in the same words.
#[test]
fn every_method_answers_and_refuses_as_its_translation_does() {
    let g = read();
    let mut compared = 0;
    for (k, def) in g.nodes.iter().enumerate() {
        if !matches!(g.run.get(k), Some(Some(_))) {
            continue;
        }
        let c = COMPILED
            .find(def.id)
            .expect("the method's row is compiled in");
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
            let said = |r: Result<[f64; vleo_modules::MAX_OUTPUTS], _>| match r {
                Ok(v) => format!(
                    "{:?}",
                    &v[..def.outputs.len()]
                        .iter()
                        .map(|x| x.to_bits())
                        .collect::<Vec<_>>()
                ),
                Err(f) => format!("{f:?}"),
            };
            assert_eq!(
                said(g.probe(k as u16, t)),
                said(COMPILED.probe(c, t)),
                "{} at {t:?}",
                def.id
            );
            compared += 1;
        }
    }
    assert!(compared > 32 * 6, "only {compared} comparisons");
}

/// The graph as text, one line per node, variable, case and cycle, and its
/// fingerprint: what `baseline/graph.txt` holds.
fn the_graph(g: &Graph) -> String {
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
fn first_difference(on_record: &str, now: &str) -> String {
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

/// The graph read from the design's files, held to the graph on record
/// (`baseline/graph.txt`) node for node, variable for variable and case for
/// case. The record was written while the compiled graph still existed and
/// was that graph exactly; it outlives it. A deliberate change to the design
/// records it again, with today's answers:
///
///     VLEO_BASELINE=write cargo test -p vleo-cli --test the_design_read_at_run_time_answers_as_today
#[test]
fn the_graph_read_at_run_time_is_the_graph_on_record() {
    let path = root().join("baseline/graph.txt");
    let now = the_graph(read());
    if std::env::var("VLEO_BASELINE").as_deref() == Ok("write") {
        std::fs::write(&path, &now).unwrap();
        return;
    }
    let on_record = std::fs::read_to_string(&path).expect("baseline/graph.txt");
    assert!(
        now == on_record,
        "the graph read from the design's files is not the graph on record — {}",
        first_difference(&on_record, &now)
    );
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
fn methods_record(
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

/// Every method, run by the interpreter, against what its translation
/// answered (`baseline/methods.csv`), to the bit or the same fault. The
/// record was written once, by the translations, while this build still had
/// them, and never again to get green:
///
///     VLEO_METHODS=write cargo test -p vleo-cli --test the_design_read_at_run_time_answers_as_today
#[test]
fn every_method_answers_and_refuses_as_its_translation_did() {
    let g = read();
    let path = root().join("baseline/methods.csv");
    if std::env::var("VLEO_METHODS").as_deref() == Ok("write") {
        let translated = methods_record(g, |k, t| {
            COMPILED.probe(COMPILED.find(g.nodes[k].id).expect("compiled in"), t)
        });
        std::fs::write(&path, translated).unwrap();
        return;
    }
    let on_record = std::fs::read_to_string(&path).expect("baseline/methods.csv");
    assert!(
        on_record.lines().count() > 32 * 6,
        "baseline/methods.csv is too short"
    );
    let now = methods_record(g, |k, t| g.probe(k as u16, t));
    assert!(
        now == on_record,
        "a method does not answer as its translation did — {}",
        first_difference(&on_record, &now)
    );
}
