// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_storm_return_level`.
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

/// half a year — rank 56 of the record, the shortest mission the tree allows
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(15778800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 66.0);
    assert!(err <= 0.0281, "half a year — rank 56 of the record, the shortest mission the tree allows: got {} want 66.0, relative error {} exceeds the declared tolerance 0.0281. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// one year — rank 28, one storm season
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(31557600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 96.0);
    assert!(err <= 0.0363, "one year — rank 28, one storm season: got {} want 96.0, relative error {} exceeds the declared tolerance 0.0363. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// two years — rank 14
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(63115200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 118.0);
    assert!(err <= 0.0245, "two years — rank 14: got {} want 118.0, relative error {} exceeds the declared tolerance 0.0245. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// three years — rank 9, and the fit's worst point: ties at ranks 9 and 10 that the curve smooths through
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(94672800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 127.0);
    assert!(err <= 0.0826, "three years — rank 9, and the fit's worst point: ties at ranks 9 and 10 that the curve smooths through: got {} want 127.0, relative error {} exceeds the declared tolerance 0.0826. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// five years — rank 6, the mission orbit_mission_duration currently declares
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(157788000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 162.0);
    assert!(err <= 0.0224, "five years — rank 6, the mission orbit_mission_duration currently declares: got {} want 162.0, relative error {} exceeds the declared tolerance 0.0224. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// ten years — rank 3, near the top of what 28.2 years of record can say anything about
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Time::new(315576000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 189.0);
    assert!(err <= 0.0119, "ten years — rank 3, near the top of what 28.2 years of record can say anything about: got {} want 189.0, relative error {} exceeds the declared tolerance 0.0119. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «half a year — rank 56 of the record, the shortest mission the tree allows.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Time::new(15778800.0)).expect("half a year — rank 56 of the record, the shortest mission the tree allows.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 66.0);
    assert!(err <= 0.0281, "half a year — rank 56 of the record, the shortest mission the tree allows.: got {} and the author's code gave 66.0; relative error {} is more than their tolerance 0.0281. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «one year — rank 28, one storm season.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Time::new(31557600.0)).expect("one year — rank 28, one storm season.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 96.0);
    assert!(err <= 0.0363, "one year — rank 28, one storm season.: got {} and the author's code gave 96.0; relative error {} is more than their tolerance 0.0363. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «two years — rank 14.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Time::new(63115200.0)).expect("two years — rank 14.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 118.0);
    assert!(err <= 0.0245, "two years — rank 14.: got {} and the author's code gave 118.0; relative error {} is more than their tolerance 0.0245. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «three years — rank 9, and the fit's worst point: ties at ranks 9 and 10 that the curve smooths through.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Time::new(94672800.0)).expect("three years — rank 9, and the fit's worst point: ties at ranks 9 and 10 that the curve smooths through.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 127.0);
    assert!(err <= 0.0826, "three years — rank 9, and the fit's worst point: ties at ranks 9 and 10 that the curve smooths through.: got {} and the author's code gave 127.0; relative error {} is more than their tolerance 0.0826. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «five years — rank 6, the mission orbit_mission_duration currently declares.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Time::new(157788000.0)).expect("five years — rank 6, the mission orbit_mission_duration currently declares.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 162.0);
    assert!(err <= 0.0224, "five years — rank 6, the mission orbit_mission_duration currently declares.: got {} and the author's code gave 162.0; relative error {} is more than their tolerance 0.0224. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «ten years — rank 3, near the top of what 28.2 years of record can say anything about.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Time::new(315576000.0)).expect("ten years — rank 3, near the top of what 28.2 years of record can say anything about.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 189.0);
    assert!(err <= 0.0119, "ten years — rank 3, near the top of what 28.2 years of record can say anything about.: got {} and the author's code gave 189.0; relative error {} is more than their tolerance 0.0119. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Time::new(0.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Time::new(1576800000.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 44 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_storm_return_level::evaluate;
    assert_eq!(evaluate(7889400.0).map(f64::to_bits), Ok(0x4041e3c12c17e6de), "at (7889400.0)");
    assert_eq!(evaluate(14200920.0).map(f64::to_bits), Ok(0x404deaee285b99c7), "at (14200920.0)");
    assert_eq!(evaluate(15621012.0).map(f64::to_bits), Ok(0x404fde38bda86498), "at (15621012.0)");
    assert_eq!(evaluate(15778800.0).map(f64::to_bits), Ok(0x4050096f85e41c17), "at (15778800.0)");
    assert_eq!(evaluate(15936588.0).map(f64::to_bits), Ok(0x4050237f9e45177f), "at (15936588.0)");
    assert_eq!(evaluate(17356680.0).map(f64::to_bits), Ok(0x40510314d08a8180), "at (17356680.0)");
    assert_eq!(evaluate(31557600.0).map(f64::to_bits), Ok(0x405720fe75bc44bf), "at (31557600.0)");
    assert_eq!(evaluate(15778800.0).map(f64::to_bits), Ok(0x4050096f85e41c17), "at (15778800.0)");
    assert_eq!(evaluate(28401840.0).map(f64::to_bits), Ok(0x40560d060405f58c), "at (28401840.0)");
    assert_eq!(evaluate(31242024.0).map(f64::to_bits), Ok(0x405706ab4eac5af4), "at (31242024.0)");
    assert_eq!(evaluate(31557600.0).map(f64::to_bits), Ok(0x405720fe75bc44bf), "at (31557600.0)");
    assert_eq!(evaluate(31873176.0).map(f64::to_bits), Ok(0x40573b0e8e1d4027), "at (31873176.0)");
    assert_eq!(evaluate(34713360.0).map(f64::to_bits), Ok(0x40581aa3c062aa28), "at (34713360.0)");
    assert_eq!(evaluate(63115200.0).map(f64::to_bits), Ok(0x405e388d65946d67), "at (63115200.0)");
    assert_eq!(evaluate(31557600.0).map(f64::to_bits), Ok(0x405720fe75bc44bf), "at (31557600.0)");
    assert_eq!(evaluate(56803680.0).map(f64::to_bits), Ok(0x405d2494f3de1e34), "at (56803680.0)");
    assert_eq!(evaluate(62484048.0).map(f64::to_bits), Ok(0x405e1e3a3e84839c), "at (62484048.0)");
    assert_eq!(evaluate(63115200.0).map(f64::to_bits), Ok(0x405e388d65946d67), "at (63115200.0)");
    assert_eq!(evaluate(63746352.0).map(f64::to_bits), Ok(0x405e529d7df568cf), "at (63746352.0)");
    assert_eq!(evaluate(69426720.0).map(f64::to_bits), Ok(0x405f3232b03ad2d0), "at (69426720.0)");
    assert_eq!(evaluate(126230400.0).map(f64::to_bits), Ok(0x4062a80e2ab64b08), "at (126230400.0)");
    assert_eq!(evaluate(47336400.0).map(f64::to_bits), Ok(0x405b47070c88b3c1), "at (47336400.0)");
    assert_eq!(evaluate(85205520.0).map(f64::to_bits), Ok(0x4060a54ec555469b), "at (85205520.0)");
    assert_eq!(evaluate(93726072.0).map(f64::to_bits), Ok(0x406122216aa87950), "at (93726072.0)");
    assert_eq!(evaluate(94672800.0).map(f64::to_bits), Ok(0x40612f4afe306e34), "at (94672800.0)");
    assert_eq!(evaluate(95619528.0).map(f64::to_bits), Ok(0x40613c530a60ebe8), "at (95619528.0)");
    assert_eq!(evaluate(104140080.00000001).map(f64::to_bits), Ok(0x4061ac1da383a0e9), "at (104140080.00000001)");
    assert_eq!(evaluate(189345600.0).map(f64::to_bits), Ok(0x4064bb12761c8289), "at (189345600.0)");
    assert_eq!(evaluate(78894000.0).map(f64::to_bits), Ok(0x406040840a85b8fc), "at (78894000.0)");
    assert_eq!(evaluate(142009200.0).map(f64::to_bits), Ok(0x4063424f4996a5b6), "at (142009200.0)");
    assert_eq!(evaluate(156210120.0).map(f64::to_bits), Ok(0x4063bf21eee9d86a), "at (156210120.0)");
    assert_eq!(evaluate(157788000.0).map(f64::to_bits), Ok(0x4063cc4b8271cd50), "at (157788000.0)");
    assert_eq!(evaluate(159365880.0).map(f64::to_bits), Ok(0x4063d9538ea24b04), "at (159365880.0)");
    assert_eq!(evaluate(173566800.0).map(f64::to_bits), Ok(0x4064491e27c50004), "at (173566800.0)");
    assert_eq!(evaluate(315576000.0).map(f64::to_bits), Ok(0x40675812fa5de1a4), "at (315576000.0)");
    assert_eq!(evaluate(157788000.0).map(f64::to_bits), Ok(0x4063cc4b8271cd50), "at (157788000.0)");
    assert_eq!(evaluate(284018400.0).map(f64::to_bits), Ok(0x4066ce16c182ba0a), "at (284018400.0)");
    assert_eq!(evaluate(312420240.0).map(f64::to_bits), Ok(0x40674ae966d5ecbe), "at (312420240.0)");
    assert_eq!(evaluate(315576000.0).map(f64::to_bits), Ok(0x40675812fa5de1a4), "at (315576000.0)");
    assert_eq!(evaluate(318731760.0).map(f64::to_bits), Ok(0x4067651b068e5f57), "at (318731760.0)");
    assert_eq!(evaluate(347133600.0).map(f64::to_bits), Ok(0x4067d4e59fb11458), "at (347133600.0)");
    assert_eq!(evaluate(631152000.0).map(f64::to_bits), Ok(0x406ae3da7249f5f8), "at (631152000.0)");
    assert!(matches!(evaluate(0.0), Err(MethodError::Refused(_))), "at (0.0)");
    assert!(matches!(evaluate(1576800000.0), Err(MethodError::Refused(_))), "at (1576800000.0)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `half a year — rank 56 of the record, the shortest mission the tree allows` and the declared domain 20 … 230.
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
        if let Err(f) = model::evaluate(Time::new(15778800.0 * scale)) {
            refused.push(format!("life x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_storm_return_level refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 20 … 230 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Time::new(15778800.0 * scale)) {
            assert!(v.get().is_finite(), "sw_storm_return_level produced a value that is not a number for Ap_T");
            assert!(v.get() >= 20.0 && v.get() <= 230.0, "sw_storm_return_level answered {} for Ap_T, outside its declared domain 20 … 230 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Time::new(15778800.0));
    let b = model::evaluate(Time::new(15778800.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_storm_return_level is not deterministic for Ap_T: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_storm_return_level refused on one call and answered on the other"),
    }
}

