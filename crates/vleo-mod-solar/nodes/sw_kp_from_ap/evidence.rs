// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_kp_from_ap`.
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

/// Kp 0 — ap 0, the table's first point
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(0.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.0);
    assert!(err <= 1e-12, "Kp 0 — ap 0, the table's first point: got {} want 0.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 1 — ap 4
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(4.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "Kp 1 — ap 4: got {} want 1.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 2 — ap 7
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(7.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "Kp 2 — ap 7: got {} want 2.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 3 — ap 15, the legacy study's own climatology constant, and what env_kp declares by hand
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(15.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "Kp 3 — ap 15, the legacy study's own climatology constant, and what env_kp declares by hand: got {} want 3.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 4 — ap 27
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(27.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "Kp 4 — ap 27: got {} want 4.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 5 — ap 48, the storm threshold
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(48.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 5.0);
    assert!(err <= 1e-12, "Kp 5 — ap 48, the storm threshold: got {} want 5.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 6 — ap 80
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(80.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 6.0);
    assert!(err <= 1e-12, "Kp 6 — ap 80: got {} want 6.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 7 — ap 132
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(132.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 7.0);
    assert!(err <= 1e-12, "Kp 7 — ap 132: got {} want 7.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 8 — ap 207
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(207.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 8.0);
    assert!(err <= 1e-12, "Kp 8 — ap 207: got {} want 8.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 9 — ap 400, the table's last point
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Ratio::new(400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 9.0);
    assert!(err <= 1e-12, "Kp 9 — ap 400, the table's last point: got {} want 9.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// above the table — ap 1000 holds at Kp 9 rather than extrapolating
///
/// Provenance: `published-source`, source `iaga_kp_ap`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Ratio::new(1000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 9.0);
    assert!(err <= 1e-12, "above the table — ap 1000 holds at Kp 9 rather than extrapolating: got {} want 9.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «Kp 0 — ap 0, the table's first point.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(0.0)).expect("Kp 0 — ap 0, the table's first point.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.0);
    assert!(err <= 1e-12, "Kp 0 — ap 0, the table's first point.: got {} and the author's code gave 0.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Kp 1 — ap 4.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(4.0)).expect("Kp 1 — ap 4.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "Kp 1 — ap 4.: got {} and the author's code gave 1.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Kp 2 — ap 7.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(7.0)).expect("Kp 2 — ap 7.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 2.0);
    assert!(err <= 1e-12, "Kp 2 — ap 7.: got {} and the author's code gave 2.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Kp 3 — ap 15, the legacy study's own climatology constant, and what env_kp declares by hand.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(15.0)).expect("Kp 3 — ap 15, the legacy study's own climatology constant, and what env_kp declares by hand.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 3.0);
    assert!(err <= 1e-12, "Kp 3 — ap 15, the legacy study's own climatology constant, and what env_kp declares by hand.: got {} and the author's code gave 3.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Kp 4 — ap 27.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(27.0)).expect("Kp 4 — ap 27.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 4.0);
    assert!(err <= 1e-12, "Kp 4 — ap 27.: got {} and the author's code gave 4.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Kp 5 — ap 48, the storm threshold.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Ratio::new(48.0)).expect("Kp 5 — ap 48, the storm threshold.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 5.0);
    assert!(err <= 1e-12, "Kp 5 — ap 48, the storm threshold.: got {} and the author's code gave 5.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Kp 6 — ap 80.», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Ratio::new(80.0)).expect("Kp 6 — ap 80.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 6.0);
    assert!(err <= 1e-12, "Kp 6 — ap 80.: got {} and the author's code gave 6.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Kp 7 — ap 132.», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Ratio::new(132.0)).expect("Kp 7 — ap 132.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 7.0);
    assert!(err <= 1e-12, "Kp 7 — ap 132.: got {} and the author's code gave 7.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Kp 8 — ap 207.», from their own  code.
#[test]
fn case_9() {
    let got = model::evaluate(Ratio::new(207.0)).expect("Kp 8 — ap 207.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 8.0);
    assert!(err <= 1e-12, "Kp 8 — ap 207.: got {} and the author's code gave 8.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Kp 9 — ap 400, the table's last point.», from their own  code.
#[test]
fn case_10() {
    let got = model::evaluate(Ratio::new(400.0)).expect("Kp 9 — ap 400, the table's last point.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 9.0);
    assert!(err <= 1e-12, "Kp 9 — ap 400, the table's last point.: got {} and the author's code gave 9.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «above the table — ap 1000 holds at Kp 9 rather than extrapolating.», from their own  code.
#[test]
fn case_11() {
    let got = model::evaluate(Ratio::new(1000.0)).expect("above the table — ap 1000 holds at Kp 9 rather than extrapolating.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 9.0);
    assert!(err <= 1e-12, "above the table — ap 1000 holds at Kp 9 rather than extrapolating.: got {} and the author's code gave 9.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_12() {
    let got = model::evaluate(Ratio::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 78 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_kp_from_ap::evaluate;
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x0000000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x0000000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x0000000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x0000000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x0000000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x0000000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x0000000000000000), "at (0.0)");
    assert_eq!(evaluate(2.0).map(f64::to_bits), Ok(0x3fd5555555555555), "at (2.0)");
    assert_eq!(evaluate(3.6).map(f64::to_bits), Ok(0x3febbbbbbbbbbbbc), "at (3.6)");
    assert_eq!(evaluate(3.96).map(f64::to_bits), Ok(0x3fef92c5f92c5f92), "at (3.96)");
    assert_eq!(evaluate(4.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (4.0)");
    assert_eq!(evaluate(4.04).map(f64::to_bits), Ok(0x3ff0369d0369d037), "at (4.04)");
    assert_eq!(evaluate(4.4).map(f64::to_bits), Ok(0x3ff2222222222222), "at (4.4)");
    assert_eq!(evaluate(8.0).map(f64::to_bits), Ok(0x4001555555555556), "at (8.0)");
    assert_eq!(evaluate(3.5).map(f64::to_bits), Ok(0x3feaaaaaaaaaaaaa), "at (3.5)");
    assert_eq!(evaluate(6.3).map(f64::to_bits), Ok(0x3ffc444444444444), "at (6.3)");
    assert_eq!(evaluate(6.93).map(f64::to_bits), Ok(0x3fffa06d3a06d3a0), "at (6.93)");
    assert_eq!(evaluate(7.0).map(f64::to_bits), Ok(0x4000000000000000), "at (7.0)");
    assert_eq!(evaluate(7.07).map(f64::to_bits), Ok(0x400017e4b17e4b18), "at (7.07)");
    assert_eq!(evaluate(7.700000000000001).map(f64::to_bits), Ok(0x4000eeeeeeeeeeef), "at (7.700000000000001)");
    assert_eq!(evaluate(14.0).map(f64::to_bits), Ok(0x40071c71c71c71c7), "at (14.0)");
    assert_eq!(evaluate(7.5).map(f64::to_bits), Ok(0x4000aaaaaaaaaaab), "at (7.5)");
    assert_eq!(evaluate(13.5).map(f64::to_bits), Ok(0x4006aaaaaaaaaaaa), "at (13.5)");
    assert_eq!(evaluate(14.85).map(f64::to_bits), Ok(0x4007ddddddddddde), "at (14.85)");
    assert_eq!(evaluate(15.0).map(f64::to_bits), Ok(0x4008000000000000), "at (15.0)");
    assert_eq!(evaluate(15.15).map(f64::to_bits), Ok(0x4008222222222222), "at (15.15)");
    assert_eq!(evaluate(16.5).map(f64::to_bits), Ok(0x4009555555555556), "at (16.5)");
    assert_eq!(evaluate(30.0).map(f64::to_bits), Ok(0x4010cccccccccccd), "at (30.0)");
    assert_eq!(evaluate(13.5).map(f64::to_bits), Ok(0x4006aaaaaaaaaaaa), "at (13.5)");
    assert_eq!(evaluate(24.3).map(f64::to_bits), Ok(0x400e8f5c28f5c28f), "at (24.3)");
    assert_eq!(evaluate(26.73).map(f64::to_bits), Ok(0x400fdb22d0e56042), "at (26.73)");
    assert_eq!(evaluate(27.0).map(f64::to_bits), Ok(0x4010000000000000), "at (27.0)");
    assert_eq!(evaluate(27.27).map(f64::to_bits), Ok(0x4010126e978d4fdf), "at (27.27)");
    assert_eq!(evaluate(29.700000000000003).map(f64::to_bits), Ok(0x4010b851eb851eb8), "at (29.700000000000003)");
    assert_eq!(evaluate(54.0).map(f64::to_bits), Ok(0x4015000000000000), "at (54.0)");
    assert_eq!(evaluate(24.0).map(f64::to_bits), Ok(0x400e666666666666), "at (24.0)");
    assert_eq!(evaluate(43.2).map(f64::to_bits), Ok(0x401349f49f49f4a0), "at (43.2)");
    assert_eq!(evaluate(47.519999999999996).map(f64::to_bits), Ok(0x4013edcba9876543), "at (47.519999999999996)");
    assert_eq!(evaluate(48.0).map(f64::to_bits), Ok(0x4014000000000000), "at (48.0)");
    assert_eq!(evaluate(48.480000000000004).map(f64::to_bits), Ok(0x4014147ae147ae15), "at (48.480000000000004)");
    assert_eq!(evaluate(52.800000000000004).map(f64::to_bits), Ok(0x4014cccccccccccd), "at (52.800000000000004)");
    assert_eq!(evaluate(96.0).map(f64::to_bits), Ok(0x40197d7d7d7d7d7d), "at (96.0)");
    assert_eq!(evaluate(40.0).map(f64::to_bits), Ok(0x4012d097b425ed0a), "at (40.0)");
    assert_eq!(evaluate(72.0).map(f64::to_bits), Ok(0x40172df2df2df2df), "at (72.0)");
    assert_eq!(evaluate(79.2).map(f64::to_bits), Ok(0x4017eafeafeafeb0), "at (79.2)");
    assert_eq!(evaluate(80.0).map(f64::to_bits), Ok(0x4018000000000000), "at (80.0)");
    assert_eq!(evaluate(80.8).map(f64::to_bits), Ok(0x4018138138138138), "at (80.8)");
    assert_eq!(evaluate(88.0).map(f64::to_bits), Ok(0x4018c30c30c30c31), "at (88.0)");
    assert_eq!(evaluate(160.0).map(f64::to_bits), Ok(0x401da740da740da7), "at (160.0)");
    assert_eq!(evaluate(66.0).map(f64::to_bits), Ok(0x40168ba2e8ba2e8c), "at (66.0)");
    assert_eq!(evaluate(118.8).map(f64::to_bits), Ok(0x401b297297297297), "at (118.8)");
    assert_eq!(evaluate(130.68).map(f64::to_bits), Ok(0x401bea8b7584250f), "at (130.68)");
    assert_eq!(evaluate(132.0).map(f64::to_bits), Ok(0x401c000000000000), "at (132.0)");
    assert_eq!(evaluate(133.32).map(f64::to_bits), Ok(0x401c147ae147ae14), "at (133.32)");
    assert_eq!(evaluate(145.20000000000002).map(f64::to_bits), Ok(0x401ccccccccccccd), "at (145.20000000000002)");
    assert_eq!(evaluate(264.0).map(f64::to_bits), Ok(0x4020f55555555555), "at (264.0)");
    assert_eq!(evaluate(103.5).map(f64::to_bits), Ok(0x401a141414141414), "at (103.5)");
    assert_eq!(evaluate(186.3).map(f64::to_bits), Ok(0x401f03a83a83a83b), "at (186.3)");
    assert_eq!(evaluate(204.93).map(f64::to_bits), Ok(0x401fe6c405d9f739), "at (204.93)");
    assert_eq!(evaluate(207.0).map(f64::to_bits), Ok(0x4020000000000000), "at (207.0)");
    assert_eq!(evaluate(209.07).map(f64::to_bits), Ok(0x40200c2e9c125c83), "at (209.07)");
    assert_eq!(evaluate(227.70000000000002).map(f64::to_bits), Ok(0x402079d218b79d22), "at (227.70000000000002)");
    assert_eq!(evaluate(414.0).map(f64::to_bits), Ok(0x4022000000000000), "at (414.0)");
    assert_eq!(evaluate(200.0).map(f64::to_bits), Ok(0x401faaaaaaaaaaab), "at (200.0)");
    assert_eq!(evaluate(360.0).map(f64::to_bits), Ok(0x4021bbbbbbbbbbbc), "at (360.0)");
    assert_eq!(evaluate(396.0).map(f64::to_bits), Ok(0x4021f92c5f92c5f9), "at (396.0)");
    assert_eq!(evaluate(400.0).map(f64::to_bits), Ok(0x4022000000000000), "at (400.0)");
    assert_eq!(evaluate(404.0).map(f64::to_bits), Ok(0x4022000000000000), "at (404.0)");
    assert_eq!(evaluate(440.00000000000006).map(f64::to_bits), Ok(0x4022000000000000), "at (440.00000000000006)");
    assert_eq!(evaluate(800.0).map(f64::to_bits), Ok(0x4022000000000000), "at (800.0)");
    assert_eq!(evaluate(500.0).map(f64::to_bits), Ok(0x4022000000000000), "at (500.0)");
    assert_eq!(evaluate(900.0).map(f64::to_bits), Ok(0x4022000000000000), "at (900.0)");
    assert_eq!(evaluate(990.0).map(f64::to_bits), Ok(0x4022000000000000), "at (990.0)");
    assert_eq!(evaluate(1000.0).map(f64::to_bits), Ok(0x4022000000000000), "at (1000.0)");
    assert_eq!(evaluate(1010.0).map(f64::to_bits), Ok(0x4022000000000000), "at (1010.0)");
    assert_eq!(evaluate(1100.0).map(f64::to_bits), Ok(0x4022000000000000), "at (1100.0)");
    assert_eq!(evaluate(2000.0).map(f64::to_bits), Ok(0x4022000000000000), "at (2000.0)");
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
/// Derived from `Kp 0 — ap 0, the table's first point` and the declared domain 0 … 9.
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
        if let Err(f) = model::evaluate(Ratio::new(0.0 * scale)) {
            refused.push(format!("ap x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_kp_from_ap refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 9 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(0.0 * scale)) {
            assert!(v.get().is_finite(), "sw_kp_from_ap produced a value that is not a number for Kp");
            assert!(v.get() >= 0.0 && v.get() <= 9.0, "sw_kp_from_ap answered {} for Kp, outside its declared domain 0 … 9 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(0.0));
    let b = model::evaluate(Ratio::new(0.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_kp_from_ap is not deterministic for Kp: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_kp_from_ap refused on one call and answered on the other"),
    }
}

/// The prior implementation, over the grid it was exported on.
///
/// Migrated from `prf_ap2kp.m:100 (01_kernel/sw_study/06_density, local function tbl_)`. These numbers are a second opinion and never
/// an expected value: an implementation cannot supply its own, and the
/// prior tool is an implementation. A disagreement is a finding about
/// one of the two.
#[test]
fn agrees_with_the_prior_implementation() {
    const GRID: &str = include_str!("parity.csv");
    const TOL: f64 = 1e-12;

    let mut lines = GRID
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'));
    let header: Vec<&str> = lines
        .next()
        .expect("parity.csv is empty — a grid with no header is not a grid")
        .split(',')
        .map(str::trim)
        .collect();

    // Columns are keyed by binding, as fixtures are, so a grid exported
    // with the columns in another order still lines up — and one missing
    // a column this node reads fails by name rather than by position.
    let want = ["ap"];
    let col: Vec<usize> = want
        .iter()
        .map(|w| {
            header.iter().position(|h| h == w).unwrap_or_else(|| {
                panic!("parity.csv has no column '{w}' — this node reads it, so the grid cannot be compared. Columns present: {header:?}")
            })
        })
        .collect();
    let out = header.len() - 1;
    assert!(
        header[out].starts_with("matlab_"),
        "the last column of parity.csv is '{}' — it must be the prior implementation's answer, named matlab_<symbol>, so that a column added on the end cannot silently become the thing being compared",
        header[out]
    );

    let mut rows = 0usize;
    let mut worst = 0.0f64;
    let mut findings = Vec::<String>::new();
    for (n, line) in lines.enumerate() {
        let line_no = n + 2;
        let row: Vec<f64> = line
            .split(',')
            .map(|c| {
                c.trim().parse::<f64>().unwrap_or_else(|_| {
                    panic!("parity.csv line {line_no}: '{}' is not a number", c.trim())
                })
            })
            .collect();
        assert_eq!(
            row.len(),
            header.len(),
            "parity.csv line {line_no}: {} value(s) against {} column(s)",
            row.len(),
            header.len()
        );
        rows += 1;

        // A refusal here is itself a finding: the prior implementation
        // answered this point, so either its inputs were outside a domain
        // this node declares too narrowly, or the guard is wrong.
        let got = match model::evaluate(Ratio::new(row[col[0]])) {
            Ok(v) => v.get(),
            Err(e) => {
                findings.push(format!(
                    "line {line_no}: this engine refused a point the prior implementation answered ({e:?})"
                ));
                continue;
            }
        };
        let err = relative_error(got, row[out]);
        if err > worst {
            worst = err;
        }
        if err > TOL {
            findings.push(format!(
                "line {line_no}: this engine {got}, the prior implementation {}, relative difference {err}",
                row[out]
            ));
        }
    }

    assert!(
        rows > 0,
        "parity.csv has a header and no rows — a grid that compares nothing passes, which is worse than not having one"
    );
    assert!(
        findings.is_empty(),
        "sw_kp_from_ap: {} of {rows} grid row(s) disagree with the prior implementation `prf_ap2kp.m:100 (01_kernel/sw_study/06_density, local function tbl_)` beyond {TOL} (worst {worst}).\n{}\nThis is a finding about one of the two implementations, not a build failure and not proof this one is wrong — both classes have been found before. Take it to the node owner. Do not widen parity_tolerance and do not edit parity.csv to agree.",
        findings.len(),
        findings.join("\n")
    );
}

