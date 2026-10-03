// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_central_expectation`.
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

/// the declared case — epoch 2027-01-01, a five-year mission
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 86.84972489094359);
    assert!(err <= 1e-12, "the declared case — epoch 2027-01-01, a five-year mission: got {} want 86.84972489094359, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// fifteen years — the window spans more than a cycle, so the mean settles near the record's own level
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(473364000.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 105.72687096778448);
    assert!(err <= 1e-12, "fifteen years — the window spans more than a cycle, so the mean settles near the record's own level: got {} want 105.72687096778448, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// half a Julian year, the shortest mission — the one lead where today's 150 sfu is still faintly visible
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(15778800.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 105.5465798678905);
    assert!(err <= 1e-12, "half a Julian year, the shortest mission — the one lead where today's 150 sfu is still faintly visible: got {} want 105.5465798678905, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// a quiet day today changes nothing at five years
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(70.0), Time::new(157788000.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 86.84972489094359);
    assert!(err <= 1e-12, "a quiet day today changes nothing at five years: got {} want 86.84972489094359, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// nor does a very active one — at five years the answer is the window climatology alone
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(340.0), Time::new(157788000.0), Time::new(852076800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 86.84972489094359);
    assert!(err <= 1e-12, "nor does a very active one — at five years the answer is the window climatology alone: got {} want 86.84972489094359, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the earliest epoch the range allows, 2018-01-01 — a mission rising into cycle 25's maximum
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(568080000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 115.03865410832137);
    assert!(err <= 1e-12, "the earliest epoch the range allows, 2018-01-01 — a mission rising into cycle 25's maximum: got {} want 115.03865410832137, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// an epoch near the next maximum, 2032-11-08 — the same mission, a different sky, and one cycle 25's size does not vouch for
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(1036800000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 144.9367498101883);
    assert!(err <= 1e-12, "an epoch near the next maximum, 2032-11-08 — the same mission, a different sky, and one cycle 25's size does not vouch for: got {} want 144.9367498101883, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the latest epoch the range allows, 2040-01-01
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(1262304000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 88.67145559562236);
    assert!(err <= 1e-12, "the latest epoch the range allows, 2040-01-01: got {} want 88.67145559562236, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// one whole cycle period later — epoch 14040 is 4178 days after 9862, so the SHAPE repeats; the answer does not, because the amplitude hands over
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(1213056000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 78.21016904177215);
    assert!(err <= 1e-12, "one whole cycle period later — epoch 14040 is 4178 days after 9862, so the SHAPE repeats; the answer does not, because the amplitude hands over: got {} want 78.21016904177215, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «the declared case — epoch 2027-01-01, a five-year mission.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0)).expect("the declared case — epoch 2027-01-01, a five-year mission.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 86.84972489094359);
    assert!(err <= 1e-12, "the declared case — epoch 2027-01-01, a five-year mission.: got {} and the author's code gave 86.84972489094359; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «fifteen years — the window spans more than a cycle, so the mean settles near the record's own level.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(473364000.0), Time::new(852076800.0)).expect("fifteen years — the window spans more than a cycle, so the mean settles near the record's own level.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 105.72687096778448);
    assert!(err <= 1e-12, "fifteen years — the window spans more than a cycle, so the mean settles near the record's own level.: got {} and the author's code gave 105.72687096778448; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «half a Julian year, the shortest mission — the one lead where today's 150 sfu is still faintly visible.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(15778800.0), Time::new(852076800.0)).expect("half a Julian year, the shortest mission — the one lead where today's 150 sfu is still faintly visible.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 105.5465798678905);
    assert!(err <= 1e-12, "half a Julian year, the shortest mission — the one lead where today's 150 sfu is still faintly visible.: got {} and the author's code gave 105.5465798678905; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «a quiet day today changes nothing at five years.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(70.0), Time::new(157788000.0), Time::new(852076800.0)).expect("a quiet day today changes nothing at five years.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 86.84972489094359);
    assert!(err <= 1e-12, "a quiet day today changes nothing at five years.: got {} and the author's code gave 86.84972489094359; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «nor does a very active one — at five years the answer is the window climatology alone.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(340.0), Time::new(157788000.0), Time::new(852076800.0)).expect("nor does a very active one — at five years the answer is the window climatology alone.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 86.84972489094359);
    assert!(err <= 1e-12, "nor does a very active one — at five years the answer is the window climatology alone.: got {} and the author's code gave 86.84972489094359; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the earliest epoch the range allows, 2018-01-01 — a mission rising into cycle 25's maximum.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(568080000.0)).expect("the earliest epoch the range allows, 2018-01-01 — a mission rising into cycle 25's maximum.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 115.03865410832137);
    assert!(err <= 1e-12, "the earliest epoch the range allows, 2018-01-01 — a mission rising into cycle 25's maximum.: got {} and the author's code gave 115.03865410832137; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «an epoch near the next maximum, 2032-11-08 — the same mission, a different sky, and one cycle 25's size does not vouch for.», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(1036800000.0)).expect("an epoch near the next maximum, 2032-11-08 — the same mission, a different sky, and one cycle 25's size does not vouch for.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 144.9367498101883);
    assert!(err <= 1e-12, "an epoch near the next maximum, 2032-11-08 — the same mission, a different sky, and one cycle 25's size does not vouch for.: got {} and the author's code gave 144.9367498101883; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the latest epoch the range allows, 2040-01-01.», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(1262304000.0)).expect("the latest epoch the range allows, 2040-01-01.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 88.67145559562236);
    assert!(err <= 1e-12, "the latest epoch the range allows, 2040-01-01.: got {} and the author's code gave 88.67145559562236; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «one whole cycle period later — epoch 14040 is 4178 days after 9862, so the SHAPE repeats; the answer does not, because the amplitude hands over.», from their own  code.
#[test]
fn case_9() {
    let got = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(1213056000.0)).expect("one whole cycle period later — epoch 14040 is 4178 days after 9862, so the SHAPE repeats; the answer does not, because the amplitude hands over.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 78.21016904177215);
    assert!(err <= 1e-12, "one whole cycle period later — epoch 14040 is 4178 days after 9862, so the SHAPE repeats; the answer does not, because the amplitude hands over.: got {} and the author's code gave 78.21016904177215; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_10() {
    let got = model::evaluate(Ratio::new(1000.0), Time::new(86400.0), Time::new(852076800.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_11() {
    let got = model::evaluate(Ratio::new(0.0), Time::new(86400.0), Time::new(852076800.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 65 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_central_expectation::evaluate;
    assert_eq!(evaluate(75.0, 78894000.0, 426038400.0).map(f64::to_bits), Ok(0x405c7bc507a0b916), "at (75.0, 78894000.0, 426038400.0)");
    assert_eq!(evaluate(135.0, 142009200.0, 766869120.0).map(f64::to_bits), Ok(0x4060715d0f5187f4), "at (135.0, 142009200.0, 766869120.0)");
    assert_eq!(evaluate(148.5, 156210120.0, 843556032.0).map(f64::to_bits), Ok(0x4055fe215e8a3411), "at (148.5, 156210120.0, 843556032.0)");
    assert_eq!(evaluate(150.0, 157788000.0, 852076800.0).map(f64::to_bits), Ok(0x4055b661e4824ccb), "at (150.0, 157788000.0, 852076800.0)");
    assert_eq!(evaluate(151.5, 159365880.0, 860597568.0).map(f64::to_bits), Ok(0x40558e9a6954155c), "at (151.5, 159365880.0, 860597568.0)");
    assert_eq!(evaluate(165.0, 173566800.0, 937284480.0000001).map(f64::to_bits), Ok(0x405b975a4abb9638), "at (165.0, 173566800.0, 937284480.0000001)");
    assert_eq!(evaluate(300.0, 315576000.0, 1704153600.0).map(f64::to_bits), Ok(0x405d0a9513961dcd), "at (300.0, 315576000.0, 1704153600.0)");
    assert_eq!(evaluate(75.0, 236682000.0, 426038400.0).map(f64::to_bits), Ok(0x40578465b5408785), "at (75.0, 236682000.0, 426038400.0)");
    assert_eq!(evaluate(135.0, 426027600.0, 766869120.0).map(f64::to_bits), Ok(0x405eb5541aa91278), "at (135.0, 426027600.0, 766869120.0)");
    assert_eq!(evaluate(148.5, 468630360.0, 843556032.0).map(f64::to_bits), Ok(0x405aacb658cf54be), "at (148.5, 468630360.0, 843556032.0)");
    assert_eq!(evaluate(150.0, 473364000.0, 852076800.0).map(f64::to_bits), Ok(0x405a6e850dcec2f5), "at (150.0, 473364000.0, 852076800.0)");
    assert_eq!(evaluate(151.5, 478097640.0, 860597568.0).map(f64::to_bits), Ok(0x405a339dfadf04b9), "at (151.5, 478097640.0, 860597568.0)");
    assert_eq!(evaluate(165.0, 520700400.00000006, 937284480.0000001).map(f64::to_bits), Ok(0x405b53bcd806e4d9), "at (165.0, 520700400.00000006, 937284480.0000001)");
    assert_eq!(evaluate(300.0, 946728000.0, 1704153600.0).map(f64::to_bits), Ok(0x405cc1cacac551da), "at (300.0, 946728000.0, 1704153600.0)");
    assert_eq!(evaluate(75.0, 7889400.0, 426038400.0).map(f64::to_bits), Ok(0x406259d6eb63b1cf), "at (75.0, 7889400.0, 426038400.0)");
    assert_eq!(evaluate(135.0, 14200920.0, 766869120.0).map(f64::to_bits), Ok(0x406a1a3bbed86060), "at (135.0, 14200920.0, 766869120.0)");
    assert_eq!(evaluate(148.5, 15621012.0, 843556032.0).map(f64::to_bits), Ok(0x405a86dd691cde28), "at (148.5, 15621012.0, 843556032.0)");
    assert_eq!(evaluate(150.0, 15778800.0, 852076800.0).map(f64::to_bits), Ok(0x405a62fb2a204f78), "at (150.0, 15778800.0, 852076800.0)");
    assert_eq!(evaluate(151.5, 15936588.0, 860597568.0).map(f64::to_bits), Ok(0x4059c44051801feb), "at (151.5, 15936588.0, 860597568.0)");
    assert_eq!(evaluate(165.0, 17356680.0, 937284480.0000001).map(f64::to_bits), Ok(0x40537b104a9ff5ed), "at (165.0, 17356680.0, 937284480.0000001)");
    assert_eq!(evaluate(300.0, 31557600.0, 1704153600.0).map(f64::to_bits), Ok(0x4054638a217eef92), "at (300.0, 31557600.0, 1704153600.0)");
    assert_eq!(evaluate(35.0, 78894000.0, 426038400.0).map(f64::to_bits), Ok(0x405c7bc507a0b910), "at (35.0, 78894000.0, 426038400.0)");
    assert_eq!(evaluate(63.0, 142009200.0, 766869120.0).map(f64::to_bits), Ok(0x4060715d0f5187f4), "at (63.0, 142009200.0, 766869120.0)");
    assert_eq!(evaluate(69.3, 156210120.0, 843556032.0).map(f64::to_bits), Ok(0x4055fe215e8a3411), "at (69.3, 156210120.0, 843556032.0)");
    assert_eq!(evaluate(70.0, 157788000.0, 852076800.0).map(f64::to_bits), Ok(0x4055b661e4824ccb), "at (70.0, 157788000.0, 852076800.0)");
    assert_eq!(evaluate(70.7, 159365880.0, 860597568.0).map(f64::to_bits), Ok(0x40558e9a6954155c), "at (70.7, 159365880.0, 860597568.0)");
    assert_eq!(evaluate(77.0, 173566800.0, 937284480.0000001).map(f64::to_bits), Ok(0x405b975a4abb9638), "at (77.0, 173566800.0, 937284480.0000001)");
    assert_eq!(evaluate(140.0, 315576000.0, 1704153600.0).map(f64::to_bits), Ok(0x405d0a9513961dcd), "at (140.0, 315576000.0, 1704153600.0)");
    assert_eq!(evaluate(170.0, 78894000.0, 426038400.0).map(f64::to_bits), Ok(0x405c7bc507a0b924), "at (170.0, 78894000.0, 426038400.0)");
    assert_eq!(evaluate(306.0, 142009200.0, 766869120.0).map(f64::to_bits), Ok(0x4060715d0f5187f4), "at (306.0, 142009200.0, 766869120.0)");
    assert_eq!(evaluate(336.6, 156210120.0, 843556032.0).map(f64::to_bits), Ok(0x4055fe215e8a3411), "at (336.6, 156210120.0, 843556032.0)");
    assert_eq!(evaluate(340.0, 157788000.0, 852076800.0).map(f64::to_bits), Ok(0x4055b661e4824ccb), "at (340.0, 157788000.0, 852076800.0)");
    assert_eq!(evaluate(343.4, 159365880.0, 860597568.0).map(f64::to_bits), Ok(0x40558e9a6954155c), "at (343.4, 159365880.0, 860597568.0)");
    assert_eq!(evaluate(374.00000000000006, 173566800.0, 937284480.0000001).map(f64::to_bits), Ok(0x405b975a4abb9638), "at (374.00000000000006, 173566800.0, 937284480.0000001)");
    assert_eq!(evaluate(680.0, 315576000.0, 1704153600.0).map(f64::to_bits), Ok(0x405d0a9513961dcd), "at (680.0, 315576000.0, 1704153600.0)");
    assert_eq!(evaluate(75.0, 78894000.0, 284040000.0).map(f64::to_bits), Ok(0x405f96f82eb45f65), "at (75.0, 78894000.0, 284040000.0)");
    assert_eq!(evaluate(135.0, 142009200.0, 511272000.0).map(f64::to_bits), Ok(0x40546e4aa9d63a57), "at (135.0, 142009200.0, 511272000.0)");
    assert_eq!(evaluate(148.5, 156210120.0, 562399200.0).map(f64::to_bits), Ok(0x405bb613e652e263), "at (148.5, 156210120.0, 562399200.0)");
    assert_eq!(evaluate(150.0, 157788000.0, 568080000.0).map(f64::to_bits), Ok(0x405cc2794f14c62a), "at (150.0, 157788000.0, 568080000.0)");
    assert_eq!(evaluate(151.5, 159365880.0, 573760800.0).map(f64::to_bits), Ok(0x405dcc3fe2950783), "at (151.5, 159365880.0, 573760800.0)");
    assert_eq!(evaluate(165.0, 173566800.0, 624888000.0).map(f64::to_bits), Ok(0x406396f30939f29a), "at (165.0, 173566800.0, 624888000.0)");
    assert_eq!(evaluate(300.0, 315576000.0, 1136160000.0).map(f64::to_bits), Ok(0x4059e6d05352258c), "at (300.0, 315576000.0, 1136160000.0)");
    assert_eq!(evaluate(75.0, 78894000.0, 518400000.0).map(f64::to_bits), Ok(0x4052de975e807bea), "at (75.0, 78894000.0, 518400000.0)");
    assert_eq!(evaluate(135.0, 142009200.0, 933120000.0).map(f64::to_bits), Ok(0x4058f1e7cae43457), "at (135.0, 142009200.0, 933120000.0)");
    assert_eq!(evaluate(148.5, 156210120.0, 1026432000.0).map(f64::to_bits), Ok(0x40623919d8604851), "at (148.5, 156210120.0, 1026432000.0)");
    assert_eq!(evaluate(150.0, 157788000.0, 1036800000.0).map(f64::to_bits), Ok(0x40621df9dabce960), "at (150.0, 157788000.0, 1036800000.0)");
    assert_eq!(evaluate(151.5, 159365880.0, 1047168000.0).map(f64::to_bits), Ok(0x4061c3d4670012d9), "at (151.5, 159365880.0, 1047168000.0)");
    assert_eq!(evaluate(165.0, 173566800.0, 1140480000.0).map(f64::to_bits), Ok(0x40588889b703d8f8), "at (165.0, 173566800.0, 1140480000.0)");
    assert_eq!(evaluate(300.0, 315576000.0, 2073600000.0).map(f64::to_bits), Ok(0x405cf8f5e26205be), "at (300.0, 315576000.0, 2073600000.0)");
    assert_eq!(evaluate(75.0, 78894000.0, 631152000.0).map(f64::to_bits), Ok(0x40609c10db81750f), "at (75.0, 78894000.0, 631152000.0)");
    assert_eq!(evaluate(135.0, 142009200.0, 1136073600.0).map(f64::to_bits), Ok(0x405af021f63452e0), "at (135.0, 142009200.0, 1136073600.0)");
    assert_eq!(evaluate(148.5, 156210120.0, 1249680960.0).map(f64::to_bits), Ok(0x4054f25d02872c56), "at (148.5, 156210120.0, 1249680960.0)");
    assert_eq!(evaluate(150.0, 157788000.0, 1262304000.0).map(f64::to_bits), Ok(0x40562af920e3fa83), "at (150.0, 157788000.0, 1262304000.0)");
    assert_eq!(evaluate(151.5, 159365880.0, 1274927040.0).map(f64::to_bits), Ok(0x40579e218bbb127b), "at (151.5, 159365880.0, 1274927040.0)");
    assert_eq!(evaluate(165.0, 173566800.0, 1388534400.0).map(f64::to_bits), Ok(0x4061bc588196d908), "at (165.0, 173566800.0, 1388534400.0)");
    assert_eq!(evaluate(300.0, 315576000.0, 2524608000.0).map(f64::to_bits), Ok(0x405a8ab7fdc40374), "at (300.0, 315576000.0, 2524608000.0)");
    assert_eq!(evaluate(75.0, 78894000.0, 606528000.0).map(f64::to_bits), Ok(0x405b6e13c9af73c9), "at (75.0, 78894000.0, 606528000.0)");
    assert_eq!(evaluate(135.0, 142009200.0, 1091750400.0).map(f64::to_bits), Ok(0x40607d087ffcd517), "at (135.0, 142009200.0, 1091750400.0)");
    assert_eq!(evaluate(148.5, 156210120.0, 1200925440.0).map(f64::to_bits), Ok(0x4053bbb06b8f810e), "at (148.5, 156210120.0, 1200925440.0)");
    assert_eq!(evaluate(150.0, 157788000.0, 1213056000.0).map(f64::to_bits), Ok(0x40538d7368da42c1), "at (150.0, 157788000.0, 1213056000.0)");
    assert_eq!(evaluate(151.5, 159365880.0, 1225186560.0).map(f64::to_bits), Ok(0x4053b76d6243d06e), "at (151.5, 159365880.0, 1225186560.0)");
    assert_eq!(evaluate(165.0, 173566800.0, 1334361600.0).map(f64::to_bits), Ok(0x40603182bdb9ea03), "at (165.0, 173566800.0, 1334361600.0)");
    assert_eq!(evaluate(300.0, 315576000.0, 2426112000.0).map(f64::to_bits), Ok(0x405d0a9513961dcd), "at (300.0, 315576000.0, 2426112000.0)");
    assert!(matches!(evaluate(1000.0, 86400.0, 852076800.0), Err(MethodError::Refused(_))), "at (1000.0, 86400.0, 852076800.0)");
    assert!(matches!(evaluate(0.0, 86400.0, 852076800.0), Err(MethodError::Refused(_))), "at (0.0, 86400.0, 852076800.0)");
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `the declared case — epoch 2027-01-01, a five-year mission` and the declared domain 60 … 400.
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
        if let Err(f) = model::evaluate(Ratio::new(150.0 * scale), Time::new(157788000.0), Time::new(852076800.0)) {
            refused.push(format!("today x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(150.0), Time::new(157788000.0 * scale), Time::new(852076800.0)) {
            refused.push(format!("lead x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0 * scale)) {
            refused.push(format!("epoch x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_central_expectation refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 60 … 400 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(150.0 * scale), Time::new(157788000.0), Time::new(852076800.0)) {
            assert!(v.get().is_finite(), "sw_central_expectation produced a value that is not a number for F107_central");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_central_expectation answered {} for F107_central, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(150.0), Time::new(157788000.0 * scale), Time::new(852076800.0)) {
            assert!(v.get().is_finite(), "sw_central_expectation produced a value that is not a number for F107_central");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_central_expectation answered {} for F107_central, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0 * scale)) {
            assert!(v.get().is_finite(), "sw_central_expectation produced a value that is not a number for F107_central");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_central_expectation answered {} for F107_central, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0));
    let b = model::evaluate(Ratio::new(150.0), Time::new(157788000.0), Time::new(852076800.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_central_expectation is not deterministic for F107_central: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_central_expectation refused on one call and answered on the other"),
    }
}

