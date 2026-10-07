//! A row whose method returns its one input unchanged is refused by the gate.
//!
//! It publishes a number the tree already holds under another name, and a
//! reader meets the same answer twice with nothing to say which to believe.
//! The gate found it in a row's code; with the code gone the relation is the
//! method, so the gate reads the method as well. Only a crossing, a relay
//! across a layer, or a retired row may relay without computing.

use std::path::Path;

use vleo_sheet::gate::{gate_node, Verdict};
use vleo_sheet::load::{load_all, Tree};
use vleo_sheet::text::producer_of;

fn tree() -> Tree {
    load_all(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
}

/// A row that may not relay: not a crossing, not retired, and reading its one
/// input from its own layer.
fn a_row_that_must_compute(t: &Tree) -> String {
    t.sheets
        .values()
        .find(|s| {
            s.inputs.len() == 1
                && s.crosses_to.trim().is_empty()
                && s.state != "deprecated"
                && !s.method.text.trim().is_empty()
                && t.sheets
                    .get(producer_of(&s.inputs[0].var))
                    .is_some_and(|p| p.layer == s.layer)
        })
        .expect("a row with a method and one input from its own layer")
        .id
        .clone()
}

fn no_identity(t: &Tree, id: &str) -> Verdict {
    gate_node(&t.sheets[id], t)
        .into_iter()
        .find(|c| c.name == "no-identity")
        .expect("the gate asks no-identity of every row")
        .verdict
}

#[test]
fn a_row_that_computes_passes() {
    let t = tree();
    let id = a_row_that_must_compute(&t);
    assert_eq!(no_identity(&t, &id), Verdict::Pass);
}

#[test]
fn a_method_that_returns_its_one_input_is_refused() {
    let mut t = tree();
    let id = a_row_that_must_compute(&t);
    let sh = t.sheets.get_mut(&id).unwrap();
    let binding = sh.inputs[0].binding.clone();
    sh.method.text = format!("# relays its input\nreturn {binding}\n");
    match no_identity(&t, &id) {
        Verdict::Fail(why) => assert!(why.contains(&binding), "{why}"),
        v => panic!("{id} returns its one input and the gate said {v:?}"),
    }
}

#[test]
fn a_crossing_may_relay() {
    let mut t = tree();
    let id = a_row_that_must_compute(&t);
    let sh = t.sheets.get_mut(&id).unwrap();
    let binding = sh.inputs[0].binding.clone();
    sh.method.text = format!("return {binding}\n");
    sh.crosses_to = "somewhere".into();
    assert_eq!(no_identity(&t, &id), Verdict::Pass);
}
