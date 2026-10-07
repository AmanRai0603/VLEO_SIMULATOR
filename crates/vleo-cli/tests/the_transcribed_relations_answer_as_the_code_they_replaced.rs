//! Every relation that was code, moved into its node's method, answers as the
//! code did: to the bit, and refusing where it refused.
//!
//! docs/PLAN_1_0.md, phase E: each relation still in code is transcribed into
//! a method "held by the parity gate to the code it replaces, to the bit, at
//! every case, both ends of every range and on bad inputs". The code it
//! replaces is gone once the method is the node's relation, so what that code
//! answered was written down first, by that code, in
//! `baseline/transcribed.csv`, as `baseline/today.csv` was for the engine:
//!
//! - each node at today's values: the declared design, run whole;
//! - at each case the node carries;
//! - with each input in turn at both ends of its declared range, at zero,
//!   and not a number, the others at today's values.
//!
//! Both ways the engine runs a method are held to the record: the interpreter,
//! and the translation the build carries as its fast path.
//!
//! The record is written once, from the build that still has the code, and
//! never again to get green:
//!
//!     VLEO_TRANSCRIBED=write cargo test -p vleo-cli --test the_transcribed_relations_answer_as_the_code_they_replaced

mod baseline;

use std::collections::BTreeMap;
use std::fmt::Write as _;

use baseline::root;
use vleo_bus::{Case, RunMode};
use vleo_modules::{opened, Graph, Scratch, COMPILED};

fn record_path() -> std::path::PathBuf {
    root().join("baseline/transcribed.csv")
}

/// Every variable's value in the declared design, run whole with the
/// reference data verified into a store of the test's own.
fn today(graph: &'static Graph) -> BTreeMap<String, f64> {
    let scratch = std::env::temp_dir().join(format!("vleo-transcribed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    let mut store = vleo_data::Store::open(&scratch.join("data"));
    store
        .sync(&vleo_data::Source::Shipped(root().join("bundles")))
        .expect("the shipped bundles verify");
    let case = Case {
        target: graph.nodes[0].id.to_string(),
        mode: RunMode::All,
        data: store.verified_names(),
        ..Default::default()
    };
    let r = graph
        .evaluate(&case, &mut Scratch::for_graph(graph))
        .expect("the declared design runs");
    let _ = std::fs::remove_dir_all(&scratch);
    r.values.iter().map(|v| (v.id.clone(), v.value)).collect()
}

/// The points one node is probed at, each with a label.
fn probes(graph: &Graph, node: usize, at: &BTreeMap<String, f64>) -> Vec<(String, Vec<f64>)> {
    let def = &graph.nodes[node];
    let names: Vec<&str> = def
        .inputs
        .iter()
        .map(|&v| graph.vars[v as usize].id)
        .collect();
    let today: Vec<f64> = names
        .iter()
        .map(|n| at.get(*n).copied().unwrap_or(0.0))
        .collect();
    // Where today's design blocks a row upstream its input reads zero, and
    // the relation is seen mostly refusing. So each is probed as well from
    // the middle of every input's declared range, where it answers.
    let middle: Vec<f64> = def
        .inputs
        .iter()
        .zip(&today)
        .map(|(&v, &t)| {
            let l = graph.vars[v as usize].limit;
            match (l.lower.is_finite(), l.upper.is_finite()) {
                (true, true) if l.lower > 0.0 && l.upper / l.lower > 100.0 => {
                    (l.lower * l.upper).sqrt()
                }
                (true, true) => 0.5 * (l.lower + l.upper),
                _ if t != 0.0 => t,
                (true, false) => l.lower + 1.0,
                (false, true) => l.upper - 1.0,
                (false, false) => 1.0,
            }
        })
        .collect();
    let mut out = vec![
        ("today".to_string(), today.clone()),
        ("the middle of every range".to_string(), middle.clone()),
    ];
    for f in def.fixtures {
        out.push((format!("case «{}»", f.label), f.inputs.to_vec()));
    }
    for (from, base) in [("today", &today), ("the middle", &middle)] {
        for (k, &v) in def.inputs.iter().enumerate() {
            let limit = graph.vars[v as usize].limit;
            for (end, x) in [
                ("lower end", limit.lower),
                ("upper end", limit.upper),
                ("zero", 0.0),
                ("not a number", f64::NAN),
            ] {
                if x.is_infinite() {
                    continue;
                }
                let mut p = base.clone();
                p[k] = x;
                out.push((format!("{} at {end}, from {from}", names[k]), p));
            }
        }
    }
    out
}

/// What the node answers at `inputs`: each output as the shortest text that
/// reads back to the same number, or that it refused.
fn answer(graph: &Graph, node: usize, inputs: &[f64]) -> String {
    let mut outs = vec![0.0; graph.nodes[node].outputs.len()];
    let r = match graph.run.get(node).copied().flatten() {
        Some(relation) => relation(inputs, &mut outs),
        None => (graph.dispatch[node])(inputs, &mut outs),
    };
    match r {
        Ok(()) => outs
            .iter()
            .map(|v| format!("{v:?}"))
            .collect::<Vec<_>>()
            .join(";"),
        Err(_) => "refused".into(),
    }
}

fn record(graph: &'static Graph, nodes: &[String]) -> String {
    let at = today(graph);
    let mut out = String::from("node,probe,inputs,answer\n");
    for id in nodes {
        let i = graph
            .nodes
            .iter()
            .position(|n| n.id == id)
            .unwrap_or_else(|| panic!("{id} is no longer a row"));
        for (label, inputs) in probes(graph, i, &at) {
            let ins: Vec<String> = inputs.iter().map(|v| format!("{v:?}")).collect();
            let _ = writeln!(
                out,
                "{id},\"{}\",{},{}",
                label.replace('"', "\"\""),
                ins.join(";"),
                answer(graph, i, &inputs)
            );
        }
    }
    out
}

/// The rows the record holds, in its order.
fn recorded_nodes(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines().skip(1) {
        let id = line.split(',').next().unwrap_or_default().to_string();
        if out.last() != Some(&id) {
            out.push(id);
        }
    }
    out
}

/// Where `now` answers otherwise than the record, line by line: what the one
/// allowed difference is, and every other.
///
/// The one allowed: an input that is not a number, where the code answered
/// anyway and the method refuses. A method refuses such an input at its
/// door; the code let some through and answered over them (a count of
/// satellites that is not a number cut to none, a `max` that dropped it),
/// which is a substitution (AGENTS.md, rule 3). In a run it cannot happen
/// either way: every row's answer is refused unless it is a finite number
/// before anything reads it.
fn differences(was: &str, now: &str) -> (Vec<String>, Vec<String>) {
    let (mut allowed, mut other) = (Vec::new(), Vec::new());
    for (x, y) in was.lines().zip(now.lines()) {
        if x == y {
            continue;
        }
        let (xa, ya) = (x.rsplit(',').next(), y.rsplit(',').next());
        let same_probe = x.rsplit_once(',').map(|p| p.0) == y.rsplit_once(',').map(|p| p.0);
        if same_probe
            && x.contains(" at not a number,")
            && xa != Some("refused")
            && ya == Some("refused")
        {
            allowed.push(x.split(',').next().unwrap_or_default().to_string());
        } else {
            other.push(format!("  on record  {x}\n  now        {y}"));
        }
    }
    if was.lines().count() != now.lines().count() {
        other.push(format!(
            "  {} line(s) on record, {} now",
            was.lines().count(),
            now.lines().count()
        ));
    }
    (allowed, other)
}

#[test]
fn the_transcribed_relations_answer_as_the_code_they_replaced() {
    let path = record_path();
    if std::env::var("VLEO_TRANSCRIBED").as_deref() == Ok("write") {
        // The rows whose relation is still code, recorded by that code.
        let tree = vleo_sheet::load::load_all(&root()).expect("the design loads");
        let nodes: Vec<String> = COMPILED
            .nodes
            .iter()
            .filter(|n| {
                tree.sheets
                    .get(n.id)
                    .is_some_and(|s| s.behaviour() == "built-in")
            })
            .map(|n| n.id.to_string())
            .collect();
        std::fs::write(&path, record(&COMPILED, &nodes)).unwrap();
        return;
    }
    let was = std::fs::read_to_string(&path).expect("baseline/transcribed.csv is on record");
    // A row is held to the code it replaced while its method is still that
    // transcription. A method its group writes in its place is its own, held
    // by its own cases and by today's answers on record, not by the code.
    let tree = vleo_sheet::load::load_all(&root()).expect("the design loads");
    let still = |id: &str| {
        tree.sheets
            .get(id)
            .is_some_and(|s| !s.method.transcribed_from.is_empty())
    };
    let nodes: Vec<String> = recorded_nodes(&was)
        .into_iter()
        .filter(|id| still(id))
        .collect();
    assert!(!nodes.is_empty());
    let was: String = was
        .lines()
        .enumerate()
        .filter(|(n, l)| *n == 0 || still(l.split(',').next().unwrap_or_default()))
        .map(|(_, l)| format!("{l}\n"))
        .collect();

    let interpreted = opened::read_interpreting(&root()).expect("the design's files make a graph");
    let mut said = String::new();
    for (how, graph) in [
        ("the interpreter", interpreted),
        ("the translation", &COMPILED),
    ] {
        let (allowed, other) = differences(&was, &record(graph, &nodes));
        if !other.is_empty() {
            let shown: Vec<&String> = other.iter().take(20).collect();
            let _ =
                writeln!(
                said,
                "{how} answers otherwise than the code each method replaced, at {} point(s):\n{}",
                other.len(),
                shown.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")
            );
        }
        // The allowed difference stays the exception it is.
        assert!(
            allowed.len() < nodes.len(),
            "{how}: {} not-a-number refusals is not an exception",
            allowed.len()
        );
    }
    assert!(said.is_empty(), "{said}");
}

#[test]
fn no_relation_of_the_design_is_left_in_code() {
    let tree = vleo_sheet::load::load_all(&root()).expect("the design loads");
    let left: Vec<&str> = tree
        .sheets
        .values()
        .filter(|s| s.behaviour() == "built-in")
        .map(|s| s.id.as_str())
        .collect();
    assert!(
        left.is_empty(),
        "{} row(s) still have their relation in code: {}",
        left.len(),
        left.join(", ")
    );
}
