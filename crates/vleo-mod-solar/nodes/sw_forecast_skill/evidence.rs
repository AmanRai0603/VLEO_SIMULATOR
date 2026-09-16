// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_forecast_skill`.
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

/// lead 13 — mid-window, still strongly skillful — +0.402 from 1237 pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(1123200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.4016285468);
    assert!(err <= 1e-12, "lead 13 — mid-window, still strongly skillful — +0.402 from 1237 pairs: got {} want 0.4016285468, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 1 — the outlook's weakest positive skill, where persistence is hardest to beat
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(86400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.068515911);
    assert!(err <= 1e-12, "lead 1 — the outlook's weakest positive skill, where persistence is hardest to beat: got {} want 0.068515911, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 9 — peak skill over the whole window — +0.438 from 1241 pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(777600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.4375371731);
    assert!(err <= 1e-12, "lead 9 — peak skill over the whole window — +0.438 from 1241 pairs: got {} want 0.4375371731, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 13.5 — between two measured leads, so this one tests the interpolation and not the table
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(1166400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.41015283365);
    assert!(err <= 1e-12, "lead 13.5 — between two measured leads, so this one tests the interpolation and not the table: got {} want 0.41015283365, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 23 — the last positive lead, +0.018, and the outlook is all but worthless here
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(1987200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.0176508866);
    assert!(err <= 1e-12, "lead 23 — the last positive lead, +0.018, and the outlook is all but worthless here: got {} want 0.0176508866, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 24 — the sign change: the outlook is now worse than assuming nothing changes
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Time::new(2073600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.0355195984);
    assert!(err <= 1e-12, "lead 24 — the sign change: the outlook is now worse than assuming nothing changes: got {} want -0.0355195984, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 26 — the declared lead — -0.022 from 868 pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Time::new(2246400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.0220863078);
    assert!(err <= 1e-12, "lead 26 — the declared lead — -0.022 from 868 pairs: got {} want -0.0220863078, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `lead 13 — mid-window, still strongly skillful — +0.402 from 1237 pairs` and the declared domain -0.1 … 0.5.
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
        if let Err(f) = model::evaluate(Time::new(1123200.0 * scale)) {
            refused.push(format!("lead x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_forecast_skill refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain -0.1 … 0.5 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Time::new(1123200.0 * scale)) {
            assert!(v.get().is_finite(), "sw_forecast_skill produced a value that is not a number for S_f107");
            assert!(v.get() >= -0.1 && v.get() <= 0.5, "sw_forecast_skill answered {} for S_f107, outside its declared domain -0.1 … 0.5 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Time::new(1123200.0));
    let b = model::evaluate(Time::new(1123200.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_forecast_skill is not deterministic for S_f107: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_forecast_skill refused on one call and answered on the other"),
    }
}

