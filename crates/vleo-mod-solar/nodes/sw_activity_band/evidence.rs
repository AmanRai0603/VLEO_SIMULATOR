// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_activity_band`.
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

/// the record's quietest day, 64 sfu — band 1, low
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(64.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "the record's quietest day, 64 sfu — band 1, low: got {} want 1.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 89.9 sfu — still low, the edge is not yet reached
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(89.9)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "89.9 sfu — still low, the edge is not yet reached: got {} want 1.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 90 sfu exactly — moderate begins AT the edge, not after it
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(90.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "90 sfu exactly — moderate begins AT the edge, not after it: got {} want 2.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 129.9 sfu — still moderate
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(129.9)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "129.9 sfu — still moderate: got {} want 2.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 130 sfu exactly — elevated begins
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(130.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "130 sfu exactly — elevated begins: got {} want 3.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 169.9 sfu — still elevated
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(169.9)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "169.9 sfu — still elevated: got {} want 3.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 170 sfu exactly — high begins
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(170.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "170 sfu exactly — high begins: got {} want 4.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the record's largest day, 343 sfu — band 4, which is unbounded above
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(343.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "the record's largest day, 343 sfu — band 4, which is unbounded above: got {} want 4.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the design value this subsystem publishes, 228 sfu — band 4, the top one
///
/// Provenance: `published-source`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(228.1374378829)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "the design value this subsystem publishes, 228 sfu — band 4, the top one: got {} want 4.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «the record's quietest day, 64 sfu — band 1, low.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(64.0)).expect("the record's quietest day, 64 sfu — band 1, low.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "the record's quietest day, 64 sfu — band 1, low.: got {} and the author's code gave 1.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «89.9 sfu — still low, the edge is not yet reached.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(89.9)).expect("89.9 sfu — still low, the edge is not yet reached.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "89.9 sfu — still low, the edge is not yet reached.: got {} and the author's code gave 1.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «90 sfu exactly — moderate begins AT the edge, not after it.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(90.0)).expect("90 sfu exactly — moderate begins AT the edge, not after it.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "90 sfu exactly — moderate begins AT the edge, not after it.: got {} and the author's code gave 2.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «129.9 sfu — still moderate.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(129.9)).expect("129.9 sfu — still moderate.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "129.9 sfu — still moderate.: got {} and the author's code gave 2.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «130 sfu exactly — elevated begins.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(130.0)).expect("130 sfu exactly — elevated begins.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "130 sfu exactly — elevated begins.: got {} and the author's code gave 3.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «169.9 sfu — still elevated.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Ratio::new(169.9)).expect("169.9 sfu — still elevated.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "169.9 sfu — still elevated.: got {} and the author's code gave 3.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «170 sfu exactly — high begins.», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Ratio::new(170.0)).expect("170 sfu exactly — high begins.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "170 sfu exactly — high begins.: got {} and the author's code gave 4.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the record's largest day, 343 sfu — band 4, which is unbounded above.», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Ratio::new(343.0)).expect("the record's largest day, 343 sfu — band 4, which is unbounded above.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "the record's largest day, 343 sfu — band 4, which is unbounded above.: got {} and the author's code gave 4.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the design value this subsystem publishes, 228 sfu — band 4, the top one.», from their own  code.
#[test]
fn case_9() {
    let got = model::evaluate(Ratio::new(228.1374378829)).expect("the design value this subsystem publishes, 228 sfu — band 4, the top one.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "the design value this subsystem publishes, 228 sfu — band 4, the top one.: got {} and the author's code gave 4.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «a value that is not a number: the node must refuse it, as every method does at its door. The code before 1.1 answered band 1 here, which no value is», from their own  code.
#[test]
fn case_10() {
    let got = model::evaluate(Ratio::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "a value that is not a number: the node must refuse it, as every method does at its door. The code before 1.1 answered band 1 here, which no value is: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 64 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_activity_band::evaluate;
    assert_eq!(evaluate(32.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (32.0)");
    assert_eq!(evaluate(57.6).map(f64::to_bits), Ok(0x3ff0000000000000), "at (57.6)");
    assert_eq!(evaluate(63.36).map(f64::to_bits), Ok(0x3ff0000000000000), "at (63.36)");
    assert_eq!(evaluate(64.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (64.0)");
    assert_eq!(evaluate(64.64).map(f64::to_bits), Ok(0x3ff0000000000000), "at (64.64)");
    assert_eq!(evaluate(70.4).map(f64::to_bits), Ok(0x3ff0000000000000), "at (70.4)");
    assert_eq!(evaluate(128.0).map(f64::to_bits), Ok(0x4000000000000000), "at (128.0)");
    assert_eq!(evaluate(44.95).map(f64::to_bits), Ok(0x3ff0000000000000), "at (44.95)");
    assert_eq!(evaluate(80.91000000000001).map(f64::to_bits), Ok(0x3ff0000000000000), "at (80.91000000000001)");
    assert_eq!(evaluate(89.001).map(f64::to_bits), Ok(0x3ff0000000000000), "at (89.001)");
    assert_eq!(evaluate(89.9).map(f64::to_bits), Ok(0x3ff0000000000000), "at (89.9)");
    assert_eq!(evaluate(90.799).map(f64::to_bits), Ok(0x4000000000000000), "at (90.799)");
    assert_eq!(evaluate(98.89000000000001).map(f64::to_bits), Ok(0x4000000000000000), "at (98.89000000000001)");
    assert_eq!(evaluate(179.8).map(f64::to_bits), Ok(0x4010000000000000), "at (179.8)");
    assert_eq!(evaluate(45.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (45.0)");
    assert_eq!(evaluate(81.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (81.0)");
    assert_eq!(evaluate(89.1).map(f64::to_bits), Ok(0x3ff0000000000000), "at (89.1)");
    assert_eq!(evaluate(90.0).map(f64::to_bits), Ok(0x4000000000000000), "at (90.0)");
    assert_eq!(evaluate(90.9).map(f64::to_bits), Ok(0x4000000000000000), "at (90.9)");
    assert_eq!(evaluate(99.00000000000001).map(f64::to_bits), Ok(0x4000000000000000), "at (99.00000000000001)");
    assert_eq!(evaluate(180.0).map(f64::to_bits), Ok(0x4010000000000000), "at (180.0)");
    assert_eq!(evaluate(64.95).map(f64::to_bits), Ok(0x3ff0000000000000), "at (64.95)");
    assert_eq!(evaluate(116.91000000000001).map(f64::to_bits), Ok(0x4000000000000000), "at (116.91000000000001)");
    assert_eq!(evaluate(128.601).map(f64::to_bits), Ok(0x4000000000000000), "at (128.601)");
    assert_eq!(evaluate(129.9).map(f64::to_bits), Ok(0x4000000000000000), "at (129.9)");
    assert_eq!(evaluate(131.199).map(f64::to_bits), Ok(0x4008000000000000), "at (131.199)");
    assert_eq!(evaluate(142.89000000000001).map(f64::to_bits), Ok(0x4008000000000000), "at (142.89000000000001)");
    assert_eq!(evaluate(259.8).map(f64::to_bits), Ok(0x4010000000000000), "at (259.8)");
    assert_eq!(evaluate(65.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (65.0)");
    assert_eq!(evaluate(117.0).map(f64::to_bits), Ok(0x4000000000000000), "at (117.0)");
    assert_eq!(evaluate(128.7).map(f64::to_bits), Ok(0x4000000000000000), "at (128.7)");
    assert_eq!(evaluate(130.0).map(f64::to_bits), Ok(0x4008000000000000), "at (130.0)");
    assert_eq!(evaluate(131.3).map(f64::to_bits), Ok(0x4008000000000000), "at (131.3)");
    assert_eq!(evaluate(143.0).map(f64::to_bits), Ok(0x4008000000000000), "at (143.0)");
    assert_eq!(evaluate(260.0).map(f64::to_bits), Ok(0x4010000000000000), "at (260.0)");
    assert_eq!(evaluate(84.95).map(f64::to_bits), Ok(0x3ff0000000000000), "at (84.95)");
    assert_eq!(evaluate(152.91).map(f64::to_bits), Ok(0x4008000000000000), "at (152.91)");
    assert_eq!(evaluate(168.201).map(f64::to_bits), Ok(0x4008000000000000), "at (168.201)");
    assert_eq!(evaluate(169.9).map(f64::to_bits), Ok(0x4008000000000000), "at (169.9)");
    assert_eq!(evaluate(171.59900000000002).map(f64::to_bits), Ok(0x4010000000000000), "at (171.59900000000002)");
    assert_eq!(evaluate(186.89000000000001).map(f64::to_bits), Ok(0x4010000000000000), "at (186.89000000000001)");
    assert_eq!(evaluate(339.8).map(f64::to_bits), Ok(0x4010000000000000), "at (339.8)");
    assert_eq!(evaluate(85.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (85.0)");
    assert_eq!(evaluate(153.0).map(f64::to_bits), Ok(0x4008000000000000), "at (153.0)");
    assert_eq!(evaluate(168.3).map(f64::to_bits), Ok(0x4008000000000000), "at (168.3)");
    assert_eq!(evaluate(170.0).map(f64::to_bits), Ok(0x4010000000000000), "at (170.0)");
    assert_eq!(evaluate(171.7).map(f64::to_bits), Ok(0x4010000000000000), "at (171.7)");
    assert_eq!(evaluate(187.00000000000003).map(f64::to_bits), Ok(0x4010000000000000), "at (187.00000000000003)");
    assert_eq!(evaluate(340.0).map(f64::to_bits), Ok(0x4010000000000000), "at (340.0)");
    assert_eq!(evaluate(171.5).map(f64::to_bits), Ok(0x4010000000000000), "at (171.5)");
    assert_eq!(evaluate(308.7).map(f64::to_bits), Ok(0x4010000000000000), "at (308.7)");
    assert_eq!(evaluate(339.57).map(f64::to_bits), Ok(0x4010000000000000), "at (339.57)");
    assert_eq!(evaluate(343.0).map(f64::to_bits), Ok(0x4010000000000000), "at (343.0)");
    assert_eq!(evaluate(346.43).map(f64::to_bits), Ok(0x4010000000000000), "at (346.43)");
    assert_eq!(evaluate(377.3).map(f64::to_bits), Ok(0x4010000000000000), "at (377.3)");
    assert_eq!(evaluate(686.0).map(f64::to_bits), Ok(0x4010000000000000), "at (686.0)");
    assert_eq!(evaluate(114.06871894145).map(f64::to_bits), Ok(0x4000000000000000), "at (114.06871894145)");
    assert_eq!(evaluate(205.32369409461).map(f64::to_bits), Ok(0x4010000000000000), "at (205.32369409461)");
    assert_eq!(evaluate(225.856063504071).map(f64::to_bits), Ok(0x4010000000000000), "at (225.856063504071)");
    assert_eq!(evaluate(228.1374378829).map(f64::to_bits), Ok(0x4010000000000000), "at (228.1374378829)");
    assert_eq!(evaluate(230.41881226172902).map(f64::to_bits), Ok(0x4010000000000000), "at (230.41881226172902)");
    assert_eq!(evaluate(250.95118167119003).map(f64::to_bits), Ok(0x4010000000000000), "at (250.95118167119003)");
    assert_eq!(evaluate(456.2748757658).map(f64::to_bits), Ok(0x4010000000000000), "at (456.2748757658)");
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
/// Derived from `the record's quietest day, 64 sfu — band 1, low` and the declared domain 1 … 4.
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
        if let Err(f) = model::evaluate(Ratio::new(64.0 * scale)) {
            refused.push(format!("f107 x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_activity_band refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 1 … 4 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(64.0 * scale)) {
            assert!(v.get().is_finite(), "sw_activity_band produced a value that is not a number for band");
            assert!(v.get() >= 1.0 && v.get() <= 4.0, "sw_activity_band answered {} for band, outside its declared domain 1 … 4 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(64.0));
    let b = model::evaluate(Ratio::new(64.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_activity_band is not deterministic for band: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_activity_band refused on one call and answered on the other"),
    }
}

