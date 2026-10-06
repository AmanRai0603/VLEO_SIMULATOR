//! A row whose relation this build of the engine does not have is refused by
//! name, never run as something else.
//!
//! The graph read from the design's files finds each row's relation by its id,
//! and takes the compiled one only when it is that sheet's — its implementation
//! the one the sheet was built with. A sheet whose implementation has moved on
//! (an edit the build has not seen), or a row the build has never heard of,
//! must stop at that row, and everything downstream of it must say so.

#![cfg(feature = "std")]

use std::path::PathBuf;

use vleo_bus::{Case, RunMode};
use vleo_modules::core_engine::graph::Kind;
use vleo_modules::{opened, Scratch};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn a_relation_built_from_another_sheet_is_refused_by_name() {
    let mut tree = vleo_sheet::load_all(&root()).unwrap();
    // A computed row that answers today, on the compiled engine …
    let all = Case {
        target: vleo_modules::NODES[0].id.to_string(),
        mode: RunMode::All,
        ..Default::default()
    };
    let today = vleo_modules::evaluate(&all, &mut Scratch::new()).unwrap();
    let id = today
        .values
        .iter()
        .map(|v| v.id.clone())
        .find(|v| {
            vleo_modules::Vleo::find(v)
                .is_some_and(|k| vleo_modules::NODES[k as usize].kind == Kind::Computed)
        })
        .expect("a computed row that answers");
    // … whose implementation has since changed.
    tree.sheets.get_mut(&id).unwrap().impl_hash ^= 1;
    let g = opened::graph(&tree).unwrap();
    let k = g.find(&id).unwrap();
    assert_eq!(g.nodes[k as usize].kind, Kind::Computed);
    let case = Case {
        target: id.clone(),
        mode: RunMode::All,
        ..Default::default()
    };
    let r = g.evaluate(&case, &mut Scratch::for_graph(g)).unwrap();
    let refused = r.blocked.iter().find(|b| b.id == id).unwrap_or_else(|| {
        panic!(
            "{id} ran on a relation that is not its own: {:#?}",
            r.values
        )
    });
    assert!(
        refused
            .message
            .contains("its relation is not in this build of the engine"),
        "{}",
        refused.message
    );
    assert!(!r.values.iter().any(|v| v.id == id));
}
