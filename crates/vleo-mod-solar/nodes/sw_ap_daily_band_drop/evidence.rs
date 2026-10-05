// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_ap_daily_band_drop`.
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

/// at the declared window's own sustained quiet level, Ap 17.50 — a genuine interpolation between the 15 and 20 knots, unlike its high-tail twin's clamp
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(17.49546836)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 12.5635147395);
    assert!(err <= 1e-9, "at the declared window's own sustained quiet level, Ap 17.50 — a genuine interpolation between the 15 and 20 knots, unlike its high-tail twin's clamp: got {} want 12.5635147395, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the Ap 15 knot, exactly — a transcription check on the lower bracket of that interpolation
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(15.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 10.837);
    assert!(err <= 1e-12, "the Ap 15 knot, exactly — a transcription check on the lower bracket of that interpolation: got {} want 10.837, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the Ap 11 knot, exactly — a transcription check where the high tail is twice this value
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(11.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 7.5556);
    assert!(err <= 1e-12, "the Ap 11 knot, exactly — a transcription check where the high tail is twice this value: got {} want 7.5556, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// between the Ap 20 and Ap 24 knots — the interpolation on the upper half of the ladder
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(22.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 15.99445);
    assert!(err <= 1e-9, "between the Ap 20 and Ap 24 knots — the interpolation on the upper half of the ladder: got {} want 15.99445, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// below the bottom of the table — the clamp
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(1.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 2.9704);
    assert!(err <= 1e-12, "below the bottom of the table — the clamp: got {} want 2.9704, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «at the declared window's own sustained quiet level, Ap 17.50 — a genuine interpolation between the 15 and 20 knots, unlike its high-tail twin's clamp.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(17.49546836)).expect("at the declared window's own sustained quiet level, Ap 17.50 — a genuine interpolation between the 15 and 20 knots, unlike its high-tail twin's clamp.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 12.5635147395);
    assert!(err <= 1e-9, "at the declared window's own sustained quiet level, Ap 17.50 — a genuine interpolation between the 15 and 20 knots, unlike its high-tail twin's clamp.: got {} and the author's code gave 12.5635147395; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the Ap 15 knot, exactly — a transcription check on the lower bracket of that interpolation.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(15.0)).expect("the Ap 15 knot, exactly — a transcription check on the lower bracket of that interpolation.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 10.837);
    assert!(err <= 1e-12, "the Ap 15 knot, exactly — a transcription check on the lower bracket of that interpolation.: got {} and the author's code gave 10.837; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the Ap 11 knot, exactly — a transcription check where the high tail is twice this value.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(11.0)).expect("the Ap 11 knot, exactly — a transcription check where the high tail is twice this value.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 7.5556);
    assert!(err <= 1e-12, "the Ap 11 knot, exactly — a transcription check where the high tail is twice this value.: got {} and the author's code gave 7.5556; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «between the Ap 20 and Ap 24 knots — the interpolation on the upper half of the ladder.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(22.0)).expect("between the Ap 20 and Ap 24 knots — the interpolation on the upper half of the ladder.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 15.99445);
    assert!(err <= 1e-9, "between the Ap 20 and Ap 24 knots — the interpolation on the upper half of the ladder.: got {} and the author's code gave 15.99445; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «below the bottom of the table — the clamp.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(1.0)).expect("below the bottom of the table — the clamp.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 2.9704);
    assert!(err <= 1e-12, "below the bottom of the table — the clamp.: got {} and the author's code gave 2.9704; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «a value that is not a number: the node must refuse it, as its code does», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Ratio::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "a value that is not a number: the node must refuse it, as its code does: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 36 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_ap_daily_band_drop::evaluate;
    assert_eq!(evaluate(8.74773418).map(f64::to_bits), Ok(0x401853d50c1bbf86), "at (8.74773418)");
    assert_eq!(evaluate(15.745921524).map(f64::to_bits), Ok(0x4026b4c604e1a44d), "at (15.745921524)");
    assert_eq!(evaluate(17.3205136764).map(f64::to_bits), Ok(0x4028e28b81657cbe), "at (17.3205136764)");
    assert_eq!(evaluate(17.49546836).map(f64::to_bits), Ok(0x4029208501025be6), "at (17.49546836)");
    assert_eq!(evaluate(17.6704230436).map(f64::to_bits), Ok(0x40295e7e809f3b0f), "at (17.6704230436)");
    assert_eq!(evaluate(19.245015196).map(f64::to_bits), Ok(0x402b8c43fd231380), "at (19.245015196)");
    assert_eq!(evaluate(34.99093672).map(f64::to_bits), Ok(0x403504bc6a7ef9db), "at (34.99093672)");
    assert_eq!(evaluate(7.5).map(f64::to_bits), Ok(0x4014f240b780346e), "at (7.5)");
    assert_eq!(evaluate(13.5).map(f64::to_bits), Ok(0x40233683e425aee6), "at (13.5)");
    assert_eq!(evaluate(14.85).map(f64::to_bits), Ok(0x40256d8a86d71f36), "at (14.85)");
    assert_eq!(evaluate(15.0).map(f64::to_bits), Ok(0x4025ac8b43958106), "at (15.0)");
    assert_eq!(evaluate(15.15).map(f64::to_bits), Ok(0x4025e1adc8fb86f4), "at (15.15)");
    assert_eq!(evaluate(16.5).map(f64::to_bits), Ok(0x4027bfe47991bc56), "at (16.5)");
    assert_eq!(evaluate(30.0).map(f64::to_bits), Ok(0x403504bc6a7ef9db), "at (30.0)");
    assert_eq!(evaluate(5.5).map(f64::to_bits), Ok(0x400ef3a92a305532), "at (5.5)");
    assert_eq!(evaluate(9.9).map(f64::to_bits), Ok(0x401b57e4b17e4b18), "at (9.9)");
    assert_eq!(evaluate(10.89).map(f64::to_bits), Ok(0x401def3af480ff28), "at (10.89)");
    assert_eq!(evaluate(11.0).map(f64::to_bits), Ok(0x401e38ef34d6a162), "at (11.0)");
    assert_eq!(evaluate(11.11).map(f64::to_bits), Ok(0x401e9556b00ffda4), "at (11.11)");
    assert_eq!(evaluate(12.100000000000001).map(f64::to_bits), Ok(0x4020ea7d028a1dfc), "at (12.100000000000001)");
    assert_eq!(evaluate(22.0).map(f64::to_bits), Ok(0x402ffd288ce703b0), "at (22.0)");
    assert_eq!(evaluate(11.0).map(f64::to_bits), Ok(0x401e38ef34d6a162), "at (11.0)");
    assert_eq!(evaluate(19.8).map(f64::to_bits), Ok(0x402c50dbf0563ed2), "at (19.8)");
    assert_eq!(evaluate(21.78).map(f64::to_bits), Ok(0x402f9d84c271fff8), "at (21.78)");
    assert_eq!(evaluate(22.0).map(f64::to_bits), Ok(0x402ffd288ce703b0), "at (22.0)");
    assert_eq!(evaluate(22.22).map(f64::to_bits), Ok(0x40302e662bae03b4), "at (22.22)");
    assert_eq!(evaluate(24.200000000000003).map(f64::to_bits), Ok(0x40320672da122fae), "at (24.200000000000003)");
    assert_eq!(evaluate(44.0).map(f64::to_bits), Ok(0x403504bc6a7ef9db), "at (44.0)");
    assert_eq!(evaluate(0.5).map(f64::to_bits), Ok(0x4007c36113404ea5), "at (0.5)");
    assert_eq!(evaluate(0.9).map(f64::to_bits), Ok(0x4007c36113404ea5), "at (0.9)");
    assert_eq!(evaluate(0.99).map(f64::to_bits), Ok(0x4007c36113404ea5), "at (0.99)");
    assert_eq!(evaluate(1.0).map(f64::to_bits), Ok(0x4007c36113404ea5), "at (1.0)");
    assert_eq!(evaluate(1.01).map(f64::to_bits), Ok(0x4007c36113404ea5), "at (1.01)");
    assert_eq!(evaluate(1.1).map(f64::to_bits), Ok(0x4007c36113404ea5), "at (1.1)");
    assert_eq!(evaluate(2.0).map(f64::to_bits), Ok(0x4007c36113404ea5), "at (2.0)");
    assert!(matches!(evaluate(f64::NAN), Err(MethodError::Refused(_))), "at (f64::NAN)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `at the declared window's own sustained quiet level, Ap 17.50 — a genuine interpolation between the 15 and 20 knots, unlike its high-tail twin's clamp` and the declared domain 0 … 150.
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
        if let Err(f) = model::evaluate(Ratio::new(17.49546836 * scale)) {
            refused.push(format!("level x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_ap_daily_band_drop refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 150 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(17.49546836 * scale)) {
            assert!(v.get().is_finite(), "sw_ap_daily_band_drop produced a value that is not a number for dAp_day_low");
            assert!(v.get() >= 0.0 && v.get() <= 150.0, "sw_ap_daily_band_drop answered {} for dAp_day_low, outside its declared domain 0 … 150 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(17.49546836));
    let b = model::evaluate(Ratio::new(17.49546836));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_ap_daily_band_drop is not deterministic for dAp_day_low: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_ap_daily_band_drop refused on one call and answered on the other"),
    }
}

