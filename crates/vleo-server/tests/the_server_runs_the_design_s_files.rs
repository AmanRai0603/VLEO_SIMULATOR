//! The server runs the graph read from the design's files — the folders of
//! this checkout, here — not the graph compiled into it. Every route answers
//! from that graph; the contract and the record of today's answers hold it to
//! today's answers.

use std::path::{Path, PathBuf};

use vleo_modules::{engine, COMPILED};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_server_runs_the_graph_read_from_the_design_s_files() {
    let scratch = std::env::temp_dir().join(format!("vleo-server-engine-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    std::env::remove_var("VLEO_DESIGN");

    assert!(std::ptr::eq(engine(), &COMPILED));
    vleo_server::serve(Some(root()), 18951, false, true).expect("the server did not start");
    let g = engine();
    assert!(
        !std::ptr::eq(g, &COMPILED),
        "the server runs the compiled graph"
    );
    assert_eq!(g.nodes.len(), COMPILED.nodes.len());
    // This build was made from these very sheets, so every method runs as
    // its translation, the fast path; a method a release changes is the one
    // the interpreter runs (today_s_design_is_built_from_the_releases).
    assert_eq!(g.run_by_the_graph(), 0);
    assert_eq!(g.graph_hash(), COMPILED.graph_hash());
}
