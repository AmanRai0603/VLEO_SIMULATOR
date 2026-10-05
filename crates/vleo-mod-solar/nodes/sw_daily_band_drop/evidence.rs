// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_daily_band_drop`.
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

/// at the declared window's own sustained cold level, 69.63 — just under the bottom knot, so this is the clamp and it is the case that unblocked the chain
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(69.62809104)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.387);
    assert!(err <= 1e-12, "at the declared window's own sustained cold level, 69.63 — just under the bottom knot, so this is the clamp and it is the case that unblocked the chain: got {} want 4.387, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the 70 sfu knot, exactly — a transcription check, and the value the clamp holds
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(70.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.387);
    assert!(err <= 1e-12, "the 70 sfu knot, exactly — a transcription check, and the value the clamp holds: got {} want 4.387, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the 120 sfu knot, exactly — a transcription check on a middle pair
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(120.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 23.2593);
    assert!(err <= 1e-12, "the 120 sfu knot, exactly — a transcription check on a middle pair: got {} want 23.2593, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// halfway between the 120 and 145 knots — the interpolation, on round numbers
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(132.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 27.82225);
    assert!(err <= 1e-9, "halfway between the 120 and 145 knots — the interpolation, on round numbers: got {} want 27.82225, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// far above the top of the table — the clamp
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(250.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 40.7778);
    assert!(err <= 1e-12, "far above the top of the table — the clamp: got {} want 40.7778, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «at the declared window's own sustained cold level, 69.63 — just under the bottom knot, so this is the clamp and it is the case that unblocked the chain.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(69.62809104)).expect("at the declared window's own sustained cold level, 69.63 — just under the bottom knot, so this is the clamp and it is the case that unblocked the chain.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 4.387);
    assert!(err <= 1e-12, "at the declared window's own sustained cold level, 69.63 — just under the bottom knot, so this is the clamp and it is the case that unblocked the chain.: got {} and the author's code gave 4.387; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the 70 sfu knot, exactly — a transcription check, and the value the clamp holds.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(70.0)).expect("the 70 sfu knot, exactly — a transcription check, and the value the clamp holds.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 4.387);
    assert!(err <= 1e-12, "the 70 sfu knot, exactly — a transcription check, and the value the clamp holds.: got {} and the author's code gave 4.387; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the 120 sfu knot, exactly — a transcription check on a middle pair.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(120.0)).expect("the 120 sfu knot, exactly — a transcription check on a middle pair.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 23.2593);
    assert!(err <= 1e-12, "the 120 sfu knot, exactly — a transcription check on a middle pair.: got {} and the author's code gave 23.2593; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «halfway between the 120 and 145 knots — the interpolation, on round numbers.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(132.5)).expect("halfway between the 120 and 145 knots — the interpolation, on round numbers.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 27.82225);
    assert!(err <= 1e-9, "halfway between the 120 and 145 knots — the interpolation, on round numbers.: got {} and the author's code gave 27.82225; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «far above the top of the table — the clamp.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(250.0)).expect("far above the top of the table — the clamp.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 40.7778);
    assert!(err <= 1e-12, "far above the top of the table — the clamp.: got {} and the author's code gave 40.7778; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Ratio::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 36 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_daily_band_drop::evaluate;
    assert_eq!(evaluate(34.81404552).map(f64::to_bits), Ok(0x40118c49ba5e353f), "at (34.81404552)");
    assert_eq!(evaluate(62.665281936).map(f64::to_bits), Ok(0x40118c49ba5e353f), "at (62.665281936)");
    assert_eq!(evaluate(68.9318101296).map(f64::to_bits), Ok(0x40118c49ba5e353f), "at (68.9318101296)");
    assert_eq!(evaluate(69.62809104).map(f64::to_bits), Ok(0x40118c49ba5e353f), "at (69.62809104)");
    assert_eq!(evaluate(70.3243719504).map(f64::to_bits), Ok(0x40120d02864dcb50), "at (70.3243719504)");
    assert_eq!(evaluate(76.590900144).map(f64::to_bits), Ok(0x401bc3c83ac14a64), "at (76.590900144)");
    assert_eq!(evaluate(139.25618208).map(f64::to_bits), Ok(0x403e49db1d7a0b2a), "at (139.25618208)");
    assert_eq!(evaluate(35.0).map(f64::to_bits), Ok(0x40118c49ba5e353f), "at (35.0)");
    assert_eq!(evaluate(63.0).map(f64::to_bits), Ok(0x40118c49ba5e353f), "at (63.0)");
    assert_eq!(evaluate(69.3).map(f64::to_bits), Ok(0x40118c49ba5e353f), "at (69.3)");
    assert_eq!(evaluate(70.0).map(f64::to_bits), Ok(0x40118c49ba5e353f), "at (70.0)");
    assert_eq!(evaluate(70.7).map(f64::to_bits), Ok(0x4012a2126799fb78), "at (70.7)");
    assert_eq!(evaluate(77.0).map(f64::to_bits), Ok(0x401c66207eb3f370), "at (77.0)");
    assert_eq!(evaluate(140.0).map(f64::to_bits), Ok(0x403e8f5d78811b1d), "at (140.0)");
    assert_eq!(evaluate(60.0).map(f64::to_bits), Ok(0x40118c49ba5e353f), "at (60.0)");
    assert_eq!(evaluate(108.0).map(f64::to_bits), Ok(0x4033591fb3fa6df0), "at (108.0)");
    assert_eq!(evaluate(118.8).map(f64::to_bits), Ok(0x4036de414e7ee914), "at (118.8)");
    assert_eq!(evaluate(120.0).map(f64::to_bits), Ok(0x403742617c1bda51), "at (120.0)");
    assert_eq!(evaluate(121.2).map(f64::to_bits), Ok(0x4037b285157e1686), "at (121.2)");
    assert_eq!(evaluate(132.0).map(f64::to_bits), Ok(0x403ba3c579f23465), "at (132.0)");
    assert_eq!(evaluate(240.0).map(f64::to_bits), Ok(0x4044638ef34d6a16), "at (240.0)");
    assert_eq!(evaluate(66.25).map(f64::to_bits), Ok(0x40118c49ba5e353f), "at (66.25)");
    assert_eq!(evaluate(119.25).map(f64::to_bits), Ok(0x403703cd5f99c38b), "at (119.25)");
    assert_eq!(evaluate(131.175).map(f64::to_bits), Ok(0x403b56ad007eab02), "at (131.175)");
    assert_eq!(evaluate(132.5).map(f64::to_bits), Ok(0x403bd27ef9db22d0), "at (132.5)");
    assert_eq!(evaluate(133.825).map(f64::to_bits), Ok(0x403c4e50f3379a9f), "at (133.825)");
    assert_eq!(evaluate(145.75).map(f64::to_bits), Ok(0x404040e410b630a9), "at (145.75)");
    assert_eq!(evaluate(265.0).map(f64::to_bits), Ok(0x4044638ef34d6a16), "at (265.0)");
    assert_eq!(evaluate(125.0).map(f64::to_bits), Ok(0x403915a07b352a84), "at (125.0)");
    assert_eq!(evaluate(225.0).map(f64::to_bits), Ok(0x4044638ef34d6a16), "at (225.0)");
    assert_eq!(evaluate(247.5).map(f64::to_bits), Ok(0x4044638ef34d6a16), "at (247.5)");
    assert_eq!(evaluate(250.0).map(f64::to_bits), Ok(0x4044638ef34d6a16), "at (250.0)");
    assert_eq!(evaluate(252.5).map(f64::to_bits), Ok(0x4044638ef34d6a16), "at (252.5)");
    assert_eq!(evaluate(275.0).map(f64::to_bits), Ok(0x4044638ef34d6a16), "at (275.0)");
    assert_eq!(evaluate(500.0).map(f64::to_bits), Ok(0x4044638ef34d6a16), "at (500.0)");
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
/// Derived from `at the declared window's own sustained cold level, 69.63 — just under the bottom knot, so this is the clamp and it is the case that unblocked the chain` and the declared domain 0 … 120.
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
        if let Err(f) = model::evaluate(Ratio::new(69.62809104 * scale)) {
            refused.push(format!("level x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_daily_band_drop refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 120 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(69.62809104 * scale)) {
            assert!(v.get().is_finite(), "sw_daily_band_drop produced a value that is not a number for dF107_day_low");
            assert!(v.get() >= 0.0 && v.get() <= 120.0, "sw_daily_band_drop answered {} for dF107_day_low, outside its declared domain 0 … 120 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(69.62809104));
    let b = model::evaluate(Ratio::new(69.62809104));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_daily_band_drop is not deterministic for dF107_day_low: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_daily_band_drop refused on one call and answered on the other"),
    }
}

