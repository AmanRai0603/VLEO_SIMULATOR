// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_horizon_climatology`.
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

/// lead 1 day — 10312 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(86400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.3929);
    assert!(err <= 1e-12, "lead 1 day — 10312 observed pairs: got {} want 44.3929, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 2 days — 10309 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(172800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.3953);
    assert!(err <= 1e-12, "lead 2 days — 10309 observed pairs: got {} want 44.3953, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 3 days — 10307 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(259200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.3968);
    assert!(err <= 1e-12, "lead 3 days — 10307 observed pairs: got {} want 44.3968, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 5 days — 10303 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(432000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.3993);
    assert!(err <= 1e-12, "lead 5 days — 10303 observed pairs: got {} want 44.3993, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 7 days — 10299 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(604800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4012);
    assert!(err <= 1e-12, "lead 7 days — 10299 observed pairs: got {} want 44.4012, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 10 days — 10293 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Time::new(864000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4036);
    assert!(err <= 1e-12, "lead 10 days — 10293 observed pairs: got {} want 44.4036, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 14 days — 10285 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Time::new(1209600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4069);
    assert!(err <= 1e-12, "lead 14 days — 10285 observed pairs: got {} want 44.4069, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 20 days — 10273 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Time::new(1728000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4071);
    assert!(err <= 1e-12, "lead 20 days — 10273 observed pairs: got {} want 44.4071, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 27 days — 10259 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Time::new(2332800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4147);
    assert!(err <= 1e-12, "lead 27 days — 10259 observed pairs: got {} want 44.4147, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 40 days — 10233 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Time::new(3456000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4242);
    assert!(err <= 1e-12, "lead 40 days — 10233 observed pairs: got {} want 44.4242, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 60 days — 10193 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Time::new(5184000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4344);
    assert!(err <= 1e-12, "lead 60 days — 10193 observed pairs: got {} want 44.4344, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 90 days — 10133 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_11() {
    let got = model::evaluate(Time::new(7776000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4458);
    assert!(err <= 1e-12, "lead 90 days — 10133 observed pairs: got {} want 44.4458, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 135 days — 10043 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_12() {
    let got = model::evaluate(Time::new(11664000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4635);
    assert!(err <= 1e-12, "lead 135 days — 10043 observed pairs: got {} want 44.4635, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 180 days — 9953 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_13() {
    let got = model::evaluate(Time::new(15552000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4714);
    assert!(err <= 1e-12, "lead 180 days — 9953 observed pairs: got {} want 44.4714, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 270 days — 9773 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_14() {
    let got = model::evaluate(Time::new(23328000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.5344);
    assert!(err <= 1e-12, "lead 270 days — 9773 observed pairs: got {} want 44.5344, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 365 days — 9675 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_15() {
    let got = model::evaluate(Time::new(31536000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.6734);
    assert!(err <= 1e-12, "lead 365 days — 9675 observed pairs: got {} want 44.6734, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 547 days — 9493 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_16() {
    let got = model::evaluate(Time::new(47260800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 45.0333);
    assert!(err <= 1e-12, "lead 547 days — 9493 observed pairs: got {} want 45.0333, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 730 days — 9310 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_17() {
    let got = model::evaluate(Time::new(63072000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 45.3201);
    assert!(err <= 1e-12, "lead 730 days — 9310 observed pairs: got {} want 45.3201, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// beyond the last measured lead — five years holds the two-year value rather than extrapolating
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_18() {
    let got = model::evaluate(Time::new(157788000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 45.3201);
    assert!(err <= 1e-12, "beyond the last measured lead — five years holds the two-year value rather than extrapolating: got {} want 45.3201, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `lead 1 day — 10312 observed pairs` and the declared domain 0 … 50.
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
        if let Err(f) = model::evaluate(Time::new(86400.0 * scale)) {
            refused.push(format!("lead x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_horizon_climatology refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 50 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Time::new(86400.0 * scale)) {
            assert!(v.get().is_finite(), "sw_horizon_climatology produced a value that is not a number");
            assert!(v.get() >= 0.0 && v.get() <= 50.0, "sw_horizon_climatology answered {}, outside its declared domain 0 … 50 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Time::new(86400.0));
    let b = model::evaluate(Time::new(86400.0));
    match (a, b) {
        (Ok(x), Ok(y)) => assert!(x.get().to_bits() == y.get().to_bits(), "sw_horizon_climatology is not deterministic: {} then {}", x.get(), y.get()),
        (Err(_), Err(_)) => {}
        _ => panic!("sw_horizon_climatology refused on one call and answered on the other"),
    }
}

