//! A name in the design that resolves to nothing stops the graph being built,
//! and says which.
//!
//! The compiled build refused these because it would not compile. Read from
//! the design's files at run time, the same name would run as the first
//! variable, or as zero, or be left out — a different question answered
//! without a word. So the reader asks first (`vleo_sheet::wiring`), and each
//! wrong name is tried here on the design in `design/`, held in memory.

#![cfg(feature = "std")]

use std::path::PathBuf;

use vleo_modules::opened;
use vleo_sheet::load::Tree;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The design as its files hold it, read the way the reader reads it.
fn design() -> Tree {
    vleo_files::convert::open(&root()).unwrap().0
}

/// What the reader says of a tree it refuses.
fn refused(tree: &Tree) -> String {
    match opened::graph(tree) {
        Ok(_) => panic!("a design with a name that resolves to nothing made a graph"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn the_design_as_it_is_makes_a_graph() {
    opened::graph(&design()).expect("every name in design/ resolves");
}

#[test]
fn an_input_reading_no_row_is_refused_by_name() {
    let mut t = design();
    let sh = t
        .sheets
        .values_mut()
        .find(|s| !s.inputs.is_empty())
        .unwrap();
    let id = sh.id.clone();
    sh.inputs[0].var = "no_such_row".into();
    let said = refused(&t);
    assert!(said.contains(&id) && said.contains("no_such_row"), "{said}");
}

#[test]
fn a_case_giving_no_value_for_an_input_is_refused_by_name() {
    let mut t = design();
    let sh = t
        .sheets
        .values_mut()
        .find(|s| !s.fixtures.is_empty() && !s.inputs.is_empty())
        .unwrap();
    let id = sh.id.clone();
    let binding = sh.inputs[0].binding.clone();
    sh.fixtures[0].inputs.retain(|(k, _)| *k != binding);
    let said = refused(&t);
    assert!(said.contains(&id) && said.contains(&binding), "{said}");
}

#[test]
fn a_case_about_an_output_the_row_does_not_publish_is_refused_by_name() {
    let mut t = design();
    let sh = t
        .sheets
        .values_mut()
        .find(|s| !s.fixtures.is_empty())
        .unwrap();
    let id = sh.id.clone();
    sh.fixtures[0].variable = "no_such_output".into();
    let said = refused(&t);
    assert!(
        said.contains(&id) && said.contains("no_such_output"),
        "{said}"
    );
}

#[test]
fn a_supply_naming_no_row_is_refused_by_name() {
    let mut t = design();
    let c = t.cases.values_mut().next().expect("the design has a case");
    c.supply.push(("not_a_row".into(), 1.0));
    let said = refused(&t);
    assert!(said.contains("not_a_row"), "{said}");
}

#[test]
fn a_loop_naming_no_row_is_refused_by_name() {
    let mut t = design();
    let c = t
        .cases
        .values_mut()
        .find(|c| !c.cycles.is_empty())
        .expect("the design's one loop is declared on a case");
    c.cycles[0].converge_on = "not_a_row".into();
    let said = refused(&t);
    assert!(said.contains("not_a_row"), "{said}");
}
