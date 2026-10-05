// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_forecast_bias`.
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

/// lead 13 — the interior recovery in the bias curve — -1.848 sfu from 1237 pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(1123200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -1.8480194018);
    assert!(err <= 1e-12, "lead 13 — the interior recovery in the bias curve — -1.848 sfu from 1237 pairs: got {} want -1.8480194018, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 1 — the shallowest bias in the window — -0.593 sfu from 1241 pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(86400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -0.5930701048);
    assert!(err <= 1e-12, "lead 1 — the shallowest bias in the window — -0.593 sfu from 1241 pairs: got {} want -0.5930701048, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 9 — the interior minimum — -2.748 sfu from 1241 pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(777600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -2.7477840451);
    assert!(err <= 1e-12, "lead 9 — the interior minimum — -2.748 sfu from 1241 pairs: got {} want -2.7477840451, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 13.5 — between two measured leads, so this one tests the interpolation and not the table
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(1166400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -1.88742433505);
    assert!(err <= 1e-12, "lead 13.5 — between two measured leads, so this one tests the interpolation and not the table: got {} want -1.88742433505, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 23 — the last lead where the outlook still beats persistence — -2.945 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(1987200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -2.9447640967);
    assert!(err <= 1e-12, "lead 23 — the last lead where the outlook still beats persistence — -2.945 sfu: got {} want -2.9447640967, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 26 — the declared lead and the deepest bias in the window — -3.968 sfu from 868 pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Time::new(2246400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), -3.9677419355);
    assert!(err <= 1e-12, "lead 26 — the declared lead and the deepest bias in the window — -3.968 sfu from 868 pairs: got {} want -3.9677419355, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 13 — the interior recovery in the bias curve — -1.848 sfu from 1237 pairs.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Time::new(1123200.0)).expect("lead 13 — the interior recovery in the bias curve — -1.848 sfu from 1237 pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), -1.8480194018);
    assert!(err <= 1e-12, "lead 13 — the interior recovery in the bias curve — -1.848 sfu from 1237 pairs.: got {} and the author's code gave -1.8480194018; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 1 — the shallowest bias in the window — -0.593 sfu from 1241 pairs.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Time::new(86400.0)).expect("lead 1 — the shallowest bias in the window — -0.593 sfu from 1241 pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), -0.5930701048);
    assert!(err <= 1e-12, "lead 1 — the shallowest bias in the window — -0.593 sfu from 1241 pairs.: got {} and the author's code gave -0.5930701048; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 9 — the interior minimum — -2.748 sfu from 1241 pairs.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Time::new(777600.0)).expect("lead 9 — the interior minimum — -2.748 sfu from 1241 pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), -2.7477840451);
    assert!(err <= 1e-12, "lead 9 — the interior minimum — -2.748 sfu from 1241 pairs.: got {} and the author's code gave -2.7477840451; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 13.5 — between two measured leads, so this one tests the interpolation and not the table.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Time::new(1166400.0)).expect("lead 13.5 — between two measured leads, so this one tests the interpolation and not the table.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), -1.88742433505);
    assert!(err <= 1e-12, "lead 13.5 — between two measured leads, so this one tests the interpolation and not the table.: got {} and the author's code gave -1.88742433505; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 23 — the last lead where the outlook still beats persistence — -2.945 sfu.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Time::new(1987200.0)).expect("lead 23 — the last lead where the outlook still beats persistence — -2.945 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), -2.9447640967);
    assert!(err <= 1e-12, "lead 23 — the last lead where the outlook still beats persistence — -2.945 sfu.: got {} and the author's code gave -2.9447640967; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 26 — the declared lead and the deepest bias in the window — -3.968 sfu from 868 pairs.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Time::new(2246400.0)).expect("lead 26 — the declared lead and the deepest bias in the window — -3.968 sfu from 868 pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), -3.9677419355);
    assert!(err <= 1e-12, "lead 26 — the declared lead and the deepest bias in the window — -3.968 sfu from 868 pairs.: got {} and the author's code gave -3.9677419355; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Time::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 43 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_forecast_bias::evaluate;
    assert_eq!(evaluate(561600.0).map(f64::to_bits), Ok(0xc0040bd030a6de48), "at (561600.0)");
    assert_eq!(evaluate(1010880.0).map(f64::to_bits), Ok(0xc00016fa038a20c7), "at (1010880.0)");
    assert_eq!(evaluate(1111968.0).map(f64::to_bits), Ok(0xbffdc4c29fab130a), "at (1111968.0)");
    assert_eq!(evaluate(1123200.0).map(f64::to_bits), Ok(0xbffd917ccad1abf7), "at (1123200.0)");
    assert_eq!(evaluate(1134432.0).map(f64::to_bits), Ok(0xbffdbb73bff00c6e), "at (1134432.0)");
    assert_eq!(evaluate(1235520.0).map(f64::to_bits), Ok(0xc0007ee2df802521), "at (1235520.0)");
    assert_eq!(evaluate(2246400.0).map(f64::to_bits), Ok(0xc00fbdef7bdf859d), "at (2246400.0)");
    assert_eq!(evaluate(43200.0).map(f64::to_bits), Ok(0xbfe2fa6e280b3dca), "at (43200.0)");
    assert_eq!(evaluate(77760.0).map(f64::to_bits), Ok(0xbfe2fa6e280b3dca), "at (77760.0)");
    assert_eq!(evaluate(85536.0).map(f64::to_bits), Ok(0xbfe2fa6e280b3dca), "at (85536.0)");
    assert_eq!(evaluate(86400.0).map(f64::to_bits), Ok(0xbfe2fa6e280b3dca), "at (86400.0)");
    assert_eq!(evaluate(87264.0).map(f64::to_bits), Ok(0xbfe30b324855c7f1), "at (87264.0)");
    assert_eq!(evaluate(95040.00000000001).map(f64::to_bits), Ok(0xbfe3a2176af4a352), "at (95040.00000000001)");
    assert_eq!(evaluate(172800.0).map(f64::to_bits), Ok(0xbfe9870ac529351b), "at (172800.0)");
    assert_eq!(evaluate(388800.0).map(f64::to_bits), Ok(0xbffdbc877e813dc2), "at (388800.0)");
    assert_eq!(evaluate(699840.0).map(f64::to_bits), Ok(0xc004b1f0bb83ed12), "at (699840.0)");
    assert_eq!(evaluate(769824.0).map(f64::to_bits), Ok(0xc005da82745cda6e), "at (769824.0)");
    assert_eq!(evaluate(777600.0).map(f64::to_bits), Ok(0xc005fb7633916695), "at (777600.0)");
    assert_eq!(evaluate(785376.0).map(f64::to_bits), Ok(0xc005c9c6d9139c0d), "at (785376.0)");
    assert_eq!(evaluate(855360.0000000001).map(f64::to_bits), Ok(0xc0040a9caaa77d41), "at (855360.0000000001)");
    assert_eq!(evaluate(1555200.0).map(f64::to_bits), Ok(0xc00202ecfb9b8b44), "at (1555200.0)");
    assert_eq!(evaluate(583200.0).map(f64::to_bits), Ok(0xc00470a00cb2376c), "at (583200.0)");
    assert_eq!(evaluate(1049760.0).map(f64::to_bits), Ok(0xbffee0bb894b754c), "at (1049760.0)");
    assert_eq!(evaluate(1154736.0).map(f64::to_bits), Ok(0xbffe074fb61d1d46), "at (1154736.0)");
    assert_eq!(evaluate(1166400.0).map(f64::to_bits), Ok(0xbffe32e3dc0b6dc2), "at (1166400.0)");
    assert_eq!(evaluate(1178064.0).map(f64::to_bits), Ok(0xbffe5e7801f9be3e), "at (1178064.0)");
    assert_eq!(evaluate(1283040.0).map(f64::to_bits), Ok(0xc0027a3e1fc0fd9b), "at (1283040.0)");
    assert_eq!(evaluate(2332800.0).map(f64::to_bits), Ok(0xc00fbdef7bdf859d), "at (2332800.0)");
    assert_eq!(evaluate(993600.0).map(f64::to_bits), Ok(0xc00072547293b360), "at (993600.0)");
    assert_eq!(evaluate(1788480.0).map(f64::to_bits), Ok(0xc001fe006de4b916), "at (1788480.0)");
    assert_eq!(evaluate(1967328.0).map(f64::to_bits), Ok(0xc006d0e9523d921f), "at (1967328.0)");
    assert_eq!(evaluate(1987200.0).map(f64::to_bits), Ok(0xc0078ee07a8e1784), "at (1987200.0)");
    assert_eq!(evaluate(2007072.0).map(f64::to_bits), Ok(0xc0083a7fced73232), "at (2007072.0)");
    assert_eq!(evaluate(2185920.0).map(f64::to_bits), Ok(0xc00e908a79e31046), "at (2185920.0)");
    assert_eq!(evaluate(3974400.0).map(f64::to_bits), Ok(0xc00fbdef7bdf859d), "at (3974400.0)");
    assert_eq!(evaluate(1123200.0).map(f64::to_bits), Ok(0xbffd917ccad1abf7), "at (1123200.0)");
    assert_eq!(evaluate(2021760.0).map(f64::to_bits), Ok(0xc008b959bf39c059), "at (2021760.0)");
    assert_eq!(evaluate(2223936.0).map(f64::to_bits), Ok(0xc00f4dfd2aada32c), "at (2223936.0)");
    assert_eq!(evaluate(2246400.0).map(f64::to_bits), Ok(0xc00fbdef7bdf859d), "at (2246400.0)");
    assert_eq!(evaluate(2268864.0).map(f64::to_bits), Ok(0xc00fbdef7bdf859d), "at (2268864.0)");
    assert_eq!(evaluate(2471040.0).map(f64::to_bits), Ok(0xc00fbdef7bdf859d), "at (2471040.0)");
    assert_eq!(evaluate(4492800.0).map(f64::to_bits), Ok(0xc00fbdef7bdf859d), "at (4492800.0)");
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
/// Derived from `lead 13 — the interior recovery in the bias curve — -1.848 sfu from 1237 pairs` and the declared domain -4 … -0.5.
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
        "sw_forecast_bias refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain -4 … -0.5 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
            assert!(v.get().is_finite(), "sw_forecast_bias produced a value that is not a number for B_f107");
            assert!(v.get() >= -4.0 && v.get() <= -0.5, "sw_forecast_bias answered {} for B_f107, outside its declared domain -4 … -0.5 — the guard did not stop it", v.get());
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
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_forecast_bias is not deterministic for B_f107: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_forecast_bias refused on one call and answered on the other"),
    }
}

