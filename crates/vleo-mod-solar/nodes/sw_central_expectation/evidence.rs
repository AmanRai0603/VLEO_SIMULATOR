// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_central_expectation`.
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

/// the declared case — epoch 2027-01-01, a five-year mission
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 86.84972489094359);
    assert!(err <= 1e-12, "the declared case — epoch 2027-01-01, a five-year mission: got {} want 86.84972489094359, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// fifteen years — the window spans more than a cycle, so the mean settles near the record's own level
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(473364000.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 105.72687096778448);
    assert!(err <= 1e-12, "fifteen years — the window spans more than a cycle, so the mean settles near the record's own level: got {} want 105.72687096778448, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// half a Julian year, the shortest mission — the one lead where today's 150 sfu is still faintly visible
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(15778800.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 105.5465798678905);
    assert!(err <= 1e-12, "half a Julian year, the shortest mission — the one lead where today's 150 sfu is still faintly visible: got {} want 105.5465798678905, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// a quiet day today changes nothing at five years
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(70.0), Time::new(157788000.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 86.84972489094359);
    assert!(err <= 1e-12, "a quiet day today changes nothing at five years: got {} want 86.84972489094359, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// nor does a very active one — at five years the answer is the window climatology alone
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(340.0), Time::new(157788000.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 86.84972489094359);
    assert!(err <= 1e-12, "nor does a very active one — at five years the answer is the window climatology alone: got {} want 86.84972489094359, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the earliest epoch the range allows, 2018-01-01 — a mission rising into cycle 25's maximum
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(568080000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 115.03865410832137);
    assert!(err <= 1e-12, "the earliest epoch the range allows, 2018-01-01 — a mission rising into cycle 25's maximum: got {} want 115.03865410832137, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// an epoch near the next maximum, 2032-11-08 — the same mission, a different sky, and one cycle 25's size does not vouch for
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(1036800000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 144.9367498101883);
    assert!(err <= 1e-12, "an epoch near the next maximum, 2032-11-08 — the same mission, a different sky, and one cycle 25's size does not vouch for: got {} want 144.9367498101883, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the latest epoch the range allows, 2040-01-01
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(1262304000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 88.67145559562236);
    assert!(err <= 1e-12, "the latest epoch the range allows, 2040-01-01: got {} want 88.67145559562236, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// one whole cycle period later — epoch 14040 is 4178 days after 9862, so the SHAPE repeats; the answer does not, because the amplitude hands over
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(1213056000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 78.21016904177215);
    assert!(err <= 1e-12, "one whole cycle period later — epoch 14040 is 4178 days after 9862, so the SHAPE repeats; the answer does not, because the amplitude hands over: got {} want 78.21016904177215, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `the declared case — epoch 2027-01-01, a five-year mission` and the declared domain 60 … 400.
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
        if let Err(f) = model::evaluate(Ratio::new(150.0 * scale), Time::new(157788000.0), Time::new(852076800.0)) {
            refused.push(format!("today x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(150.0), Time::new(157788000.0 * scale), Time::new(852076800.0)) {
            refused.push(format!("lead x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0 * scale)) {
            refused.push(format!("epoch x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_central_expectation refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 60 … 400 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(150.0 * scale), Time::new(157788000.0), Time::new(852076800.0)) {
            assert!(v.get().is_finite(), "sw_central_expectation produced a value that is not a number");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_central_expectation answered {}, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(150.0), Time::new(157788000.0 * scale), Time::new(852076800.0)) {
            assert!(v.get().is_finite(), "sw_central_expectation produced a value that is not a number");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_central_expectation answered {}, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0 * scale)) {
            assert!(v.get().is_finite(), "sw_central_expectation produced a value that is not a number");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_central_expectation answered {}, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0));
    let b = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0));
    match (a, b) {
        (Ok(x), Ok(y)) => assert!(x.get().to_bits() == y.get().to_bits(), "sw_central_expectation is not deterministic: {} then {}", x.get(), y.get()),
        (Err(_), Err(_)) => {}
        _ => panic!("sw_central_expectation refused on one call and answered on the other"),
    }
}

