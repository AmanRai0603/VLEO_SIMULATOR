//! A name in a sheet that resolves to nothing stops the build.
//!
//! The compiled-in tables once guessed: an unknown input became variable 0, an
//! unknown cycle member was left out, a fixture's missing input was fed 0.0.
//! Each is tried here on a copy of the design, read from `design/`, held in
//! memory.

use std::path::Path;
use vleo_sheet::wiring::errors as wiring_errors;

fn tree() -> vleo_sheet::load::Tree {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    vleo_files::convert::open(&root).unwrap().0
}

#[test]
fn the_tree_as_it_is_resolves_every_name() {
    assert_eq!(wiring_errors(&tree()), Vec::<String>::new());
}

#[test]
fn an_input_reading_no_row_is_named() {
    let mut t = tree();
    let sh = t
        .sheets
        .values_mut()
        .find(|s| !s.inputs.is_empty())
        .unwrap();
    let id = sh.id.clone();
    sh.inputs[0].var = "no_such_row".into();
    let e = wiring_errors(&t);
    assert!(
        e.iter()
            .any(|m| m.starts_with(&id) && m.contains("no_such_row")),
        "{e:?}"
    );
}

#[test]
fn a_fixture_missing_an_input_is_named() {
    let mut t = tree();
    let sh = t
        .sheets
        .values_mut()
        .find(|s| !s.fixtures.is_empty() && !s.inputs.is_empty())
        .unwrap();
    let binding = sh.inputs[0].binding.clone();
    sh.fixtures[0].inputs.retain(|(k, _)| *k != binding);
    let e = wiring_errors(&t);
    assert!(e.iter().any(|m| m.contains(&binding)), "{e:?}");
}

#[test]
fn a_cycle_naming_no_row_is_named() {
    let mut t = tree();
    let c = t
        .cases
        .values_mut()
        .find(|c| !c.cycles.is_empty())
        .expect("every case inherits the architecture's cycles");
    c.cycles[0].converge_on = "not_a_row".into();
    let e = wiring_errors(&t);
    assert!(e.iter().any(|m| m.contains("not_a_row")), "{e:?}");
}
