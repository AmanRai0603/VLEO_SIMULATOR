//! A sweep is one loop, the engine's, whichever face asks for it.
//!
//! What must hold: the answer at every point is the answer a single run gives
//! there; the axis must be a row a reader can move; a sweep needs two points;
//! and every point that could not be computed is kept with why, never dropped.

use vleo_bus::{Case, RunMode};
use vleo_modules::results::sweep;
use vleo_modules::{evaluate, Scratch};

fn base() -> Case {
    Case {
        target: "sw_ap_design_long".into(),
        mode: RunMode::Branch,
        ..Default::default()
    }
}

#[test]
fn every_point_is_the_answer_a_single_run_gives_there() {
    let w = sweep(
        &base(),
        "sw_ap_design_long",
        "sw_ap_central_expectation",
        5.0,
        35.0,
        4,
    )
    .unwrap();
    assert_eq!(w.x.len() + w.refused.len(), 4, "a point was dropped");
    assert!(!w.x.is_empty(), "no point answered");
    for (x, y) in w.x.iter().zip(&w.y) {
        let mut c = base();
        c.supply.push(("sw_ap_central_expectation".into(), *x));
        let r = evaluate(&c, &mut Scratch::new()).unwrap();
        let one = r
            .values
            .iter()
            .find(|v| v.id == "sw_ap_design_long")
            .unwrap()
            .value;
        assert_eq!(*y, one, "the sweep at {x} is not the run at {x}");
    }
    assert!(
        w.y.windows(2).all(|p| p[1] > p[0]),
        "the design level does not rise with the expectation"
    );
}

#[test]
fn the_axis_is_a_row_a_reader_can_move_and_a_sweep_has_two_points() {
    let e = sweep(
        &base(),
        "sw_ap_design_long",
        "sw_ap_design_short",
        0.0,
        1.0,
        5,
    )
    .unwrap_err();
    assert_eq!(e.kind(), vleo_modules::ErrorKind::Invalid, "{e}");
    assert!(e.message().contains("overwritten"), "{e}");
    assert!(sweep(
        &base(),
        "sw_ap_design_long",
        "sw_ap_central_expectation",
        5.0,
        35.0,
        1
    )
    .is_err());
    assert!(sweep(
        &base(),
        "no_such_row",
        "sw_ap_central_expectation",
        5.0,
        35.0,
        3
    )
    .is_err());
}

#[test]
fn a_point_that_cannot_be_computed_is_kept_with_why() {
    // Needs reference data this test does not supply: every point refuses,
    // and every one is kept.
    let w = sweep(
        &base(),
        "sw_ap_design",
        "sw_storm_design_level",
        1.0,
        3.0,
        3,
    )
    .unwrap();
    assert!(w.x.is_empty());
    assert_eq!(w.refused.len(), 3, "a refused point was dropped");
}
