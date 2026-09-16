// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_activity_band`.
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

/// the record's quietest day, 64 sfu — band 1, low
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(64.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "the record's quietest day, 64 sfu — band 1, low: got {} want 1.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 89.9 sfu — still low, the edge is not yet reached
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(89.9)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "89.9 sfu — still low, the edge is not yet reached: got {} want 1.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 90 sfu exactly — moderate begins AT the edge, not after it
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(90.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "90 sfu exactly — moderate begins AT the edge, not after it: got {} want 2.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 129.9 sfu — still moderate
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(129.9)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "129.9 sfu — still moderate: got {} want 2.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 130 sfu exactly — elevated begins
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(130.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "130 sfu exactly — elevated begins: got {} want 3.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 169.9 sfu — still elevated
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(169.9)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "169.9 sfu — still elevated: got {} want 3.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 170 sfu exactly — high begins
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(170.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "170 sfu exactly — high begins: got {} want 4.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the record's largest day, 343 sfu — band 4, which is unbounded above
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(343.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "the record's largest day, 343 sfu — band 4, which is unbounded above: got {} want 4.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the design value this subsystem publishes, 228 sfu — band 4, the top one
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(228.1374378829)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "the design value this subsystem publishes, 228 sfu — band 4, the top one: got {} want 4.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `the record's quietest day, 64 sfu — band 1, low` and the declared domain 1 … 4.
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
        if let Err(f) = model::evaluate(Ratio::new(64.0 * scale)) {
            refused.push(format!("f107 x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_activity_band refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 1 … 4 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(64.0 * scale)) {
            assert!(v.get().is_finite(), "sw_activity_band produced a value that is not a number for band");
            assert!(v.get() >= 1.0 && v.get() <= 4.0, "sw_activity_band answered {} for band, outside its declared domain 1 … 4 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(64.0));
    let b = model::evaluate(Ratio::new(64.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_activity_band is not deterministic for band: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_activity_band refused on one call and answered on the other"),
    }
}

