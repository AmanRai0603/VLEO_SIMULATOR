//! A method run by the interpreter has its answer guarded as the compiled row
//! guards it: below its declared range, then above it, each refused with the
//! sheet's own reason and the bound in SI.
//!
//! Every method in the design today refuses on its own at the bounds its row
//! declares, so no input reaches these guards through it. A method a group
//! writes tomorrow need not, and the guards are what stand between its answer
//! and the design. So here a row's declared range is narrowed, in memory, until
//! its own answer falls outside it, in the graph that interprets every method.

#![cfg(feature = "std")]

use std::path::PathBuf;

use vleo_modules::core_engine::fault::{Edge, Fault};
use vleo_modules::opened;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A row whose relation is a method, with one answer, and the inputs of its
/// first fixture.
fn a_method(tree: &vleo_sheet::Tree) -> (String, Vec<f64>) {
    let g = opened::graph(tree).unwrap();
    for (k, def) in g.nodes.iter().enumerate() {
        if matches!(g.run.get(k), Some(Some(_))) && def.outputs.len() == 1 {
            if let Some(f) = def.fixtures.first() {
                if g.probe(k as u16, f.inputs).is_ok() {
                    return (def.id.to_string(), f.inputs.to_vec());
                }
            }
        }
    }
    panic!("no method row with an answering fixture");
}

#[test]
fn an_answer_outside_its_declared_range_is_refused_lower_bound_first() {
    let mut tree = vleo_files::convert::open(&root()).unwrap().0;
    let (id, inputs) = a_method(&tree);
    let answer = {
        let g = opened::graph(&tree).unwrap();
        g.probe(g.find(&id).unwrap(), &inputs).unwrap()[0]
    };
    let (unit, reason_lower, reason_upper) = {
        let sh = &tree.sheets[&id];
        (
            sh.unit.clone(),
            sh.reason_lower.clone(),
            sh.reason_upper.clone(),
        )
    };
    let si = vleo_units::Unit::from_name(&unit)
        .unwrap_or(vleo_units::Unit::One)
        .si_factor();

    // Above: the upper bound moved below the answer.
    let upper = answer / si - 1.0;
    tree.sheets.get_mut(&id).unwrap().upper = upper;
    let g = opened::graph(&tree).unwrap();
    match g.probe(g.find(&id).unwrap(), &inputs) {
        Err(Fault::OutOfDomain {
            edge: Edge::Upper,
            bound,
            reason,
            value,
            ..
        }) => {
            assert_eq!(bound, upper * si);
            assert_eq!(value, answer);
            assert_eq!(reason, reason_upper);
        }
        other => panic!("{id}: {other:?}"),
    }

    // Both: the lower bound above the answer too — and the lower is said
    // first, as the compiled row says it.
    let lower = answer / si + 1.0;
    tree.sheets.get_mut(&id).unwrap().lower = lower;
    let g = opened::graph(&tree).unwrap();
    match g.probe(g.find(&id).unwrap(), &inputs) {
        Err(Fault::OutOfDomain {
            edge: Edge::Lower,
            bound,
            reason,
            ..
        }) => {
            assert_eq!(bound, lower * si);
            assert_eq!(reason, reason_lower);
        }
        other => panic!("{id}: {other:?}"),
    }
}
