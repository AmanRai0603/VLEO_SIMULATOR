// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_exceedance_phase`.
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

/// the G2 bound, Ap 80 — median phase 0.576
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(80.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.5763024435);
    assert!(err <= 1e-12, "the G2 bound, Ap 80 — median phase 0.576: got {} want 0.5763024435, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the G1 bound, Ap 48 — the lowest level the G scale defines
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(48.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.571692024);
    assert!(err <= 1e-12, "the G1 bound, Ap 48 — the lowest level the G scale defines: got {} want 0.571692024, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the G3 bound, Ap 132 — the declared design level
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(132.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.6026970954);
    assert!(err <= 1e-12, "the G3 bound, Ap 132 — the declared design level: got {} want 0.6026970954, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «the G2 bound, Ap 80 — median phase 0.576.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(80.0)).expect("the G2 bound, Ap 80 — median phase 0.576.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.5763024435);
    assert!(err <= 1e-12, "the G2 bound, Ap 80 — median phase 0.576.: got {} and the author's code gave 0.5763024435; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the G1 bound, Ap 48 — the lowest level the G scale defines.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(48.0)).expect("the G1 bound, Ap 48 — the lowest level the G scale defines.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.571692024);
    assert!(err <= 1e-12, "the G1 bound, Ap 48 — the lowest level the G scale defines.: got {} and the author's code gave 0.571692024; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the G3 bound, Ap 132 — the declared design level.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(132.0)).expect("the G3 bound, Ap 132 — the declared design level.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.6026970954);
    assert!(err <= 1e-12, "the G3 bound, Ap 132 — the declared design level.: got {} and the author's code gave 0.6026970954; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 22 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_exceedance_phase::evaluate;
    assert_eq!(evaluate(40.0).map(f64::to_bits), Ok(0x3fe24b4d124ed979), "at (40.0)");
    assert_eq!(evaluate(72.0).map(f64::to_bits), Ok(0x3fe267a0a26635ae), "at (72.0)");
    assert_eq!(evaluate(79.2).map(f64::to_bits), Ok(0x3fe270201a6d37be), "at (79.2)");
    assert_eq!(evaluate(80.0).map(f64::to_bits), Ok(0x3fe27111d26dff15), "at (80.0)");
    assert_eq!(evaluate(80.8).map(f64::to_bits), Ok(0x3fe274656a714b77), "at (80.8)");
    assert_eq!(evaluate(88.0).map(f64::to_bits), Ok(0x3fe29255c28efae6), "at (88.0)");
    assert_eq!(evaluate(160.0).map(f64::to_bits), Ok(0x3fe3494b6b4463e1), "at (160.0)");
    assert_eq!(evaluate(24.0).map(f64::to_bits), Ok(0x3fe24b4d124ed979), "at (24.0)");
    assert_eq!(evaluate(43.2).map(f64::to_bits), Ok(0x3fe24b4d124ed979), "at (43.2)");
    assert_eq!(evaluate(47.519999999999996).map(f64::to_bits), Ok(0x3fe24b4d124ed979), "at (47.519999999999996)");
    assert_eq!(evaluate(48.0).map(f64::to_bits), Ok(0x3fe24b4d124ed979), "at (48.0)");
    assert_eq!(evaluate(48.480000000000004).map(f64::to_bits), Ok(0x3fe24bde1a4f5114), "at (48.480000000000004)");
    assert_eq!(evaluate(52.800000000000004).map(f64::to_bits), Ok(0x3fe250f762538584), "at (52.800000000000004)");
    assert_eq!(evaluate(96.0).map(f64::to_bits), Ok(0x3fe2b399b2aff6b6), "at (96.0)");
    assert_eq!(evaluate(66.0).map(f64::to_bits), Ok(0x3fe2608bbe605ea1), "at (66.0)");
    assert_eq!(evaluate(118.8).map(f64::to_bits), Ok(0x3fe312681f0df796), "at (118.8)");
    assert_eq!(evaluate(130.68).map(f64::to_bits), Ok(0x3fe343ce4a0bbf73), "at (130.68)");
    assert_eq!(evaluate(132.0).map(f64::to_bits), Ok(0x3fe3494b6b4463e1), "at (132.0)");
    assert_eq!(evaluate(133.32).map(f64::to_bits), Ok(0x3fe3494b6b4463e1), "at (133.32)");
    assert_eq!(evaluate(145.20000000000002).map(f64::to_bits), Ok(0x3fe3494b6b4463e1), "at (145.20000000000002)");
    assert_eq!(evaluate(264.0).map(f64::to_bits), Ok(0x3fe3494b6b4463e1), "at (264.0)");
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
/// Derived from `the G2 bound, Ap 80 — median phase 0.576` and the declared domain 0 … 1.
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
        if let Err(f) = model::evaluate(Ratio::new(80.0 * scale)) {
            refused.push(format!("ap_design x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_exceedance_phase refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 1 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(80.0 * scale)) {
            assert!(v.get().is_finite(), "sw_exceedance_phase produced a value that is not a number for P_exc");
            assert!(v.get() >= 0.0 && v.get() <= 1.0, "sw_exceedance_phase answered {} for P_exc, outside its declared domain 0 … 1 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(80.0));
    let b = model::evaluate(Ratio::new(80.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_exceedance_phase is not deterministic for P_exc: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_exceedance_phase refused on one call and answered on the other"),
    }
}

