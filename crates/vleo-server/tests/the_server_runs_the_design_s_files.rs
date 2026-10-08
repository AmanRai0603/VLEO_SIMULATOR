//! The server runs the graph read from the design's files — the folders of
//! this checkout, here — and before it opens them, the engine holds nothing. Every route answers
//! from that graph; the contract and the record of today's answers hold it to
//! today's answers.

use std::path::{Path, PathBuf};

use vleo_modules::{engine, EMPTY};

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

    assert!(std::ptr::eq(engine(), &EMPTY));
    vleo_server::serve(Some(root()), 18951, false, true).expect("the server did not start");
    let g = engine();
    assert!(!std::ptr::eq(g, &EMPTY), "the server opened no design");
    // The design's files hold every row the sheets hold, and the open blocks
    // they propose where a breakdown holds none yet.
    let sheets = vleo_sheet::load_all(&root()).unwrap();
    for id in sheets.sheets.keys() {
        assert!(g.find(id).is_some(), "{id} is not in the design");
    }
    for d in g.nodes.iter().filter(|d| !sheets.sheets.contains_key(d.id)) {
        assert_eq!(
            d.behaviour,
            vleo_modules::core_engine::graph::Behaviour::Open,
            "{} is not open",
            d.id
        );
    }
    // Every stated value is published by the graph, and every method is run
    // by the interpreter: no row runs the code compiled for it.
    use vleo_modules::core_engine::graph::Behaviour;
    for (k, d) in g.nodes.iter().enumerate() {
        if d.behaviour == Behaviour::Stated && d.inputs.is_empty() {
            assert!(g.run[k].is_some(), "{} is not published by the graph", d.id);
        }
        if d.behaviour == Behaviour::Method {
            assert!(g.run[k].is_some(), "{} is not run by the interpreter", d.id);
        }
    }
}
