// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_ap_daily_band_spread`.
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

/// at the declared window's own sustained level, Ap 26.70 — just above the top knot, so this is the clamp and the sheet argues about what it costs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(26.69530964)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 63.8519);
    assert!(err <= 1e-12, "at the declared window's own sustained level, Ap 26.70 — just above the top knot, so this is the clamp and the sheet argues about what it costs: got {} want 63.8519, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the Ap 11 knot, exactly — a transcription check, and near where the study's own sample sat at Ap 11.64
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(11.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 14.6667);
    assert!(err <= 1e-12, "the Ap 11 knot, exactly — a transcription check, and near where the study's own sample sat at Ap 11.64: got {} want 14.6667, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the Ap 20 knot, exactly — a transcription check below the steep last segment
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(20.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 28.6759);
    assert!(err <= 1e-12, "the Ap 20 knot, exactly — a transcription check below the steep last segment: got {} want 28.6759, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// halfway between the Ap 15 and Ap 20 knots — the interpolation
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(17.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 26.23055);
    assert!(err <= 1e-9, "halfway between the Ap 15 and Ap 20 knots — the interpolation: got {} want 26.23055, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// below the bottom of the table — the clamp, not an extrapolation toward zero
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(2.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 6.0167);
    assert!(err <= 1e-12, "below the bottom of the table — the clamp, not an extrapolation toward zero: got {} want 6.0167, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `at the declared window's own sustained level, Ap 26.70 — just above the top knot, so this is the clamp and the sheet argues about what it costs` and the declared domain 0 … 150.
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
        if let Err(f) = model::evaluate(Ratio::new(26.69530964 * scale)) {
            refused.push(format!("level x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_ap_daily_band_spread refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 150 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(26.69530964 * scale)) {
            assert!(v.get().is_finite(), "sw_ap_daily_band_spread produced a value that is not a number for dAp_day");
            assert!(v.get() >= 0.0 && v.get() <= 150.0, "sw_ap_daily_band_spread answered {} for dAp_day, outside its declared domain 0 … 150 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(26.69530964));
    let b = model::evaluate(Ratio::new(26.69530964));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_ap_daily_band_spread is not deterministic for dAp_day: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_ap_daily_band_spread refused on one call and answered on the other"),
    }
}

