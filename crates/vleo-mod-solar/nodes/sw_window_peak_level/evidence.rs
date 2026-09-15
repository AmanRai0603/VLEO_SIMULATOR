// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_window_peak_level`.
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

/// the declared case — the highest sustained level a 2027 five-year mission meets
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(157788000.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 110.65627313580244);
    assert!(err <= 1e-12, "the declared case — the highest sustained level a 2027 five-year mission meets: got {} want 110.65627313580244, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// MATLAB's own window, 2027-06-26 for a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(31536000.0), Time::new(867283200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 105.56045788424966);
    assert!(err <= 1e-12, "MATLAB's own window, 2027-06-26 for a year: got {} want 105.56045788424966, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// half a Julian year from the declared epoch — short enough that the peak is at its own start
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(15778800.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 110.65627313580244);
    assert!(err <= 1e-12, "half a Julian year from the declared epoch — short enough that the peak is at its own start: got {} want 110.65627313580244, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// fifteen years — long enough to contain a whole cycle, so the peak is the next maximum at the mean amplitude
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(473364000.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 193.85802469135803);
    assert!(err <= 1e-12, "fifteen years — long enough to contain a whole cycle, so the peak is the next maximum at the mean amplitude: got {} want 193.85802469135803, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the earliest epoch, 2018-01-01 — the window contains cycle 25's own maximum
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(157788000.0), Time::new(568080000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 174.09864172839502);
    assert!(err <= 1e-12, "the earliest epoch, 2018-01-01 — the window contains cycle 25's own maximum: got {} want 174.09864172839502, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the latest epoch, 2040-01-01
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Time::new(157788000.0), Time::new(1262304000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 147.97900299382718);
    assert!(err <= 1e-12, "the latest epoch, 2040-01-01: got {} want 147.97900299382718, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// an epoch near the next maximum, 2032-11-08
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Time::new(157788000.0), Time::new(1036800000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 193.85802469135803);
    assert!(err <= 1e-12, "an epoch near the next maximum, 2032-11-08: got {} want 193.85802469135803, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// a 2039 window straddling the amplitude handover — the maximum is a limit, not a value at any knot
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Time::new(63115200.0), Time::new(1257984000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 82.38214311111112);
    assert!(err <= 1e-12, "a 2039 window straddling the amplitude handover — the maximum is a limit, not a value at any knot: got {} want 82.38214311111112, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `the declared case — the highest sustained level a 2027 five-year mission meets` and the declared domain 60 … 400.
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
        if let Err(f) = model::evaluate(Time::new(157788000.0 * scale), Time::new(852076800.0)) {
            refused.push(format!("lead x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Time::new(157788000.0), Time::new(852076800.0 * scale)) {
            refused.push(format!("epoch x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_window_peak_level refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 60 … 400 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Time::new(157788000.0 * scale), Time::new(852076800.0)) {
            assert!(v.get().is_finite(), "sw_window_peak_level produced a value that is not a number");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_window_peak_level answered {}, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Time::new(157788000.0), Time::new(852076800.0 * scale)) {
            assert!(v.get().is_finite(), "sw_window_peak_level produced a value that is not a number");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_window_peak_level answered {}, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Time::new(157788000.0), Time::new(852076800.0));
    let b = model::evaluate(Time::new(157788000.0), Time::new(852076800.0));
    match (a, b) {
        (Ok(x), Ok(y)) => assert!(x.get().to_bits() == y.get().to_bits(), "sw_window_peak_level is not deterministic: {} then {}", x.get(), y.get()),
        (Err(_), Err(_)) => {}
        _ => panic!("sw_window_peak_level refused on one call and answered on the other"),
    }
}

