// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `env_exospheric_temperature`.
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

/// solar minimum, quiet
///
/// Provenance: `published-source`, source `jacchia1971`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(70.0), Ratio::new(70.0), Ratio::new(1.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 633.9);
    assert!(err <= 0.002, "solar minimum, quiet: got {} want 633.9, relative error {} exceeds the declared tolerance 0.002. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// moderate activity
///
/// Provenance: `published-source`, source `jacchia1971`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(150.0), Ratio::new(150.0), Ratio::new(3.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 949.6);
    assert!(err <= 0.002, "moderate activity: got {} want 949.6, relative error {} exceeds the declared tolerance 0.002. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// solar maximum, storm
///
/// Provenance: `published-source`, source `jacchia1971`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(250.0), Ratio::new(250.0), Ratio::new(7.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1417.9);
    assert!(err <= 0.002, "solar maximum, storm: got {} want 1417.9, relative error {} exceeds the declared tolerance 0.002. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the intercept, at the lowest activity the guard admits
///
/// Provenance: `independent-derivation`, source `jacchia1971`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(10.0), Ratio::new(10.0), Ratio::new(0.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 411.43);
    assert!(err <= 1e-9, "the intercept, at the lowest activity the guard admits: got {} want 411.43, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the fast term, flux 50 above its own 81-day mean
///
/// Provenance: `independent-derivation`, source `jacchia1971`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(200.0), Ratio::new(150.0), Ratio::new(3.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1014.6025661077);
    assert!(err <= 1e-9, "the fast term, flux 50 above its own 81-day mean: got {} want 1014.6025661077, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the fast term, flux 50 below its own 81-day mean
///
/// Provenance: `independent-derivation`, source `jacchia1971`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(100.0), Ratio::new(150.0), Ratio::new(3.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 884.6025661077);
    assert!(err <= 1e-9, "the fast term, flux 50 below its own 81-day mean: got {} want 884.6025661077, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the storm tail at Kp 7, where the exponential is 32.9 K of 228.9
///
/// Provenance: `independent-derivation`, source `jacchia1971`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(150.0), Ratio::new(150.0), Ratio::new(7.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1093.8989947529);
    assert!(err <= 1e-9, "the storm tail at Kp 7, where the exponential is 32.9 K of 228.9: got {} want 1093.8989947529, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the storm tail at Kp 8, one step up and 84.5 K rather than 28
///
/// Provenance: `independent-derivation`, source `jacchia1971`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(150.0), Ratio::new(150.0), Ratio::new(8.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1178.4287396113);
    assert!(err <= 1e-9, "the storm tail at Kp 8, one step up and 84.5 K rather than 28: got {} want 1178.4287396113, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `solar minimum, quiet` and the declared domain 400 … 2500.
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
        if let Err(f) = model::evaluate(Ratio::new(70.0 * scale), Ratio::new(70.0), Ratio::new(1.0)) {
            refused.push(format!("f107 x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(70.0), Ratio::new(70.0 * scale), Ratio::new(1.0)) {
            refused.push(format!("f107a x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(70.0), Ratio::new(70.0), Ratio::new(1.0 * scale)) {
            refused.push(format!("kp x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "env_exospheric_temperature refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 400 … 2500 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(70.0 * scale), Ratio::new(70.0), Ratio::new(1.0)) {
            assert!(v.get().is_finite(), "env_exospheric_temperature produced a value that is not a number for T_inf");
            assert!(v.get() >= 400.0 && v.get() <= 2500.0, "env_exospheric_temperature answered {} for T_inf, outside its declared domain 400 … 2500 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(70.0), Ratio::new(70.0 * scale), Ratio::new(1.0)) {
            assert!(v.get().is_finite(), "env_exospheric_temperature produced a value that is not a number for T_inf");
            assert!(v.get() >= 400.0 && v.get() <= 2500.0, "env_exospheric_temperature answered {} for T_inf, outside its declared domain 400 … 2500 — the guard did not stop it", v.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(70.0), Ratio::new(70.0), Ratio::new(1.0 * scale)) {
            assert!(v.get().is_finite(), "env_exospheric_temperature produced a value that is not a number for T_inf");
            assert!(v.get() >= 400.0 && v.get() <= 2500.0, "env_exospheric_temperature answered {} for T_inf, outside its declared domain 400 … 2500 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(70.0), Ratio::new(70.0), Ratio::new(1.0));
    let b = model::evaluate(Ratio::new(70.0), Ratio::new(70.0), Ratio::new(1.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "env_exospheric_temperature is not deterministic for T_inf: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("env_exospheric_temperature refused on one call and answered on the other"),
    }
}

