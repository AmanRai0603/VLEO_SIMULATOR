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

/// the author's case «at the declared window's own sustained level, 104.07 — an interpolation between the 100 and 120 knots.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(104.07110896)).expect("at the declared window's own sustained level, 104.07 — an interpolation between the 100 and 120 knots.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 20.0721663388);
    assert!(err <= 1e-9, "at the declared window's own sustained level, 104.07 — an interpolation between the 100 and 120 knots.: got {} and the author's code gave 20.0721663388; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the 100 sfu knot, exactly — a transcription check on one of the seven pairs.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(100.0)).expect("the 100 sfu knot, exactly — a transcription check on one of the seven pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 18.2741);
    assert!(err <= 1e-12, "the 100 sfu knot, exactly — a transcription check on one of the seven pairs.: got {} and the author's code gave 18.2741; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the 145 sfu knot, exactly — near where the study's own sample sat.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(145.0)).expect("the 145 sfu knot, exactly — near where the study's own sample sat.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 34.6148);
    assert!(err <= 1e-12, "the 145 sfu knot, exactly — near where the study's own sample sat.: got {} and the author's code gave 34.6148; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «halfway between the 100 and 120 knots — the interpolation, on round numbers.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(112.5)).expect("halfway between the 100 and 120 knots — the interpolation, on round numbers.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 23.7949125);
    assert!(err <= 1e-9, "halfway between the 100 and 120 knots — the interpolation, on round numbers.: got {} and the author's code gave 23.7949125; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «below the bottom of the table — the clamp, not an extrapolation toward zero.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(60.0)).expect("below the bottom of the table — the clamp, not an extrapolation toward zero.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 4.6259);
    assert!(err <= 1e-12, "below the bottom of the table — the clamp, not an extrapolation toward zero.: got {} and the author's code gave 4.6259; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «far above the top of the table — the clamp again, which is where a cycle-maximum window would land.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Ratio::new(300.0)).expect("far above the top of the table — the clamp again, which is where a cycle-maximum window would land.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 46.9259);
    assert!(err <= 1e-12, "far above the top of the table — the clamp again, which is where a cycle-maximum window would land.: got {} and the author's code gave 46.9259; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Ratio::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 43 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_daily_band_spread::evaluate;
    assert_eq!(evaluate(52.03555448).map(f64::to_bits), Ok(0x401280ebedfa43fe), "at (52.03555448)");
    assert_eq!(evaluate(93.66399806400001).map(f64::to_bits), Ok(0x402ecdb5fe3d639b), "at (93.66399806400001)");
    assert_eq!(evaluate(103.03039787040001).map(f64::to_bits), Ok(0x40339cce27ad18bd), "at (103.03039787040001)");
    assert_eq!(evaluate(104.07110896).map(f64::to_bits), Ok(0x403412797e4118db), "at (104.07110896)");
    assert_eq!(evaluate(105.1118200496).map(f64::to_bits), Ok(0x40348824d4d518fa), "at (105.1118200496)");
    assert_eq!(evaluate(114.47821985600001).map(f64::to_bits), Ok(0x4038ab2ae0091a12), "at (114.47821985600001)");
    assert_eq!(evaluate(208.14221792).map(f64::to_bits), Ok(0x40474c2a422ddc2b), "at (208.14221792)");
    assert_eq!(evaluate(50.0).map(f64::to_bits), Ok(0x401280ebedfa43fe), "at (50.0)");
    assert_eq!(evaluate(90.0).map(f64::to_bits), Ok(0x402b7b45c4be99bc), "at (90.0)");
    assert_eq!(evaluate(99.0).map(f64::to_bits), Ok(0x4031d21dc3a6faf3), "at (99.0)");
    assert_eq!(evaluate(100.0).map(f64::to_bits), Ok(0x4032462b6ae7d567), "at (100.0)");
    assert_eq!(evaluate(101.0).map(f64::to_bits), Ok(0x4032b73c60029f17), "at (101.0)");
    assert_eq!(evaluate(110.00000000000001).map(f64::to_bits), Ok(0x4036b0d4fdf3b647), "at (110.00000000000001)");
    assert_eq!(evaluate(200.0).map(f64::to_bits), Ok(0x4046928df4a5f24f), "at (200.0)");
    assert_eq!(evaluate(72.5).map(f64::to_bits), Ok(0x401711ae5a6293ba), "at (72.5)");
    assert_eq!(evaluate(130.5).map(f64::to_bits), Ok(0x403e42b0a6fc58ac), "at (130.5)");
    assert_eq!(evaluate(143.55).map(f64::to_bits), Ok(0x404116f59f53edbf), "at (143.55)");
    assert_eq!(evaluate(145.0).map(f64::to_bits), Ok(0x40414eb1c432ca58), "at (145.0)");
    assert_eq!(evaluate(146.45).map(f64::to_bits), Ok(0x4041744bb47b0e05), "at (146.45)");
    assert_eq!(evaluate(159.5).map(f64::to_bits), Ok(0x4042c6b527056f1c), "at (159.5)");
    assert_eq!(evaluate(290.0).map(f64::to_bits), Ok(0x40477683e425aee6), "at (290.0)");
    assert_eq!(evaluate(56.25).map(f64::to_bits), Ok(0x401280ebedfa43fe), "at (56.25)");
    assert_eq!(evaluate(101.25).map(f64::to_bits), Ok(0x4032d3809d495183), "at (101.25)");
    assert_eq!(evaluate(111.375).map(f64::to_bits), Ok(0x40374c4c4ef88b97), "at (111.375)");
    assert_eq!(evaluate(112.5).map(f64::to_bits), Ok(0x4037cb7f62b6ae7d), "at (112.5)");
    assert_eq!(evaluate(113.625).map(f64::to_bits), Ok(0x40384ab27674d163), "at (113.625)");
    assert_eq!(evaluate(123.75000000000001).map(f64::to_bits), Ok(0x403c3bc74fb549fa), "at (123.75000000000001)");
    assert_eq!(evaluate(225.0).map(f64::to_bits), Ok(0x40477683e425aee6), "at (225.0)");
    assert_eq!(evaluate(30.0).map(f64::to_bits), Ok(0x401280ebedfa43fe), "at (30.0)");
    assert_eq!(evaluate(54.0).map(f64::to_bits), Ok(0x401280ebedfa43fe), "at (54.0)");
    assert_eq!(evaluate(59.4).map(f64::to_bits), Ok(0x401280ebedfa43fe), "at (59.4)");
    assert_eq!(evaluate(60.0).map(f64::to_bits), Ok(0x401280ebedfa43fe), "at (60.0)");
    assert_eq!(evaluate(60.6).map(f64::to_bits), Ok(0x401280ebedfa43fe), "at (60.6)");
    assert_eq!(evaluate(66.0).map(f64::to_bits), Ok(0x401280ebedfa43fe), "at (66.0)");
    assert_eq!(evaluate(120.0).map(f64::to_bits), Ok(0x403b1b7e90ff9724), "at (120.0)");
    assert_eq!(evaluate(150.0).map(f64::to_bits), Ok(0x4041d05aa87b6d17), "at (150.0)");
    assert_eq!(evaluate(270.0).map(f64::to_bits), Ok(0x40477683e425aee6), "at (270.0)");
    assert_eq!(evaluate(297.0).map(f64::to_bits), Ok(0x40477683e425aee6), "at (297.0)");
    assert_eq!(evaluate(300.0).map(f64::to_bits), Ok(0x40477683e425aee6), "at (300.0)");
    assert_eq!(evaluate(303.0).map(f64::to_bits), Ok(0x40477683e425aee6), "at (303.0)");
    assert_eq!(evaluate(330.0).map(f64::to_bits), Ok(0x40477683e425aee6), "at (330.0)");
    assert_eq!(evaluate(600.0).map(f64::to_bits), Ok(0x40477683e425aee6), "at (600.0)");
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

