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
    // The design's files hold every compiled row, and the open blocks they
    // propose where a breakdown holds none yet.
    for d in COMPILED.nodes {
        assert!(g.find(d.id).is_some(), "{} is not in the design", d.id);
    }
    for d in g.nodes.iter().filter(|d| COMPILED.find(d.id).is_none()) {
        assert_eq!(
            d.behaviour,
            vleo_modules::core_engine::graph::Behaviour::Open,
            "{} is not open",
            d.id
        );
    }
    // Every stated value is published by the graph; a method this build was
    // made from runs as its translation, the fast path.
    for (k, d) in g.nodes.iter().enumerate() {
        if d.behaviour == vleo_modules::core_engine::graph::Behaviour::Stated && d.inputs.is_empty()
        {
            assert!(g.run[k].is_some(), "{} is not published by the graph", d.id);
        }
    }
}
