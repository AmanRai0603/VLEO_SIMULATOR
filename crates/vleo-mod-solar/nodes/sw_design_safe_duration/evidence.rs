// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_design_safe_duration`.
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

/// the G2 bound, Ap 80 — exceeded beyond a 0.7365-year mission
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(80.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 23243127.613750264);
    assert!(err <= 1e-9, "the G2 bound, Ap 80 — exceeded beyond a 0.7365-year mission: got {} want 23243127.613750264, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the G1 bound, Ap 48 — exceeded beyond a 0.3370-year mission
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(48.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 10634679.615335282);
    assert!(err <= 1e-9, "the G1 bound, Ap 48 — exceeded beyond a 0.3370-year mission: got {} want 10634679.615335282, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the G3 bound, Ap 132 — the declared design level — exceeded beyond 2.6242 years, against a declared mission of 5
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(132.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 82812533.26941039);
    assert!(err <= 1e-9, "the G3 bound, Ap 132 — the declared design level — exceeded beyond 2.6242 years, against a declared mission of 5: got {} want 82812533.26941039, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «the G2 bound, Ap 80 — exceeded beyond a 0.7365-year mission.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(80.0)).expect("the G2 bound, Ap 80 — exceeded beyond a 0.7365-year mission.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 23243127.613750264);
    assert!(err <= 1e-9, "the G2 bound, Ap 80 — exceeded beyond a 0.7365-year mission.: got {} and the author's code gave 23243127.613750264; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the G1 bound, Ap 48 — exceeded beyond a 0.3370-year mission.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(48.0)).expect("the G1 bound, Ap 48 — exceeded beyond a 0.3370-year mission.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 10634679.615335282);
    assert!(err <= 1e-9, "the G1 bound, Ap 48 — exceeded beyond a 0.3370-year mission.: got {} and the author's code gave 10634679.615335282; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the G3 bound, Ap 132 — the declared design level — exceeded beyond 2.6242 years, against a declared mission of 5.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(132.0)).expect("the G3 bound, Ap 132 — the declared design level — exceeded beyond 2.6242 years, against a declared mission of 5.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 82812533.26941039);
    assert!(err <= 1e-9, "the G3 bound, Ap 132 — the declared design level — exceeded beyond 2.6242 years, against a declared mission of 5.: got {} and the author's code gave 82812533.26941039; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(40.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(207.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 23 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_design_safe_duration::evaluate;
    assert!(matches!(evaluate(40.0), Err(MethodError::Refused(_))), "at (40.0)");
    assert_eq!(evaluate(72.0).map(f64::to_bits), Ok(0x41723b0bc56b3e48), "at (72.0)");
    assert_eq!(evaluate(79.2).map(f64::to_bits), Ok(0x4175bcbf09a96b87), "at (79.2)");
    assert_eq!(evaluate(80.0).map(f64::to_bits), Ok(0x41762a9779d1ebcc), "at (80.0)");
    assert_eq!(evaluate(80.8).map(f64::to_bits), Ok(0x41769a9aff86035e), "at (80.8)");
    assert_eq!(evaluate(88.0).map(f64::to_bits), Ok(0x417af3a6cb536c8b), "at (88.0)");
    assert!(matches!(evaluate(160.0), Err(MethodError::Refused(_))), "at (160.0)");
    assert!(matches!(evaluate(24.0), Err(MethodError::Refused(_))), "at (24.0)");
    assert!(matches!(evaluate(43.2), Err(MethodError::Refused(_))), "at (43.2)");
    assert_eq!(evaluate(47.519999999999996).map(f64::to_bits), Ok(0x41640c2b2248c343), "at (47.519999999999996)");
    assert_eq!(evaluate(48.0).map(f64::to_bits), Ok(0x416448b6f3b0d39e), "at (48.0)");
    assert_eq!(evaluate(48.480000000000004).map(f64::to_bits), Ok(0x416485f9a091885e), "at (48.480000000000004)");
    assert_eq!(evaluate(52.800000000000004).map(f64::to_bits), Ok(0x4166cee2c455ba1b), "at (52.800000000000004)");
    assert_eq!(evaluate(96.0).map(f64::to_bits), Ok(0x418062977e9ebf72), "at (96.0)");
    assert_eq!(evaluate(66.0).map(f64::to_bits), Ok(0x416f7d41560bbb7b), "at (66.0)");
    assert_eq!(evaluate(118.8).map(f64::to_bits), Ok(0x418c9a1006127bcd), "at (118.8)");
    assert_eq!(evaluate(130.68).map(f64::to_bits), Ok(0x41931e0e206af077), "at (130.68)");
    assert_eq!(evaluate(132.0).map(f64::to_bits), Ok(0x4193be79d513e051), "at (132.0)");
    assert_eq!(evaluate(133.32).map(f64::to_bits), Ok(0x41946427af06bc45), "at (133.32)");
    assert!(matches!(evaluate(145.20000000000002), Err(MethodError::Refused(_))), "at (145.20000000000002)");
    assert!(matches!(evaluate(264.0), Err(MethodError::Refused(_))), "at (264.0)");
    assert!(matches!(evaluate(40.0), Err(MethodError::Refused(_))), "at (40.0)");
    assert!(matches!(evaluate(207.0), Err(MethodError::Refused(_))), "at (207.0)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `the G2 bound, Ap 80 — exceeded beyond a 0.7365-year mission` and the declared domain 9467280 … 94672800.
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
        "sw_design_safe_duration refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 9467280 … 94672800 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
            assert!(v.get().is_finite(), "sw_design_safe_duration produced a value that is not a number for T_safe");
            assert!(v.get() >= 9467280.0 && v.get() <= 94672800.0, "sw_design_safe_duration answered {} for T_safe, outside its declared domain 9467280 … 94672800 — the guard did not stop it", v.get());
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
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_design_safe_duration is not deterministic for T_safe: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_design_safe_duration refused on one call and answered on the other"),
    }
}

