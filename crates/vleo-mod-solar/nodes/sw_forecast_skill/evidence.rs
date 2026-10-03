// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_forecast_skill`.
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

/// lead 13 — mid-window, still strongly skillful — +0.402 from 1237 pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(1123200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.4016285468);
    assert!(err <= 1e-12, "lead 13 — mid-window, still strongly skillful — +0.402 from 1237 pairs: got {} want 0.4016285468, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 1 — the outlook's weakest positive skill, where persistence is hardest to beat
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(86400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.068515911);
    assert!(err <= 1e-12, "lead 1 — the outlook's weakest positive skill, where persistence is hardest to beat: got {} want 0.068515911, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 9 — peak skill over the whole window — +0.438 from 1241 pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(777600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.4375371731);
    assert!(err <= 1e-12, "lead 9 — peak skill over the whole window — +0.438 from 1241 pairs: got {} want 0.4375371731, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 13.5 — between two measured leads, so this one tests the interpolation and not the table
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(1166400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.41015283365);
    assert!(err <= 1e-12, "lead 13.5 — between two measured leads, so this one tests the interpolation and not the table: got {} want 0.41015283365, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 23 — the last positive lead, +0.018, and the outlook is all but worthless here
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(1987200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.0176508866);
    assert!(err <= 1e-12, "lead 23 — the last positive lead, +0.018, and the outlook is all but worthless here: got {} want 0.0176508866, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 24 — the sign change: the outlook is now worse than assuming nothing changes
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Time::new(2073600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.0355195984);
    assert!(err <= 1e-12, "lead 24 — the sign change: the outlook is now worse than assuming nothing changes: got {} want -0.0355195984, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 26 — the declared lead — -0.022 from 868 pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Time::new(2246400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.0220863078);
    assert!(err <= 1e-12, "lead 26 — the declared lead — -0.022 from 868 pairs: got {} want -0.0220863078, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 13 — mid-window, still strongly skillful — +0.402 from 1237 pairs.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Time::new(1123200.0)).expect("lead 13 — mid-window, still strongly skillful — +0.402 from 1237 pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.4016285468);
    assert!(err <= 1e-12, "lead 13 — mid-window, still strongly skillful — +0.402 from 1237 pairs.: got {} and the author's code gave 0.4016285468; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 1 — the outlook's weakest positive skill, where persistence is hardest to beat.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Time::new(86400.0)).expect("lead 1 — the outlook's weakest positive skill, where persistence is hardest to beat.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.068515911);
    assert!(err <= 1e-12, "lead 1 — the outlook's weakest positive skill, where persistence is hardest to beat.: got {} and the author's code gave 0.068515911; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 9 — peak skill over the whole window — +0.438 from 1241 pairs.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Time::new(777600.0)).expect("lead 9 — peak skill over the whole window — +0.438 from 1241 pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.4375371731);
    assert!(err <= 1e-12, "lead 9 — peak skill over the whole window — +0.438 from 1241 pairs.: got {} and the author's code gave 0.4375371731; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 13.5 — between two measured leads, so this one tests the interpolation and not the table.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Time::new(1166400.0)).expect("lead 13.5 — between two measured leads, so this one tests the interpolation and not the table.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.41015283365);
    assert!(err <= 1e-12, "lead 13.5 — between two measured leads, so this one tests the interpolation and not the table.: got {} and the author's code gave 0.41015283365; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 23 — the last positive lead, +0.018, and the outlook is all but worthless here.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Time::new(1987200.0)).expect("lead 23 — the last positive lead, +0.018, and the outlook is all but worthless here.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.0176508866);
    assert!(err <= 1e-12, "lead 23 — the last positive lead, +0.018, and the outlook is all but worthless here.: got {} and the author's code gave 0.0176508866; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 24 — the sign change: the outlook is now worse than assuming nothing changes.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Time::new(2073600.0)).expect("lead 24 — the sign change: the outlook is now worse than assuming nothing changes.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), -0.0355195984);
    assert!(err <= 1e-12, "lead 24 — the sign change: the outlook is now worse than assuming nothing changes.: got {} and the author's code gave -0.0355195984; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 26 — the declared lead — -0.022 from 868 pairs.», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Time::new(2246400.0)).expect("lead 26 — the declared lead — -0.022 from 868 pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), -0.0220863078);
    assert!(err <= 1e-12, "lead 26 — the declared lead — -0.022 from 868 pairs.: got {} and the author's code gave -0.0220863078; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Time::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 50 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_forecast_skill::evaluate;
    assert_eq!(evaluate(561600.0).map(f64::to_bits), Ok(0x3fd5c70edc989b0d), "at (561600.0)");
    assert_eq!(evaluate(1010880.0).map(f64::to_bits), Ok(0x3fda942d7abfc046), "at (1010880.0)");
    assert_eq!(evaluate(1111968.0).map(f64::to_bits), Ok(0x3fd9ce237b8b83fe), "at (1111968.0)");
    assert_eq!(evaluate(1123200.0).map(f64::to_bits), Ok(0x3fd9b44838695827), "at (1123200.0)");
    assert_eq!(evaluate(1134432.0).map(f64::to_bits), Ok(0x3fd9d8981e127fc5), "at (1134432.0)");
    assert_eq!(evaluate(1235520.0).map(f64::to_bits), Ok(0x3fda47a440a0b3dc), "at (1235520.0)");
    assert_eq!(evaluate(2246400.0).map(f64::to_bits), Ok(0xbf969dcb06c2f585), "at (2246400.0)");
    assert_eq!(evaluate(43200.0).map(f64::to_bits), Ok(0x3fb18a423d002a61), "at (43200.0)");
    assert_eq!(evaluate(77760.0).map(f64::to_bits), Ok(0x3fb18a423d002a61), "at (77760.0)");
    assert_eq!(evaluate(85536.0).map(f64::to_bits), Ok(0x3fb18a423d002a61), "at (85536.0)");
    assert_eq!(evaluate(86400.0).map(f64::to_bits), Ok(0x3fb18a423d002a61), "at (86400.0)");
    assert_eq!(evaluate(87264.0).map(f64::to_bits), Ok(0x3fb1dfaaf8fc48bb), "at (87264.0)");
    assert_eq!(evaluate(95040.00000000001).map(f64::to_bits), Ok(0x3fb4e05994d959e7), "at (95040.00000000001)");
    assert_eq!(evaluate(172800.0).map(f64::to_bits), Ok(0x3fc97395d5be02c8), "at (172800.0)");
    assert_eq!(evaluate(388800.0).map(f64::to_bits), Ok(0x3fcf67fad42148de), "at (388800.0)");
    assert_eq!(evaluate(699840.0).map(f64::to_bits), Ok(0x3fd9f5ea50abeb06), "at (699840.0)");
    assert_eq!(evaluate(769824.0).map(f64::to_bits), Ok(0x3fdbcc56f48c527a), "at (769824.0)");
    assert_eq!(evaluate(777600.0).map(f64::to_bits), Ok(0x3fdc009bea4fec31), "at (777600.0)");
    assert_eq!(evaluate(785376.0).map(f64::to_bits), Ok(0x3fdbf942034fde74), "at (785376.0)");
    assert_eq!(evaluate(855360.0000000001).map(f64::to_bits), Ok(0x3fdbb718e44f62d0), "at (855360.0000000001)");
    assert_eq!(evaluate(1555200.0).map(f64::to_bits), Ok(0x3fcfaef57fd72252), "at (1555200.0)");
    assert_eq!(evaluate(583200.0).map(f64::to_bits), Ok(0x3fd67de6c9f7501c), "at (583200.0)");
    assert_eq!(evaluate(1049760.0).map(f64::to_bits), Ok(0x3fda5d58030db1dd), "at (1049760.0)");
    assert_eq!(evaluate(1154736.0).map(f64::to_bits), Ok(0x3fda1a3c3d3a6ec4), "at (1154736.0)");
    assert_eq!(evaluate(1166400.0).map(f64::to_bits), Ok(0x3fda3ff1abb8dcd4), "at (1166400.0)");
    assert_eq!(evaluate(1178064.0).map(f64::to_bits), Ok(0x3fda65a71a374ae4), "at (1178064.0)");
    assert_eq!(evaluate(1283040.0).map(f64::to_bits), Ok(0x3fd955b4fe37f583), "at (1283040.0)");
    assert_eq!(evaluate(2332800.0).map(f64::to_bits), Ok(0xbf969dcb06c2f585), "at (2332800.0)");
    assert_eq!(evaluate(993600.0).map(f64::to_bits), Ok(0x3fdaa4d80d0f0a9e), "at (993600.0)");
    assert_eq!(evaluate(1788480.0).map(f64::to_bits), Ok(0x3fc3193cfb6d230b), "at (1788480.0)");
    assert_eq!(evaluate(1967328.0).map(f64::to_bits), Ok(0x3fa033cb0098bf60), "at (1967328.0)");
    assert_eq!(evaluate(1987200.0).map(f64::to_bits), Ok(0x3f921312f2c505b8), "at (1987200.0)");
    assert_eq!(evaluate(2007072.0).map(f64::to_bits), Ok(0x3f763509d077f7ea), "at (2007072.0)");
    assert_eq!(evaluate(2185920.0).map(f64::to_bits), Ok(0xbf9e0ad10bc32424), "at (2185920.0)");
    assert_eq!(evaluate(3974400.0).map(f64::to_bits), Ok(0xbf969dcb06c2f585), "at (3974400.0)");
    assert_eq!(evaluate(1036800.0).map(f64::to_bits), Ok(0x3fda7b2d9f48d0c1), "at (1036800.0)");
    assert_eq!(evaluate(1866240.0).map(f64::to_bits), Ok(0x3fba70b10003292c), "at (1866240.0)");
    assert_eq!(evaluate(2052864.0).map(f64::to_bits), Ok(0xbf974e0d4b8f6ce8), "at (2052864.0)");
    assert_eq!(evaluate(2073600.0).map(f64::to_bits), Ok(0xbfa22f9ff2fd77b5), "at (2073600.0)");
    assert_eq!(evaluate(2094336.0).map(f64::to_bits), Ok(0xbfa1cef0944e8d99), "at (2094336.0)");
    assert_eq!(evaluate(2280960.0).map(f64::to_bits), Ok(0xbf969dcb06c2f585), "at (2280960.0)");
    assert_eq!(evaluate(4147200.0).map(f64::to_bits), Ok(0xbf969dcb06c2f585), "at (4147200.0)");
    assert_eq!(evaluate(1123200.0).map(f64::to_bits), Ok(0x3fd9b44838695827), "at (1123200.0)");
    assert_eq!(evaluate(2021760.0).map(f64::to_bits), Ok(0xbf6da20b1f714918), "at (2021760.0)");
    assert_eq!(evaluate(2223936.0).map(f64::to_bits), Ok(0xbf995fe3348132b9), "at (2223936.0)");
    assert_eq!(evaluate(2246400.0).map(f64::to_bits), Ok(0xbf969dcb06c2f585), "at (2246400.0)");
    assert_eq!(evaluate(2268864.0).map(f64::to_bits), Ok(0xbf969dcb06c2f585), "at (2268864.0)");
    assert_eq!(evaluate(2471040.0).map(f64::to_bits), Ok(0xbf969dcb06c2f585), "at (2471040.0)");
    assert_eq!(evaluate(4492800.0).map(f64::to_bits), Ok(0xbf969dcb06c2f585), "at (4492800.0)");
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
/// Derived from `lead 13 — mid-window, still strongly skillful — +0.402 from 1237 pairs` and the declared domain -0.1 … 0.5.
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
        if let Err(f) = model::evaluate(Time::new(1123200.0 * scale)) {
            refused.push(format!("lead x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_forecast_skill refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain -0.1 … 0.5 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Time::new(1123200.0 * scale)) {
            assert!(v.get().is_finite(), "sw_forecast_skill produced a value that is not a number for S_f107");
            assert!(v.get() >= -0.1 && v.get() <= 0.5, "sw_forecast_skill answered {} for S_f107, outside its declared domain -0.1 … 0.5 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Time::new(1123200.0));
    let b = model::evaluate(Time::new(1123200.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_forecast_skill is not deterministic for S_f107: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_forecast_skill refused on one call and answered on the other"),
    }
}

