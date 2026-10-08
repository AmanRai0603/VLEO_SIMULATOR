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
//! The interpreter, which is how every face runs a method, is held to that
//! record. It was written once, by the build that still had the code; that
//! code is gone, so it can never be written again, and is never rewritten to
//! get green.

mod baseline;

use std::fmt::Write as _;

use baseline::root;
use vleo_modules::{opened, Graph};

fn record_path() -> std::path::PathBuf {
    root().join("baseline/transcribed.csv")
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

/// The record's every line answered again by `graph`, at the inputs the line
/// holds. The inputs are the record's, not probes made again from today's
/// design: a release taken in moves what the rest of the design answers, and
/// a relation is held to the code it replaced at the points that code was
/// asked, whatever else has changed.
fn replay(graph: &Graph, was: &str) -> String {
    let mut out = String::from("node,probe,inputs,answer\n");
    for line in was.lines().skip(1) {
        let (id, rest) = line.split_once(',').expect("a line names its row");
        // The probe's label is quoted, and may hold commas.
        let close = rest[1..]
            .match_indices('"')
            .map(|(i, _)| i + 1)
            .find(|&i| rest[i + 1..].starts_with(','))
            .expect("a probe's label is closed");
        let label = &rest[..=close];
        let (ins, _) = rest[close + 2..]
            .rsplit_once(',')
            .expect("a line holds inputs and an answer");
        let inputs: Vec<f64> = ins
            .split(';')
            .filter(|v| !v.is_empty())
            .map(|v| v.parse().expect("an input on record is a number"))
            .collect();
        let i = graph
            .nodes
            .iter()
            .position(|n| n.id == id)
            .unwrap_or_else(|| panic!("{id} is no longer a row"));
        let _ = writeln!(out, "{id},{label},{ins},{}", answer(graph, i, &inputs));
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

    let graph = opened::read(&root()).expect("the design's files make a graph");
    let (allowed, other) = differences(&was, &replay(graph, &was));
    let shown: Vec<&str> = other.iter().take(20).map(|s| s.as_str()).collect();
    assert!(
        other.is_empty(),
        "the interpreter answers otherwise than the code each method replaced, at {} point(s):\n{}",
        other.len(),
        shown.join("\n")
    );
    // The allowed difference stays the exception it is.
    assert!(
        allowed.len() < nodes.len(),
        "{} not-a-number refusals is not an exception",
        allowed.len()
    );
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
