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

