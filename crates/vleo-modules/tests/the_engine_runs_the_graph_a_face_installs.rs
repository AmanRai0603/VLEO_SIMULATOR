//! The engine runs the graph a face installs from the design's files, and no
//! other: every function a face calls answers from it, and a graph the faces
//! could not name correctly is refused before it runs.
//!
//! One test, run alone in its own binary, because which graph runs is the one
//! setting the engine shares across a process.

#![cfg(feature = "std")]

use std::path::PathBuf;

use vleo_bus::{Case, RunMode};
use vleo_modules::core_engine::graph::Kind;
use vleo_modules::{engine, opened, run_compiled, run_on, Scratch, COMPILED};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn alone(id: &str) -> Case {
    Case {
        target: id.to_string(),
        mode: RunMode::All,
        ..Default::default()
    }
}

#[test]
fn the_engine_runs_the_installed_graph_and_refuses_one_laid_out_otherwise() {
    assert!(
        std::ptr::eq(engine(), &COMPILED),
        "before a face installs one"
    );

    // A built-in row that answers, in a design whose copy of it has moved on:
    // the same layout, one relation this build does not have.
    let mut tree = vleo_sheet::load_all(&root()).unwrap();
    // Read with every method interpreted, so a row that runs as compiled code
    // here is one whose relation is built in.
    let read = opened::interpreting(&tree).unwrap();
    let id = read
        .nodes
        .iter()
        .enumerate()
        .find(|(k, d)| {
            d.kind == Kind::Computed
                && !matches!(read.run.get(*k), Some(Some(_)))
                && vleo_modules::evaluate(&alone(d.id), &mut Scratch::new())
                    .is_ok_and(|r| r.values.iter().any(|v| v.id == d.id))
        })
        .map(|(_, d)| d.id.to_string())
        .expect("a built-in row that answers");
    tree.sheets.get_mut(&id).unwrap().impl_hash ^= 1;
    let moved = opened::graph(&tree).unwrap();

    // Installed, every free function runs on it: the row is refused by name.
    run_on(moved).expect("a graph laid out as this build's");
    assert!(std::ptr::eq(engine(), moved));
    let r = vleo_modules::evaluate(&alone(&id), &mut Scratch::new()).unwrap();
    assert!(
        r.blocked.iter().any(|b| b.id == id
            && b.message
                .contains("its relation is not in this build of the engine")),
        "{id} ran on the compiled graph after a face installed another: {:#?}",
        r.values
    );
    assert!(!r.values.iter().any(|v| v.id == id));

    // Compiled again, it answers again.
    run_compiled();
    assert!(std::ptr::eq(engine(), &COMPILED));
    let r = vleo_modules::evaluate(&alone(&id), &mut Scratch::new()).unwrap();
    assert!(r.values.iter().any(|v| v.id == id));

    // A design with a row fewer is laid out otherwise: refused, by what
    // differs, and the engine goes on running what it ran.
    let mut short = vleo_sheet::load_all(&root()).unwrap();
    let last = short.ordered().last().unwrap().id.clone();
    short.sheets.remove(&last);
    let fewer = opened::graph(&short).unwrap();
    let e = run_on(fewer).expect_err("a graph laid out otherwise was installed");
    let said = e.to_string();
    assert!(
        said.contains("laid out otherwise than this build of the engine")
            && said.contains(&format!(
                "{} rows, where this build has {}",
                COMPILED.nodes.len() - 1,
                COMPILED.nodes.len()
            )),
        "{said}"
    );
    assert!(std::ptr::eq(engine(), &COMPILED));
}
