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

mod baseline;

use std::sync::OnceLock;

use baseline::{record_path, root, today};
use vleo_modules::{opened, Graph, COMPILED};

fn read() -> &'static Graph {
    static G: OnceLock<&'static Graph> = OnceLock::new();
    G.get_or_init(|| opened::read(&root()).expect("the design's files make a graph"))
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
