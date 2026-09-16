// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_daily_band_spread`.
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

/// at the declared window's own sustained level, 104.07 — an interpolation between the 100 and 120 knots
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(104.07110896)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 20.0721663388);
    assert!(err <= 1e-9, "at the declared window's own sustained level, 104.07 — an interpolation between the 100 and 120 knots: got {} want 20.0721663388, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the 100 sfu knot, exactly — a transcription check on one of the seven pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(100.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 18.2741);
    assert!(err <= 1e-12, "the 100 sfu knot, exactly — a transcription check on one of the seven pairs: got {} want 18.2741, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the 145 sfu knot, exactly — near where the study's own sample sat
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(145.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 34.6148);
    assert!(err <= 1e-12, "the 145 sfu knot, exactly — near where the study's own sample sat: got {} want 34.6148, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// halfway between the 100 and 120 knots — the interpolation, on round numbers
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(112.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 23.7949125);
    assert!(err <= 1e-9, "halfway between the 100 and 120 knots — the interpolation, on round numbers: got {} want 23.7949125, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// below the bottom of the table — the clamp, not an extrapolation toward zero
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(60.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.6259);
    assert!(err <= 1e-12, "below the bottom of the table — the clamp, not an extrapolation toward zero: got {} want 4.6259, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// far above the top of the table — the clamp again, which is where a cycle-maximum window would land
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(300.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 46.9259);
    assert!(err <= 1e-12, "far above the top of the table — the clamp again, which is where a cycle-maximum window would land: got {} want 46.9259, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `at the declared window's own sustained level, 104.07 — an interpolation between the 100 and 120 knots` and the declared domain 0 … 120.
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
        if let Err(f) = model::evaluate(Ratio::new(104.07110896 * scale)) {
            refused.push(format!("level x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_daily_band_spread refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 120 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(104.07110896 * scale)) {
            assert!(v.get().is_finite(), "sw_daily_band_spread produced a value that is not a number for dF107_day");
            assert!(v.get() >= 0.0 && v.get() <= 120.0, "sw_daily_band_spread answered {} for dF107_day, outside its declared domain 0 … 120 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(104.07110896));
    let b = model::evaluate(Ratio::new(104.07110896));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_daily_band_spread is not deterministic for dF107_day: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_daily_band_spread refused on one call and answered on the other"),
    }
}

