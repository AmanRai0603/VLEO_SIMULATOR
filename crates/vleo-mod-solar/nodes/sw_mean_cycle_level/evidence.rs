// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_mean_cycle_level`.
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

/// phase 0.025 — 418 days from two cycles — mean F10.7 71.76 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(0.025)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 71.7632);
    assert!(err <= 1e-12, "phase 0.025 — 418 days from two cycles — mean F10.7 71.76 sfu: got {} want 71.7632, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.075 — 418 days from two cycles — mean F10.7 83.89 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(0.075)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 83.8947);
    assert!(err <= 1e-12, "phase 0.075 — 418 days from two cycles — mean F10.7 83.89 sfu: got {} want 83.8947, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.125 — 418 days from two cycles — mean F10.7 99.22 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(0.125)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 99.2177);
    assert!(err <= 1e-12, "phase 0.125 — 418 days from two cycles — mean F10.7 99.22 sfu: got {} want 99.2177, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.175 — 418 days from two cycles — mean F10.7 111.05 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(0.175)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 111.0502);
    assert!(err <= 1e-12, "phase 0.175 — 418 days from two cycles — mean F10.7 111.05 sfu: got {} want 111.0502, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.225 — 418 days from two cycles — mean F10.7 135.66 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(0.225)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 135.6555);
    assert!(err <= 1e-12, "phase 0.225 — 418 days from two cycles — mean F10.7 135.66 sfu: got {} want 135.6555, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.275 — 418 days from two cycles — mean F10.7 159.66 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(0.275)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 159.6603);
    assert!(err <= 1e-12, "phase 0.275 — 418 days from two cycles — mean F10.7 159.66 sfu: got {} want 159.6603, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.325 — 416 days from two cycles — mean F10.7 145.37 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(0.325)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 145.3702);
    assert!(err <= 1e-12, "phase 0.325 — 416 days from two cycles — mean F10.7 145.37 sfu: got {} want 145.3702, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.375 — 418 days from two cycles — mean F10.7 146.73 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(0.375)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 146.7273);
    assert!(err <= 1e-12, "phase 0.375 — 418 days from two cycles — mean F10.7 146.73 sfu: got {} want 146.7273, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.425 — 418 days from two cycles — mean F10.7 165.28 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(0.425)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 165.2799);
    assert!(err <= 1e-12, "phase 0.425 — 418 days from two cycles — mean F10.7 165.28 sfu: got {} want 165.2799, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.475 — 415 days from two cycles — mean F10.7 160.33 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Ratio::new(0.475)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 160.3301);
    assert!(err <= 1e-12, "phase 0.475 — 415 days from two cycles — mean F10.7 160.33 sfu: got {} want 160.3301, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.525 — 418 days from two cycles — mean F10.7 135.32 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Ratio::new(0.525)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 135.3182);
    assert!(err <= 1e-12, "phase 0.525 — 418 days from two cycles — mean F10.7 135.32 sfu: got {} want 135.3182, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.575 — 418 days from two cycles — mean F10.7 126.23 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_11() {
    let got = model::evaluate(Ratio::new(0.575)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 126.2321);
    assert!(err <= 1e-12, "phase 0.575 — 418 days from two cycles — mean F10.7 126.23 sfu: got {} want 126.2321, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.625 — 418 days from two cycles — mean F10.7 105.84 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_12() {
    let got = model::evaluate(Ratio::new(0.625)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 105.8445);
    assert!(err <= 1e-12, "phase 0.625 — 418 days from two cycles — mean F10.7 105.84 sfu: got {} want 105.8445, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.675 — 417 days from two cycles — mean F10.7 95.51 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_13() {
    let got = model::evaluate(Ratio::new(0.675)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 95.5084);
    assert!(err <= 1e-12, "phase 0.675 — 417 days from two cycles — mean F10.7 95.51 sfu: got {} want 95.5084, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.725 — 358 days from two cycles — mean F10.7 86.99 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_14() {
    let got = model::evaluate(Ratio::new(0.725)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 86.9916);
    assert!(err <= 1e-12, "phase 0.725 — 358 days from two cycles — mean F10.7 86.99 sfu: got {} want 86.9916, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.775 — 217 days from two cycles — mean F10.7 80.92 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_15() {
    let got = model::evaluate(Ratio::new(0.775)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 80.9171);
    assert!(err <= 1e-12, "phase 0.775 — 217 days from two cycles — mean F10.7 80.92 sfu: got {} want 80.9171, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.825 — 406 days from two cycles — mean F10.7 76.21 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_16() {
    let got = model::evaluate(Ratio::new(0.825)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 76.2094);
    assert!(err <= 1e-12, "phase 0.825 — 406 days from two cycles — mean F10.7 76.21 sfu: got {} want 76.2094, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.875 — 418 days from two cycles — mean F10.7 71.05 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_17() {
    let got = model::evaluate(Ratio::new(0.875)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 71.0478);
    assert!(err <= 1e-12, "phase 0.875 — 418 days from two cycles — mean F10.7 71.05 sfu: got {} want 71.0478, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.925 — 418 days from two cycles — mean F10.7 71.54 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_18() {
    let got = model::evaluate(Ratio::new(0.925)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 71.5383);
    assert!(err <= 1e-12, "phase 0.925 — 418 days from two cycles — mean F10.7 71.54 sfu: got {} want 71.5383, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// phase 0.975 — 416 days from two cycles — mean F10.7 67.65 sfu
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_19() {
    let got = model::evaluate(Ratio::new(0.975)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 67.6538);
    assert!(err <= 1e-12, "phase 0.975 — 416 days from two cycles — mean F10.7 67.65 sfu: got {} want 67.6538, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the declared epoch's phase of 0.6194 falls in the 0.625 bin — 105.84 sfu, against the unconditional mean of 114.84
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_20() {
    let got = model::evaluate(Ratio::new(0.625)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 105.8445);
    assert!(err <= 1e-12, "the declared epoch's phase of 0.6194 falls in the 0.625 bin — 105.84 sfu, against the unconditional mean of 114.84: got {} want 105.8445, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `phase 0.025 — 418 days from two cycles — mean F10.7 71.76 sfu` and the declared domain 60 … 400.
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
        if let Err(f) = model::evaluate(Ratio::new(0.025 * scale)) {
            refused.push(format!("phase x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_mean_cycle_level refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 60 … 400 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(0.025 * scale)) {
            assert!(v.get().is_finite(), "sw_mean_cycle_level produced a value that is not a number");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_mean_cycle_level answered {}, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(0.025));
    let b = model::evaluate(Ratio::new(0.025));
    match (a, b) {
        (Ok(x), Ok(y)) => assert!(x.get().to_bits() == y.get().to_bits(), "sw_mean_cycle_level is not deterministic: {} then {}", x.get(), y.get()),
        (Err(_), Err(_)) => {}
        _ => panic!("sw_mean_cycle_level refused on one call and answered on the other"),
    }
}

