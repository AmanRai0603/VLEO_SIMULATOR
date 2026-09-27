//! V16: a customer or condition either changes a run or says why it cannot.
//!
//! The solar maximum and minimum conditions set three rows that later became
//! computed. From then on every run of "solar maximum" was the nominal run,
//! offered under a storm's name, and nothing anywhere said so. These tests
//! hold the check that would have caught it, in both directions — the real
//! tree passes, and each way of breaking a file fails.

use vleo_sheet::gate::{validate_tree, Verdict};
use vleo_sheet::load::{load_all, Tree};

fn tree() -> Tree {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    load_all(root).unwrap()
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

/// A computed, published row, so the test uses one the tree actually has.
fn a_computed_row(t: &Tree) -> String {
    t.ordered()
        .iter()
        .find(|s| s.kind == "computed" && s.state == "published")
        .unwrap()
        .id
        .clone()
}

fn a_runnable_condition(t: &Tree) -> String {
    t.cases
        .values()
        .find(|c| c.kind == "condition" && c.unavailable.is_empty())
        .unwrap()
        .id
        .clone()
}

#[test]
fn the_tree_as_it_is_passes() {
    assert_eq!(v16(&tree()), None);
}

#[test]
fn a_value_aimed_at_a_computed_row_fails() {
    let mut t = tree();
    let row = a_computed_row(&t);
    let id = a_runnable_condition(&t);
    t.cases
        .get_mut(&id)
        .unwrap()
        .supply
        .push((row.clone(), 1.0));
    let why = v16(&t).expect("a condition setting a computed row passed");
    assert!(why.contains(&row) && why.contains("computed"), "{why}");
}

#[test]
fn a_value_aimed_at_no_row_fails() {
    let mut t = tree();
    let id = a_runnable_condition(&t);
    t.cases
        .get_mut(&id)
        .unwrap()
        .supply
        .push(("no_such_row".into(), 1.0));
    let why = v16(&t).expect("a condition naming no row passed");
    assert!(why.contains("no_such_row"), "{why}");
}

#[test]
fn saying_why_it_cannot_be_applied_is_the_way_out() {
    let mut t = tree();
    let row = a_computed_row(&t);
    let id = a_runnable_condition(&t);
    let c = t.cases.get_mut(&id).unwrap();
    c.supply.push((row, 1.0));
    c.unavailable = "re-pointing it is the environment owner's decision".into();
    assert_eq!(v16(&t), None);
}

#[test]
fn a_kind_that_is_neither_fails() {
    let mut t = tree();
    let id = a_runnable_condition(&t);
    t.cases.get_mut(&id).unwrap().kind = "scenario".into();
    let why = v16(&t).expect("a case of a third kind passed");
    assert!(why.contains("scenario"), "{why}");
}

#[test]
fn a_tree_with_no_customer_fails() {
    let mut t = tree();
    for c in t.cases.values_mut() {
        c.kind = "condition".into();
    }
    let why = v16(&t).expect("a tree with no customer passed");
    assert!(why.contains("no customer"), "{why}");
}
