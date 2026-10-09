//! V16: the case sets only what a run can take, and groups only real inputs.
//!
//! Two stored skies once set three rows that later became computed. From then
//! on every run of "solar maximum" was the plain design, offered under a
//! storm's name, and nothing anywhere said so. These hold the check that would
//! have caught it — and the same check over the Condition list, which decides
//! which half of every uploaded CSV an input sits in. Both directions: the
//! design, read from `design/`, passes, and each way of breaking the case fails.

use vleo_sheet::gate::{validate_tree, Verdict};
use vleo_sheet::load::Tree;

fn tree() -> Tree {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    vleo_files::convert::open(&root).unwrap().0
}

/// V16's verdict: `None` when it passed, the failure text when it did not.
fn v16(t: &Tree) -> Option<String> {
    let c = validate_tree(t)
        .into_iter()
        .find(|c| c.name.starts_with("V16"))
        .expect("V16 is not among the assembly checks");
    match c.verdict {
        Verdict::Fail(why) => Some(why),
        _ => None,
    }
}

fn a_computed_row(t: &Tree) -> String {
    t.ordered()
        .iter()
        .find(|s| s.kind == "computed" && s.state == "published")
        .unwrap()
        .id
        .clone()
}

fn the_case(t: &mut Tree) -> &mut vleo_sheet::model::Case {
    t.cases.values_mut().next().expect("cases/ holds no case")
}

#[test]
fn the_tree_as_it_is_passes() {
    assert_eq!(v16(&tree()), None);
}

#[test]
fn a_value_aimed_at_a_computed_row_fails() {
    let mut t = tree();
    let row = a_computed_row(&t);
    the_case(&mut t).supply.push((row.clone(), 1.0));
    let why = v16(&t).expect("a case setting a computed row passed");
    assert!(why.contains(&row) && why.contains("computed"), "{why}");
}

#[test]
fn a_condition_that_is_computed_fails() {
    let mut t = tree();
    let row = a_computed_row(&t);
    the_case(&mut t).conditions.push(row.clone());
    let why = v16(&t).expect("a computed row listed as a condition passed");
    assert!(why.contains(&row) && why.contains("computed"), "{why}");
}

#[test]
fn a_name_that_is_no_row_fails_in_either_list() {
    let mut t = tree();
    the_case(&mut t).supply.push(("no_such_row".into(), 1.0));
    assert!(v16(&t)
        .expect("a supply naming no row passed")
        .contains("no_such_row"));
    let mut t = tree();
    the_case(&mut t).conditions.push("no_such_row".into());
    assert!(v16(&t)
        .expect("a condition naming no row passed")
        .contains("no_such_row"));
}

#[test]
fn a_condition_listed_twice_fails() {
    let mut t = tree();
    let c = the_case(&mut t);
    let first = c.conditions[0].clone();
    c.conditions.push(first.clone());
    let why = v16(&t).expect("a condition listed twice passed");
    assert!(why.contains(&first) && why.contains("twice"), "{why}");
}

#[test]
fn a_tree_with_no_case_fails() {
    let mut t = tree();
    t.cases.clear();
    let why = v16(&t).expect("a tree with no case passed");
    assert!(why.contains("no case"), "{why}");
}
