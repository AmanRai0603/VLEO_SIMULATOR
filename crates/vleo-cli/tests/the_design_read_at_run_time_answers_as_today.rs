//! The parity gate: the graph read from the design's files when the engine
//! opens answers exactly as the record of today's answers says.
//!
//! docs/PLAN_1_0.md, phase D: "For every row in every case, the new engine
//! gives today's answer within the row's own tolerance, and refuses where
//! today refuses. There is no switch without it." The tolerance here is none:
//! the record of today's answers (`baseline/today.csv`), written by the same
//! recording from the graph built at run time, must be the record byte for
//! byte — every value, every status and credibility, every refusal and its
//! reason, at both ends of every input's range and on every fixture.
//!
//! And the graph itself is held to the graph on record (`baseline/graph.txt`),
//! node for node, variable for variable and case for case, so a difference
//! shows as the field that differs rather than only as an answer that moved;
//! and every method to its answers on record (`baseline/methods.csv`) at
//! inputs today's answers never reach. The records were written while the
//! compiled graph still existed, and were that graph exactly.
//!
//! Every row whose relation is a method runs in the interpreter.

mod baseline;

use std::sync::OnceLock;

use baseline::{first_difference, methods_record, record_path, root, the_graph, today};
use vleo_modules::core_engine::graph::Behaviour;
use vleo_modules::{opened, Graph};

fn read() -> &'static Graph {
    static G: OnceLock<&'static Graph> = OnceLock::new();
    G.get_or_init(|| opened::read(&root()).expect("the design's files make a graph"))
}

#[test]
fn no_row_runs_the_code_compiled_for_it() {
    // Every method is run by the interpreter, and every stated value is
    // published by the graph itself: no row falls back to the code compiled
    // for it.
    let g = read();
    let methods = g
        .nodes
        .iter()
        .filter(|d| d.behaviour == Behaviour::Method)
        .count();
    assert!(methods > 150, "only {methods} methods");
    let by_the_graph = |k: usize| g.run.get(k).is_some_and(|r| r.is_some());
    for (k, d) in g.nodes.iter().enumerate() {
        if d.behaviour == Behaviour::Method {
            assert!(by_the_graph(k), "{} is not run by the interpreter", d.id);
        }
        if d.behaviour == Behaviour::Stated && d.inputs.is_empty() {
            assert!(by_the_graph(k), "{} is not published by the graph", d.id);
        }
    }
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

/// The graph read from the design's files, held to the graph on record
/// (`baseline/graph.txt`) node for node, variable for variable and case for
/// case. It is recorded with today's answers
/// (`today_s_answers_are_on_record`), and the record was written while the
/// compiled graph still existed and was that graph exactly.
#[test]
fn the_graph_read_at_run_time_is_the_graph_on_record() {
    let on_record =
        std::fs::read_to_string(root().join("baseline/graph.txt")).expect("baseline/graph.txt");
    let now = the_graph(read());
    assert!(
        now == on_record,
        "the graph read from the design's files is not the graph on record — {}",
        first_difference(&on_record, &now)
    );
}

/// Every method, run by the interpreter, against what it answers on record
/// (`baseline/methods.csv`), to the bit or the same fault: at its fixtures
/// and at inputs today's answers never reach. The record is written with
/// today's answers, by the graph a face runs, which takes each method's
/// translation while this build has them.
#[test]
fn every_method_answers_and_refuses_as_on_record() {
    let g = read();
    let on_record =
        std::fs::read_to_string(root().join("baseline/methods.csv")).expect("baseline/methods.csv");
    assert!(
        on_record.lines().count() > 32 * 6,
        "baseline/methods.csv is too short"
    );
    let now = methods_record(g, |k, t| g.probe(k as u16, t));
    assert!(
        now == on_record,
        "a method does not answer as on record — {}",
        first_difference(&on_record, &now)
    );
}
