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

/// the author's case «the declared case — the highest sustained level a 2027 five-year mission meets.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Time::new(157788000.0), Time::new(852076800.0)).expect("the declared case — the highest sustained level a 2027 five-year mission meets.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 110.65627313580244);
    assert!(err <= 1e-12, "the declared case — the highest sustained level a 2027 five-year mission meets.: got {} and the author's code gave 110.65627313580244; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «MATLAB's own window, 2027-06-26 for a year.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Time::new(31536000.0), Time::new(867283200.0)).expect("MATLAB's own window, 2027-06-26 for a year.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 105.56045788424966);
    assert!(err <= 1e-12, "MATLAB's own window, 2027-06-26 for a year.: got {} and the author's code gave 105.56045788424966; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «half a Julian year from the declared epoch — short enough that the peak is at its own start.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Time::new(15778800.0), Time::new(852076800.0)).expect("half a Julian year from the declared epoch — short enough that the peak is at its own start.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 110.65627313580244);
    assert!(err <= 1e-12, "half a Julian year from the declared epoch — short enough that the peak is at its own start.: got {} and the author's code gave 110.65627313580244; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «fifteen years — long enough to contain a whole cycle, so the peak is the next maximum at the mean amplitude.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Time::new(473364000.0), Time::new(852076800.0)).expect("fifteen years — long enough to contain a whole cycle, so the peak is the next maximum at the mean amplitude.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 193.85802469135803);
    assert!(err <= 1e-12, "fifteen years — long enough to contain a whole cycle, so the peak is the next maximum at the mean amplitude.: got {} and the author's code gave 193.85802469135803; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the earliest epoch, 2018-01-01 — the window contains cycle 25's own maximum.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Time::new(157788000.0), Time::new(568080000.0)).expect("the earliest epoch, 2018-01-01 — the window contains cycle 25's own maximum.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 174.09864172839502);
    assert!(err <= 1e-12, "the earliest epoch, 2018-01-01 — the window contains cycle 25's own maximum.: got {} and the author's code gave 174.09864172839502; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «the latest epoch, 2040-01-01.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Time::new(157788000.0), Time::new(1262304000.0)).expect("the latest epoch, 2040-01-01.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 147.97900299382718);
    assert!(err <= 1e-12, "the latest epoch, 2040-01-01.: got {} and the author's code gave 147.97900299382718; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «an epoch near the next maximum, 2032-11-08.», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Time::new(157788000.0), Time::new(1036800000.0)).expect("an epoch near the next maximum, 2032-11-08.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 193.85802469135803);
    assert!(err <= 1e-12, "an epoch near the next maximum, 2032-11-08.: got {} and the author's code gave 193.85802469135803; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «a 2039 window straddling the amplitude handover — the maximum is a limit, not a value at any knot.», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Time::new(63115200.0), Time::new(1257984000.0)).expect("a 2039 window straddling the amplitude handover — the maximum is a limit, not a value at any knot.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 82.38214311111112);
    assert!(err <= 1e-12, "a 2039 window straddling the amplitude handover — the maximum is a limit, not a value at any knot.: got {} and the author's code gave 82.38214311111112; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_9() {
    let got = model::evaluate(Time::new(f64::NEG_INFINITY), Time::new(852076800.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 57 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_window_peak_level::evaluate;
    assert_eq!(evaluate(78894000.0, 426038400.0).map(f64::to_bits), Ok(0x40637026d304f23d), "at (78894000.0, 426038400.0)");
    assert_eq!(evaluate(142009200.0, 766869120.0).map(f64::to_bits), Ok(0x406c24587e6b74f0), "at (142009200.0, 766869120.0)");
    assert_eq!(evaluate(156210120.0, 843556032.0).map(f64::to_bits), Ok(0x405bc649cf3337e9), "at (156210120.0, 843556032.0)");
    assert_eq!(evaluate(157788000.0, 852076800.0).map(f64::to_bits), Ok(0x405baa006109e0f3), "at (157788000.0, 852076800.0)");
    assert_eq!(evaluate(159365880.0, 860597568.0).map(f64::to_bits), Ok(0x405b03b279960f2f), "at (159365880.0, 860597568.0)");
    assert_eq!(evaluate(173566800.0, 937284480.0000001).map(f64::to_bits), Ok(0x4064b102ab66329a), "at (173566800.0, 937284480.0000001)");
    assert_eq!(evaluate(315576000.0, 1704153600.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (315576000.0, 1704153600.0)");
    assert_eq!(evaluate(15768000.0, 433641600.0).map(f64::to_bits), Ok(0x40623071e78a0f39), "at (15768000.0, 433641600.0)");
    assert_eq!(evaluate(28382400.0, 780554880.0).map(f64::to_bits), Ok(0x406ad969ec30684f), "at (28382400.0, 780554880.0)");
    assert_eq!(evaluate(31220640.0, 858610368.0).map(f64::to_bits), Ok(0x405baa006109e0f3), "at (31220640.0, 858610368.0)");
    assert_eq!(evaluate(31536000.0, 867283200.0).map(f64::to_bits), Ok(0x405a63de8abee8cf), "at (31536000.0, 867283200.0)");
    assert_eq!(evaluate(31851360.0, 875956032.0).map(f64::to_bits), Ok(0x40589abb093e0daf), "at (31851360.0, 875956032.0)");
    assert_eq!(evaluate(34689600.0, 954011520.0000001).map(f64::to_bits), Ok(0x4054e08744aef0b1), "at (34689600.0, 954011520.0000001)");
    assert_eq!(evaluate(63072000.0, 1734566400.0).map(f64::to_bits), Ok(0x406297b9ee3afe0d), "at (63072000.0, 1734566400.0)");
    assert_eq!(evaluate(7889400.0, 426038400.0).map(f64::to_bits), Ok(0x40637026d304f23d), "at (7889400.0, 426038400.0)");
    assert_eq!(evaluate(14200920.0, 766869120.0).map(f64::to_bits), Ok(0x406c24587e6b74f0), "at (14200920.0, 766869120.0)");
    assert_eq!(evaluate(15621012.0, 843556032.0).map(f64::to_bits), Ok(0x405bc649cf3337e9), "at (15621012.0, 843556032.0)");
    assert_eq!(evaluate(15778800.0, 852076800.0).map(f64::to_bits), Ok(0x405baa006109e0f3), "at (15778800.0, 852076800.0)");
    assert_eq!(evaluate(15936588.0, 860597568.0).map(f64::to_bits), Ok(0x405b03b279960f2f), "at (15936588.0, 860597568.0)");
    assert_eq!(evaluate(17356680.0, 937284480.0000001).map(f64::to_bits), Ok(0x40558daabff87d11), "at (17356680.0, 937284480.0000001)");
    assert_eq!(evaluate(31557600.0, 1704153600.0).map(f64::to_bits), Ok(0x4056e80c8c0096dd), "at (31557600.0, 1704153600.0)");
    assert_eq!(evaluate(236682000.0, 426038400.0).map(f64::to_bits), Ok(0x40637026d304f23d), "at (236682000.0, 426038400.0)");
    assert_eq!(evaluate(426027600.0, 766869120.0).map(f64::to_bits), Ok(0x406c24587e6b74f0), "at (426027600.0, 766869120.0)");
    assert_eq!(evaluate(468630360.0, 843556032.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (468630360.0, 843556032.0)");
    assert_eq!(evaluate(473364000.0, 852076800.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (473364000.0, 852076800.0)");
    assert_eq!(evaluate(478097640.0, 860597568.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (478097640.0, 860597568.0)");
    assert_eq!(evaluate(520700400.00000006, 937284480.0000001).map(f64::to_bits), Ok(0x40683b74f0329162), "at (520700400.00000006, 937284480.0000001)");
    assert_eq!(evaluate(946728000.0, 1704153600.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (946728000.0, 1704153600.0)");
    assert_eq!(evaluate(78894000.0, 284040000.0).map(f64::to_bits), Ok(0x4062bd2a05efc9b6), "at (78894000.0, 284040000.0)");
    assert_eq!(evaluate(142009200.0, 511272000.0).map(f64::to_bits), Ok(0x405aa9a7a3918fc8), "at (142009200.0, 511272000.0)");
    assert_eq!(evaluate(156210120.0, 562399200.0).map(f64::to_bits), Ok(0x406597ada5aae2d7), "at (156210120.0, 562399200.0)");
    assert_eq!(evaluate(157788000.0, 568080000.0).map(f64::to_bits), Ok(0x4065c32812b2af48), "at (157788000.0, 568080000.0)");
    assert_eq!(evaluate(159365880.0, 573760800.0).map(f64::to_bits), Ok(0x406639ab0378d476), "at (159365880.0, 573760800.0)");
    assert_eq!(evaluate(173566800.0, 624888000.0).map(f64::to_bits), Ok(0x406c24587e6b74f0), "at (173566800.0, 624888000.0)");
    assert_eq!(evaluate(315576000.0, 1136160000.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (315576000.0, 1136160000.0)");
    assert_eq!(evaluate(78894000.0, 631152000.0).map(f64::to_bits), Ok(0x40657b57f7525d7e), "at (78894000.0, 631152000.0)");
    assert_eq!(evaluate(142009200.0, 1136073600.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (142009200.0, 1136073600.0)");
    assert_eq!(evaluate(156210120.0, 1249680960.0).map(f64::to_bits), Ok(0x40627f53fe162590), "at (156210120.0, 1249680960.0)");
    assert_eq!(evaluate(157788000.0, 1262304000.0).map(f64::to_bits), Ok(0x40627f53fe162590), "at (157788000.0, 1262304000.0)");
    assert_eq!(evaluate(159365880.0, 1274927040.0).map(f64::to_bits), Ok(0x40627f53fe162590), "at (159365880.0, 1274927040.0)");
    assert_eq!(evaluate(173566800.0, 1388534400.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (173566800.0, 1388534400.0)");
    assert_eq!(evaluate(315576000.0, 2524608000.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (315576000.0, 2524608000.0)");
    assert_eq!(evaluate(78894000.0, 518400000.0).map(f64::to_bits), Ok(0x40552fa84f27260d), "at (78894000.0, 518400000.0)");
    assert_eq!(evaluate(142009200.0, 933120000.0).map(f64::to_bits), Ok(0x406297b9ee3afe13), "at (142009200.0, 933120000.0)");
    assert_eq!(evaluate(156210120.0, 1026432000.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (156210120.0, 1026432000.0)");
    assert_eq!(evaluate(157788000.0, 1036800000.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (157788000.0, 1036800000.0)");
    assert_eq!(evaluate(159365880.0, 1047168000.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (159365880.0, 1047168000.0)");
    assert_eq!(evaluate(173566800.0, 1140480000.0).map(f64::to_bits), Ok(0x4067c27c21583c76), "at (173566800.0, 1140480000.0)");
    assert_eq!(evaluate(315576000.0, 2073600000.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (315576000.0, 2073600000.0)");
    assert_eq!(evaluate(31557600.0, 628992000.0).map(f64::to_bits), Ok(0x405f183712bf09a8), "at (31557600.0, 628992000.0)");
    assert_eq!(evaluate(56803680.0, 1132185600.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (56803680.0, 1132185600.0)");
    assert_eq!(evaluate(62484048.0, 1245404160.0).map(f64::to_bits), Ok(0x40552fa84f27260d), "at (62484048.0, 1245404160.0)");
    assert_eq!(evaluate(63115200.0, 1257984000.0).map(f64::to_bits), Ok(0x405498750861274b), "at (63115200.0, 1257984000.0)");
    assert_eq!(evaluate(63746352.0, 1270563840.0).map(f64::to_bits), Ok(0x405498750861274b), "at (63746352.0, 1270563840.0)");
    assert_eq!(evaluate(69426720.0, 1383782400.0).map(f64::to_bits), Ok(0x40632336053a8d74), "at (69426720.0, 1383782400.0)");
    assert_eq!(evaluate(126230400.0, 2515968000.0).map(f64::to_bits), Ok(0x40683b74f0329162), "at (126230400.0, 2515968000.0)");
    assert!(matches!(evaluate(f64::NEG_INFINITY, 852076800.0), Err(MethodError::Refused(_))), "at (f64::NEG_INFINITY, 852076800.0)");
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
            assert!(v.get().is_finite(), "sw_window_peak_level produced a value that is not a number for F107_window_peak");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_window_peak_level answered {} for F107_window_peak, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Time::new(157788000.0), Time::new(852076800.0 * scale)) {
            assert!(v.get().is_finite(), "sw_window_peak_level produced a value that is not a number for F107_window_peak");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_window_peak_level answered {} for F107_window_peak, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
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
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_window_peak_level is not deterministic for F107_window_peak: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_window_peak_level refused on one call and answered on the other"),
    }
}

