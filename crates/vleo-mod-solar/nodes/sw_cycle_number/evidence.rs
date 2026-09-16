// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_cycle_number`.
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

/// one day before cycle 24 begins — still 23
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(281318400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 23.0);
    assert!(err <= 1e-12, "one day before cycle 24 begins — still 23: got {} want 23.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the day cycle 24 begins — 24, because a boundary belongs to the cycle it opens
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(281404800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 24.0);
    assert!(err <= 1e-12, "the day cycle 24 begins — 24, because a boundary belongs to the cycle it opens: got {} want 24.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// one day before cycle 25 begins — still 24
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(628387200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 24.0);
    assert!(err <= 1e-12, "one day before cycle 25 begins — still 24: got {} want 24.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the day cycle 25 begins — 25
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(628473600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 25.0);
    assert!(err <= 1e-12, "the day cycle 25 begins — 25: got {} want 25.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the declared mission epoch, 2027-01-01 — cycle 25, past the record's end
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 25.0);
    assert!(err <= 1e-12, "the declared mission epoch, 2027-01-01 — cycle 25, past the record's end: got {} want 25.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `one day before cycle 24 begins — still 23` and the declared domain 23 … 26.
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
        if let Err(f) = model::evaluate(Time::new(281318400.0 * scale)) {
            refused.push(format!("epoch x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_cycle_number refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 23 … 26 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Time::new(281318400.0 * scale)) {
            assert!(v.get().is_finite(), "sw_cycle_number produced a value that is not a number for cyc");
            assert!(v.get() >= 23.0 && v.get() <= 26.0, "sw_cycle_number answered {} for cyc, outside its declared domain 23 … 26 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Time::new(281318400.0));
    let b = model::evaluate(Time::new(281318400.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_cycle_number is not deterministic for cyc: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_cycle_number refused on one call and answered on the other"),
    }
}

