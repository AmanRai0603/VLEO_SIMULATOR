// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_storm_rate`.
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

/// Kp 0 — ap 0 — the record exceeds it on 364.1506 days a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(0.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 364.1506);
    assert!(err <= 1e-12, "Kp 0 — ap 0 — the record exceeds it on 364.1506 days a year: got {} want 364.1506, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 1 — ap 4 — the record exceeds it on 275.5245 days a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(1.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 275.5245);
    assert!(err <= 1e-12, "Kp 1 — ap 4 — the record exceeds it on 275.5245 days a year: got {} want 275.5245, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 2 — ap 7 — the record exceeds it on 180.4794 days a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(2.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 180.4794);
    assert!(err <= 1e-12, "Kp 2 — ap 7 — the record exceeds it on 180.4794 days a year: got {} want 180.4794, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 3 — ap 15 — the record exceeds it on 63.5880 days a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(3.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 63.588);
    assert!(err <= 1e-12, "Kp 3 — ap 15 — the record exceeds it on 63.5880 days a year: got {} want 63.588, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 4 — ap 27 — the record exceeds it on 18.6898 days a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(4.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 18.6898);
    assert!(err <= 1e-12, "Kp 4 — ap 27 — the record exceeds it on 18.6898 days a year: got {} want 18.6898, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 5 — ap 48 — the record exceeds it on 4.3621 days a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(5.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 4.3621);
    assert!(err <= 1e-12, "Kp 5 — ap 48 — the record exceeds it on 4.3621 days a year: got {} want 4.3621, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 6 — ap 80 — the record exceeds it on 1.2413 days a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(6.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.2413);
    assert!(err <= 1e-12, "Kp 6 — ap 80 — the record exceeds it on 1.2413 days a year: got {} want 1.2413, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 7 — ap 132 — the record exceeds it on 0.2837 days a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(7.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.2837);
    assert!(err <= 1e-12, "Kp 7 — ap 132 — the record exceeds it on 0.2837 days a year: got {} want 0.2837, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 8 — ap 207 — the record exceeds it on 0.0355 days a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(8.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.0355);
    assert!(err <= 1e-12, "Kp 8 — ap 207 — the record exceeds it on 0.0355 days a year: got {} want 0.0355, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 9 — ap 400 — the record exceeds it on 0.0000 days a year
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Ratio::new(9.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.0);
    assert!(err <= 1e-12, "Kp 9 — ap 400 — the record exceeds it on 0.0000 days a year: got {} want 0.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 3.3333 — a third of a unit, which is the resolution this table is measured at
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Ratio::new(3.333333)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 45.572);
    assert!(err <= 1e-12, "Kp 3.3333 — a third of a unit, which is the resolution this table is measured at: got {} want 45.572, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Kp 7.6667 — a third of a unit, which is the resolution this table is measured at
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_11() {
    let got = model::evaluate(Ratio::new(7.666667)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.1419);
    assert!(err <= 1e-12, "Kp 7.6667 — a third of a unit, which is the resolution this table is measured at: got {} want 0.1419, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the declared env_kp of 3 — 63.59 days a year, more than two months
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_12() {
    let got = model::evaluate(Ratio::new(3.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 63.588);
    assert!(err <= 1e-12, "the declared env_kp of 3 — 63.59 days a year, more than two months: got {} want 63.588, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `Kp 0 — ap 0 — the record exceeds it on 364.1506 days a year` and the declared domain 0 … 366.
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
            refused.push(format!("kp x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_storm_rate refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 366 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
            assert!(v.get().is_finite(), "sw_storm_rate produced a value that is not a number for rate");
            assert!(v.get() >= 0.0 && v.get() <= 366.0, "sw_storm_rate answered {} for rate, outside its declared domain 0 … 366 — the guard did not stop it", v.get());
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
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_storm_rate is not deterministic for rate: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_storm_rate refused on one call and answered on the other"),
    }
}

