// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_kp_mean_bias`.
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

/// Ap bin 0-5, centre 2.5 — 2529 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.041666666666666685);
    assert!(err <= 1e-12, "Ap bin 0-5, centre 2.5 — 2529 days: got {} want 0.041666666666666685, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 5-10, centre 7.5 — 3938 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(7.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.125);
    assert!(err <= 1e-12, "Ap bin 5-10, centre 7.5 — 3938 days: got {} want -0.125, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 10-15, centre 12.5 — 1836 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(12.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.09763888888888905);
    assert!(err <= 1e-12, "Ap bin 10-15, centre 12.5 — 1836 days: got {} want -0.09763888888888905, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 15-20, centre 17.5 — 839 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(17.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.11111111111111116);
    assert!(err <= 1e-12, "Ap bin 15-20, centre 17.5 — 839 days: got {} want -0.11111111111111116, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 20-30, centre 25 — 707 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(25.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.13333333333333286);
    assert!(err <= 1e-12, "Ap bin 20-30, centre 25 — 707 days: got {} want -0.13333333333333286, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 30-45, centre 37.5 — 288 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(37.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.24099537037037067);
    assert!(err <= 1e-12, "Ap bin 30-45, centre 37.5 — 288 days: got {} want -0.24099537037037067, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 45-70, centre 57.5 — 115 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(57.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.33333333333333304);
    assert!(err <= 1e-12, "Ap bin 45-70, centre 57.5 — 115 days: got {} want -0.33333333333333304, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 70-110, centre 90 — 25 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(90.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.4487179487179489);
    assert!(err <= 1e-12, "Ap bin 70-110, centre 90 — 25 days: got {} want -0.4487179487179489, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 110-400, centre 255 — 20 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(255.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.4868948412698413);
    assert!(err <= 1e-12, "Ap bin 110-400, centre 255 — 20 days: got {} want -0.4868948412698413, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// below the first bin centre — Ap 0 holds the 0-to-5 bin rather than extrapolating
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Ratio::new(0.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.041666666666666685);
    assert!(err <= 1e-12, "below the first bin centre — Ap 0 holds the 0-to-5 bin rather than extrapolating: got {} want 0.041666666666666685, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// above the last bin centre — Ap 1000 holds the 110-to-400 bin
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Ratio::new(1000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.4868948412698413);
    assert!(err <= 1e-12, "above the last bin centre — Ap 1000 holds the 110-to-400 bin: got {} want -0.4868948412698413, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `Ap bin 0-5, centre 2.5 — 2529 days` and the declared domain -1 … 0.5.
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
        if let Err(f) = model::evaluate(Ratio::new(2.5 * scale)) {
            refused.push(format!("ap x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_kp_mean_bias refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain -1 … 0.5 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(2.5 * scale)) {
            assert!(v.get().is_finite(), "sw_kp_mean_bias produced a value that is not a number for dKp_mean");
            assert!(v.get() >= -1.0 && v.get() <= 0.5, "sw_kp_mean_bias answered {} for dKp_mean, outside its declared domain -1 … 0.5 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(2.5));
    let b = model::evaluate(Ratio::new(2.5));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_kp_mean_bias is not deterministic for dKp_mean: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_kp_mean_bias refused on one call and answered on the other"),
    }
}

