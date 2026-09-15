// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_uncertainty_growth`.
//!
//! Every expected value below names a source outside this code. A number
//! produced by the thing being tested proves nothing, so the schema
//! refuses a fixture whose provenance is the implementation.

#![allow(clippy::approx_constant, clippy::excessive_precision)]

use super::model;
use vleo_core::units::*;

fn relative_error(got: f64, expected: f64) -> f64 {
    if expected == 0.0 { pmath::abs(got) } else { pmath::abs((got - expected) / expected) }
}

/// lead 183 days (0.50 yr) — 9947 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(15811200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 57.0);
    assert!(err <= 1e-12, "lead 183 days (0.50 yr) — 9947 observed pairs: got {} want 57.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 365 days (1.00 yr) — 9675 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(31536000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 68.3);
    assert!(err <= 1e-12, "lead 365 days (1.00 yr) — 9675 observed pairs: got {} want 68.3, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 548 days (1.50 yr) — 9492 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(47347200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 77.0);
    assert!(err <= 1e-12, "lead 548 days (1.50 yr) — 9492 observed pairs: got {} want 77.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 730 days (2.00 yr) — 9310 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(63072000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 91.0);
    assert!(err <= 1e-12, "lead 730 days (2.00 yr) — 9310 observed pairs: got {} want 91.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 1096 days (3.00 yr) — 8946 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(94694400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 107.0);
    assert!(err <= 1e-12, "lead 1096 days (3.00 yr) — 8946 observed pairs: got {} want 107.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 1461 days (4.00 yr) — 8579 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Time::new(126230400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 114.0);
    assert!(err <= 1e-12, "lead 1461 days (4.00 yr) — 8579 observed pairs: got {} want 114.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 1826 days (5.00 yr) — 8215 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Time::new(157766400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 113.3);
    assert!(err <= 1e-12, "lead 1826 days (5.00 yr) — 8215 observed pairs: got {} want 113.3, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 2191 days (6.00 yr) — 7849 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Time::new(189302400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 104.0);
    assert!(err <= 1e-12, "lead 2191 days (6.00 yr) — 7849 observed pairs: got {} want 104.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 2557 days (7.00 yr) — 7483 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Time::new(220924800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 92.0);
    assert!(err <= 1e-12, "lead 2557 days (7.00 yr) — 7483 observed pairs: got {} want 92.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 2922 days (8.00 yr) — 7118 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Time::new(252460800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 88.0);
    assert!(err <= 1e-12, "lead 2922 days (8.00 yr) — 7118 observed pairs: got {} want 88.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 3287 days (9.00 yr) — 7026 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Time::new(283996800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 78.0);
    assert!(err <= 1e-12, "lead 3287 days (9.00 yr) — 7026 observed pairs: got {} want 78.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 3653 days (10.00 yr) — 6660 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_11() {
    let got = model::evaluate(Time::new(315619200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 66.0);
    assert!(err <= 1e-12, "lead 3653 days (10.00 yr) — 6660 observed pairs: got {} want 66.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 4018 days (11.00 yr) — 6295 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_12() {
    let got = model::evaluate(Time::new(347155200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 66.0);
    assert!(err <= 1e-12, "lead 4018 days (11.00 yr) — 6295 observed pairs: got {} want 66.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 4383 days (12.00 yr) — 5932 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_13() {
    let got = model::evaluate(Time::new(378691200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 76.0);
    assert!(err <= 1e-12, "lead 4383 days (12.00 yr) — 5932 observed pairs: got {} want 76.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 4748 days (13.00 yr) — 5567 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_14() {
    let got = model::evaluate(Time::new(410227200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 91.0);
    assert!(err <= 1e-12, "lead 4748 days (13.00 yr) — 5567 observed pairs: got {} want 91.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 5113 days (14.00 yr) — 5203 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_15() {
    let got = model::evaluate(Time::new(441763200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 106.0);
    assert!(err <= 1e-12, "lead 5113 days (14.00 yr) — 5203 observed pairs: got {} want 106.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 5478 days (15.00 yr) — 4838 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_16() {
    let got = model::evaluate(Time::new(473299200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 115.0);
    assert!(err <= 1e-12, "lead 5478 days (15.00 yr) — 4838 observed pairs: got {} want 115.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// below the first measured lead — 100 days holds the 183-day value rather than extrapolating
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_17() {
    let got = model::evaluate(Time::new(8640000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 57.0);
    assert!(err <= 1e-12, "below the first measured lead — 100 days holds the 183-day value rather than extrapolating: got {} want 57.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// above the last measured lead — 20 years holds the 15-year value
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_18() {
    let got = model::evaluate(Time::new(631152000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 115.0);
    assert!(err <= 1e-12, "above the last measured lead — 20 years holds the 15-year value: got {} want 115.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `lead 183 days (0.50 yr) — 9947 observed pairs` and the declared domain 0 … 120.
///
/// One per cent, not a decade. These domains are design bands — an altitude
/// range somebody chose, not a range over which the mathematics holds — so a
/// decade leaves most of them legitimately, and a check that cries wolf is a
/// check people turn off. What is left is still worth asking: a relation that
/// refuses at the immediate neighbours of the one point somebody verified is
/// either discontinuous there, or has a domain declared tighter than the
/// physics. Both are sheet questions, and both are invisible from the fixture.
#[test]
fn answers_near_the_known_good_point() {
    let mut refused: Vec<String> = Vec::new();
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Time::new(15811200.0 * scale)) {
            refused.push(format!("lead x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_uncertainty_growth refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 120 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
        refused
    );
}

/// Every answer sits inside the declared domain, and no call panics.
///
/// Not a restatement of the generated guard: it proves the guard is reachable,
/// that nothing routes around it, and that a hole cannot return a value that
/// is not a number. A division by zero inside a hole is caught by no guard.
#[test]
fn every_answer_is_inside_the_declared_domain() {
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Time::new(15811200.0 * scale)) {
            assert!(v.get().is_finite(), "sw_uncertainty_growth produced a value that is not a number");
            assert!(v.get() >= 0.0 && v.get() <= 120.0, "sw_uncertainty_growth answered {}, outside its declared domain 0 … 120 — the guard did not stop it", v.get());
        }
    }
}

/// The same inputs give a bit-identical answer.
///
/// A relation that reaches a clock, a hash order or any hidden state fails
/// here and nowhere else, and it is the one defect that makes bit-for-bit
/// agreement across the faces impossible rather than merely hard.
#[test]
fn the_same_inputs_give_the_same_answer() {
    let a = model::evaluate(Time::new(15811200.0));
    let b = model::evaluate(Time::new(15811200.0));
    match (a, b) {
        (Ok(x), Ok(y)) => assert!(x.get().to_bits() == y.get().to_bits(), "sw_uncertainty_growth is not deterministic: {} then {}", x.get(), y.get()),
        (Err(_), Err(_)) => {}
        _ => panic!("sw_uncertainty_growth refused on one call and answered on the other"),
    }
}

