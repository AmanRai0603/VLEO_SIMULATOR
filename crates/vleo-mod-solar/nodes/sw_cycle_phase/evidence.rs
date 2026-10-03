// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_cycle_phase`.
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

/// the declared mission epoch, 2027-01-01 — 2588 days into cycle 25
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.6193669438022621);
    assert!(err <= 1e-12, "the declared mission epoch, 2027-01-01 — 2588 days into cycle 25: got {} want 0.6193669438022621, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the day cycle 25 begins — phase must be exactly zero
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(628473600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.0);
    assert!(err <= 1e-12, "the day cycle 25 begins — phase must be exactly zero: got {} want 0.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// half a mean cycle after the start — phase one half, to the last bit float allows
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(808983072.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.4999999999999999);
    assert!(err <= 1e-12, "half a mean cycle after the start — phase one half, to the last bit float allows: got {} want 0.4999999999999999, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «the declared mission epoch, 2027-01-01 — 2588 days into cycle 25.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Time::new(852076800.0)).expect("the declared mission epoch, 2027-01-01 — 2588 days into cycle 25.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.6193669438022621);
    assert!(err <= 1e-12, "the declared mission epoch, 2027-01-01 — 2588 days into cycle 25.: got {} and the author's code gave 0.6193669438022621; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the day cycle 25 begins — phase must be exactly zero.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Time::new(628473600.0)).expect("the day cycle 25 begins — phase must be exactly zero.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.0);
    assert!(err <= 1e-12, "the day cycle 25 begins — phase must be exactly zero.: got {} and the author's code gave 0.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «half a mean cycle after the start — phase one half, to the last bit float allows.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Time::new(808983072.0)).expect("half a mean cycle after the start — phase one half, to the last bit float allows.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.4999999999999999);
    assert!(err <= 1e-12, "half a mean cycle after the start — phase one half, to the last bit float allows.: got {} and the author's code gave 0.4999999999999999; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Time::new(0.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Time::new(1000000000.0000001));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 23 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_cycle_phase::evaluate;
    assert!(matches!(evaluate(426038400.0), Err(MethodError::Refused(_))), "at (426038400.0)");
    assert_eq!(evaluate(766869120.0).map(f64::to_bits), Ok(0x3fd888c1bfdd1a41), "at (766869120.0)");
    assert_eq!(evaluate(843556032.0).map(f64::to_bits), Ok(0x3fe31081a66074a7), "at (843556032.0)");
    assert_eq!(evaluate(852076800.0).map(f64::to_bits), Ok(0x3fe3d1da9ffb557d), "at (852076800.0)");
    assert_eq!(evaluate(860597568.0).map(f64::to_bits), Ok(0x3fe4933399963653), "at (860597568.0)");
    assert_eq!(evaluate(937284480.0000001).map(f64::to_bits), Ok(0x3feb5f5460081ddc), "at (937284480.0000001)");
    assert!(matches!(evaluate(1704153600.0), Err(MethodError::Refused(_))), "at (1704153600.0)");
    assert!(matches!(evaluate(314236800.0), Err(MethodError::Refused(_))), "at (314236800.0)");
    assert!(matches!(evaluate(565626240.0), Err(MethodError::Refused(_))), "at (565626240.0)");
    assert!(matches!(evaluate(622188864.0), Err(MethodError::Refused(_))), "at (622188864.0)");
    assert_eq!(evaluate(628473600.0).map(f64::to_bits), Ok(0x0000000000000000), "at (628473600.0)");
    assert_eq!(evaluate(634758336.0).map(f64::to_bits), Ok(0x3f91d37d14a54715), "at (634758336.0)");
    assert_eq!(evaluate(691320960.0).map(f64::to_bits), Ok(0x3fc6485c59ce98da), "at (691320960.0)");
    assert!(matches!(evaluate(1256947200.0), Err(MethodError::Refused(_))), "at (1256947200.0)");
    assert!(matches!(evaluate(404491536.0), Err(MethodError::Refused(_))), "at (404491536.0)");
    assert_eq!(evaluate(728084764.8000001).map(f64::to_bits), Ok(0x3fd1a89e9fe58064), "at (728084764.8000001)");
    assert_eq!(evaluate(800893241.28).map(f64::to_bits), Ok(0x3fde90dca996f33b), "at (800893241.28)");
    assert_eq!(evaluate(808983072.0).map(f64::to_bits), Ok(0x3fe0000000000000), "at (808983072.0)");
    assert_eq!(evaluate(817072902.72).map(f64::to_bits), Ok(0x3fe0b791ab348662), "at (817072902.72)");
    assert_eq!(evaluate(889881379.2).map(f64::to_bits), Ok(0x3fe72bb0b00d3fd1), "at (889881379.2)");
    assert!(matches!(evaluate(1617966144.0), Err(MethodError::Refused(_))), "at (1617966144.0)");
    assert!(matches!(evaluate(0.0), Err(MethodError::Refused(_))), "at (0.0)");
    assert!(matches!(evaluate(1000000000.0000001), Err(MethodError::Refused(_))), "at (1000000000.0000001)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `the declared mission epoch, 2027-01-01 — 2588 days into cycle 25` and the declared domain 0 … 1.
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
        if let Err(f) = model::evaluate(Time::new(852076800.0 * scale)) {
            refused.push(format!("epoch x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_cycle_phase refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 1 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Time::new(852076800.0 * scale)) {
            assert!(v.get().is_finite(), "sw_cycle_phase produced a value that is not a number for phase");
            assert!(v.get() >= 0.0 && v.get() <= 1.0, "sw_cycle_phase answered {} for phase, outside its declared domain 0 … 1 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Time::new(852076800.0));
    let b = model::evaluate(Time::new(852076800.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_cycle_phase is not deterministic for phase: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_cycle_phase refused on one call and answered on the other"),
    }
}

