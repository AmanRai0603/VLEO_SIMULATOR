//! The parity gate, on the design converted to its files: the engine run on
//! the graph read from the group and node files gives today's answers, byte
//! for byte (docs/PLAN_1_0.md, phase E: "the converted design passes the
//! parity gate").
//!
//! The record of today's answers (`baseline/today.csv`) is made by the same
//! recording, from the graph read through the files' inverse with every
//! method in the interpreter. The blocks the conversion adds where a
//! breakdown holds none are open and on record with the rest; what they add
//! is held on its own too: each is a row that is not run, by its own id, and
//! nothing else moves.

mod baseline;

use std::sync::{Arc, OnceLock};

use baseline::{record_path, root, today};
use vleo_files::convert::{self, proposed_id, Served, PROPOSED};
use vleo_files::model::File;
use vleo_modules::{opened, Graph};
use vleo_sheet::files::Disk;

fn proposed() -> Vec<String> {
    PROPOSED
        .iter()
        .flat_map(|(g, names)| names.iter().map(move |(n, _)| proposed_id(g, n)))
        .collect()
}

fn files() -> &'static [(String, File)] {
    static FILES: OnceLock<Vec<(String, File)>> = OnceLock::new();
    FILES.get_or_init(|| {
        let tree = vleo_sheet::load::load_all(&root()).expect("the design loads");
        convert::convert(&tree, &Disk, "vleo test").expect("the design converts")
    })
}

/// The graph the engine reads from `files`, every method interpreted.
fn graph_of(files: &[(String, File)]) -> &'static Graph {
    let served = Served::new(&root(), files, Arc::new(Disk)).expect("the files are served");
    let tree = vleo_sheet::load::load_all_from(&served, &root()).expect("the files load");
    opened::graph(&tree).expect("the files make a graph")
}

fn first_difference(on_record: &str, now: &str) -> String {
    on_record
        .lines()
        .zip(now.lines())
        .position(|(a, b)| a != b)
        .map_or(
            format!(
                "a line count: {} on record, {} now",
                on_record.lines().count(),
                now.lines().count()
            ),
            |n| {
                format!(
                    "line {}:\n  on record  {}\n  converted  {}",
                    n + 1,
                    on_record.lines().nth(n).unwrap_or(""),
                    now.lines().nth(n).unwrap_or("")
                )
            },
        )
}

/// The sheets, converted to their files, give today's answers, the open
/// blocks the conversion proposes with them: today's answers are recorded
/// from the design's files (`today_s_answers_are_on_record`).
#[test]
fn the_sheets_converted_give_today_s_answers() {
    let on_record = std::fs::read_to_string(record_path()).expect("baseline/today.csv");
    let now = today(graph_of(files()));
    assert!(
        now == on_record,
        "the converted design does not give today's answers — {}",
        first_difference(&on_record, &now)
    );
}

#[test]
fn the_blocks_added_are_open_and_move_nothing_else() {
    let g = graph_of(files());
    for id in proposed() {
        let k = g
            .find(&id)
            .unwrap_or_else(|| panic!("{id} is not in the graph"));
        assert_eq!(g.nodes[k as usize].behaviour.name(), "open", "{id}");
        // Nothing reads them: an open block no row depends on.
        assert!(
            g.nodes
                .iter()
                .all(|n| n.inputs.iter().all(|&v| g.vars[v as usize].id != id)),
            "{id} is read"
        );
    }
}
