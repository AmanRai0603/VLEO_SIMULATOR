//! What the generator puts where a hole has no body yet.
//!
//! A row whose sheet is finished and whose Rust is not is the ordinary
//! half-finished state: `xtask docs` emits the scaffold, `xtask fill` splices
//! the bodies, and the gap pass names what is still open. The generated code in
//! between has to be something a run can survive.
//!
//! It was `todo!()`, which is a panic. The daemon evaluates every published row
//! to serve a page, so one published row with an unfilled hole took down the
//! whole tool — on a request that had nothing to do with that row. It is
//! reachable in one click now that a row can be published from a browser, but it
//! was never right: the fifth rule says a refusal is never a substitution, and a
//! row with no content returns `NotRun` UNDER ITS OWN NAME so a run can print
//! "n ran, m blocked" and name the blocked. A process that died names nothing.

use std::collections::BTreeMap;

/// The first published row in the tree that has a numbered step, so this is
/// checked against a sheet the repository actually has rather than one written
/// to suit the test.
fn a_row_with_a_step() -> (vleo_sheet::model::Sheet, vleo_sheet::load::Tree) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let tree = vleo_sheet::load::load_all(root).unwrap();
    let sh = tree
        .ordered()
        .iter()
        .find(|s| !s.is_seeded() && !s.steps.is_empty())
        .map(|s| (*s).clone())
        .expect("the tree has a published row with a numbered step");
    (sh, tree)
}

#[test]
fn an_empty_hole_returns_not_run_rather_than_panicking() {
    let (sh, _) = a_row_with_a_step();
    // No hole bodies at all: the state a row is in the moment it is published.
    let out = vleo_sheet::emit::model_rs(&sh, &BTreeMap::new());
    assert!(
        out.contains("return Err(Fault::NotRun { node: NODE_ID })"),
        "an empty hole must return, under this row's own name:\n{out}"
    );
    assert!(
        !out.contains("todo!"),
        "and it must not panic — a panic in a node takes down whatever called it:\n{out}"
    );
}

#[test]
fn the_placeholder_sits_outside_the_hole_so_the_hole_stays_empty() {
    // The second half of the same defect. The placeholder used to be written
    // BETWEEN the numbered markers, where `read_holes` reads it back as
    // somebody's body — so the hole looked filled, the next generation
    // preserved the placeholder verbatim, and the row could never be
    // regenerated out of that state.
    let (sh, _) = a_row_with_a_step();
    let out = vleo_sheet::emit::model_rs(&sh, &BTreeMap::new());
    let n = sh.steps[0].number;
    let open = out
        .find(&format!("// ---- HOLE {n} :"))
        .expect("an open marker");
    let close = out
        .find(&format!("// ---- end HOLE {n}"))
        .expect("a close marker");
    let inside = &out[open..close];
    assert!(
        !inside.contains("NotRun"),
        "the hole itself must be empty, or the placeholder becomes the body:\n{inside}"
    );
    assert!(
        out[close..].contains("NotRun"),
        "and the placeholder goes after the hole:\n{out}"
    );
}

#[test]
fn a_filled_hole_gets_no_placeholder_at_all() {
    let (sh, _) = a_row_with_a_step();
    let n = sh.steps[0].number;
    let mut holes = BTreeMap::new();
    // Every hole filled, so nothing is blocked and nothing is unreachable.
    for st in &sh.steps {
        holes.insert(
            st.number,
            format!("let {}: {} = Default::default();", st.binds, st.ty),
        );
    }
    let out = vleo_sheet::emit::model_rs(&sh, &holes);
    assert!(
        !out.contains("Fault::NotRun"),
        "a row whose holes are filled is not blocked:\n{out}"
    );
    assert!(
        !out.contains("#[allow(unreachable_code"),
        "and nothing after the body is unreachable, so nothing is allowed away"
    );
    assert!(
        out.contains(&format!("// ---- end HOLE {n}")),
        "the markers are still there for the next generation to splice into"
    );
}
