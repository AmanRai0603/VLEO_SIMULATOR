// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_kp_scenarios`.
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

/// the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94
///
/// Provenance: `independent-derivation`, source `iaga_kp_ap`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.Kp_peak_hotday.get(), 7.9850088183);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94: got {} want 7.9850088183, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Kp_peak_hotday.get(), err);
}

/// ap 12.5, a bin centre. The scale interpolates between Kp 8/3 at ap 12 and Kp 3 at ap 15; the mean offset -0.0976 is exact
///
/// Provenance: `independent-derivation`, source `iaga_kp_ap`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.Kp_mean_nominal.get(), 2.6245833333);
    assert!(err <= 1e-9, "ap 12.5, a bin centre. The scale interpolates between Kp 8/3 at ap 12 and Kp 3 at ap 15; the mean offset -0.0976 is exact: got {} want 2.6245833333, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Kp_mean_nominal.get(), err);
}

/// ap 25, a bin centre. Scale between Kp 11/3 at ap 22 and Kp 4 at ap 27, mean offset -0.1333 exact
///
/// Provenance: `independent-derivation`, source `iaga_kp_ap`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.Kp_mean_hotmean.get(), 3.7333333333);
    assert!(err <= 1e-9, "ap 25, a bin centre. Scale between Kp 11/3 at ap 22 and Kp 4 at ap 27, mean offset -0.1333 exact: got {} want 3.7333333333, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Kp_mean_hotmean.get(), err);
}

/// ap 7.5, a bin centre. Scale between Kp 2 at ap 7 and Kp 7/3 at ap 9, mean offset -0.125 exact
///
/// Provenance: `independent-derivation`, source `iaga_kp_ap`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.Kp_mean_coldmean.get(), 1.9583333333);
    assert!(err <= 1e-9, "ap 7.5, a bin centre. Scale between Kp 2 at ap 7 and Kp 7/3 at ap 9, mean offset -0.125 exact: got {} want 1.9583333333, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Kp_mean_coldmean.get(), err);
}

/// ap 90, a bin centre. The mean offset -0.4487 is exact and the scale is the same interpolation the primary uses — so this and Kp_peak_hotday differ by exactly the two offsets, 2.1956
///
/// Provenance: `independent-derivation`, source `iaga_kp_ap`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.Kp_mean_hotday.get(), 5.7893772894);
    assert!(err <= 1e-9, "ap 90, a bin centre. The mean offset -0.4487 is exact and the scale is the same interpolation the primary uses — so this and Kp_peak_hotday differ by exactly the two offsets, 2.1956: got {} want 5.7893772894, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Kp_mean_hotday.get(), err);
}

/// ap 2.5, the first bin centre. The scale interpolates between Kp 1/3 at ap 2 and Kp 2/3 at ap 3; the mean offset is the one POSITIVE entry in that table, +0.0417
///
/// Provenance: `independent-derivation`, source `iaga_kp_ap`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.Kp_mean_coldday.get(), 0.5416666667);
    assert!(err <= 1e-9, "ap 2.5, the first bin centre. The scale interpolates between Kp 1/3 at ap 2 and Kp 2/3 at ap 3; the mean offset is the one POSITIVE entry in that table, +0.0417: got {} want 0.5416666667, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Kp_mean_coldday.get(), err);
}

/// ap 12.5. Peak offset 1.1144 exact, and this is the bin where that table is not monotone with its neighbours
///
/// Provenance: `independent-derivation`, source `iaga_kp_ap`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.Kp_peak_nominal.get(), 3.8366290019);
    assert!(err <= 1e-9, "ap 12.5. Peak offset 1.1144 exact, and this is the bin where that table is not monotone with its neighbours: got {} want 3.8366290019, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Kp_peak_nominal.get(), err);
}

/// ap 25. Peak offset 1.2 exact. Kp 5.07 is a G1 storm in the worst slot of an ordinary sustained day
///
/// Provenance: `independent-derivation`, source `iaga_kp_ap`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.Kp_peak_hotmean.get(), 5.0666666667);
    assert!(err <= 1e-9, "ap 25. Peak offset 1.2 exact. Kp 5.07 is a G1 storm in the worst slot of an ordinary sustained day: got {} want 5.0666666667, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Kp_peak_hotmean.get(), err);
}

/// ap 7.5. Peak offset 0.8333 exact
///
/// Provenance: `independent-derivation`, source `iaga_kp_ap`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.Kp_peak_coldmean.get(), 2.9166666667);
    assert!(err <= 1e-9, "ap 7.5. Peak offset 0.8333 exact: got {} want 2.9166666667, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Kp_peak_coldmean.get(), err);
}

/// ap 2.5, the first bin centre, where the peak offset is exactly 1.0 and the scale is exactly 0.5 — the one case in this set a reader can check without a calculator
///
/// Provenance: `independent-derivation`, source `iaga_kp_ap`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.Kp_peak_coldday.get(), 1.5);
    assert!(err <= 1e-9, "ap 2.5, the first bin centre, where the peak offset is exactly 1.0 and the scale is exactly 0.5 — the one case in this set a reader can check without a calculator: got {} want 1.5, relative error {} exceeds the declared tolerance 1e-9. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Kp_peak_coldday.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94` and the declared domain 0 … 9.
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
        if let Err(f) = model::evaluate(Ratio::new(12.5 * scale), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)) {
            refused.push(format!("ap_nominal x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(12.5), Ratio::new(25.0 * scale), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)) {
            refused.push(format!("ap_hotmean x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5 * scale), Ratio::new(90.0), Ratio::new(2.5)) {
            refused.push(format!("ap_coldmean x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0 * scale), Ratio::new(2.5)) {
            refused.push(format!("ap_hotday x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5 * scale)) {
            refused.push(format!("ap_coldday x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_kp_scenarios refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 9 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(12.5 * scale), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)) {
            assert!(v.Kp_peak_hotday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_hotday");
            assert!(v.Kp_peak_hotday.get() >= 0.0 && v.Kp_peak_hotday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_hotday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_hotday.get());
            assert!(v.Kp_mean_nominal.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_nominal");
            assert!(v.Kp_mean_nominal.get() >= 0.0 && v.Kp_mean_nominal.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_nominal, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_nominal.get());
            assert!(v.Kp_mean_hotmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_hotmean");
            assert!(v.Kp_mean_hotmean.get() >= 0.0 && v.Kp_mean_hotmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_hotmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_hotmean.get());
            assert!(v.Kp_mean_coldmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_coldmean");
            assert!(v.Kp_mean_coldmean.get() >= 0.0 && v.Kp_mean_coldmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_coldmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_coldmean.get());
            assert!(v.Kp_mean_hotday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_hotday");
            assert!(v.Kp_mean_hotday.get() >= 0.0 && v.Kp_mean_hotday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_hotday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_hotday.get());
            assert!(v.Kp_mean_coldday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_coldday");
            assert!(v.Kp_mean_coldday.get() >= 0.0 && v.Kp_mean_coldday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_coldday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_coldday.get());
            assert!(v.Kp_peak_nominal.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_nominal");
            assert!(v.Kp_peak_nominal.get() >= 0.0 && v.Kp_peak_nominal.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_nominal, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_nominal.get());
            assert!(v.Kp_peak_hotmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_hotmean");
            assert!(v.Kp_peak_hotmean.get() >= 0.0 && v.Kp_peak_hotmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_hotmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_hotmean.get());
            assert!(v.Kp_peak_coldmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_coldmean");
            assert!(v.Kp_peak_coldmean.get() >= 0.0 && v.Kp_peak_coldmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_coldmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_coldmean.get());
            assert!(v.Kp_peak_coldday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_coldday");
            assert!(v.Kp_peak_coldday.get() >= 0.0 && v.Kp_peak_coldday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_coldday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(12.5), Ratio::new(25.0 * scale), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)) {
            assert!(v.Kp_peak_hotday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_hotday");
            assert!(v.Kp_peak_hotday.get() >= 0.0 && v.Kp_peak_hotday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_hotday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_hotday.get());
            assert!(v.Kp_mean_nominal.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_nominal");
            assert!(v.Kp_mean_nominal.get() >= 0.0 && v.Kp_mean_nominal.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_nominal, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_nominal.get());
            assert!(v.Kp_mean_hotmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_hotmean");
            assert!(v.Kp_mean_hotmean.get() >= 0.0 && v.Kp_mean_hotmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_hotmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_hotmean.get());
            assert!(v.Kp_mean_coldmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_coldmean");
            assert!(v.Kp_mean_coldmean.get() >= 0.0 && v.Kp_mean_coldmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_coldmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_coldmean.get());
            assert!(v.Kp_mean_hotday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_hotday");
            assert!(v.Kp_mean_hotday.get() >= 0.0 && v.Kp_mean_hotday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_hotday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_hotday.get());
            assert!(v.Kp_mean_coldday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_coldday");
            assert!(v.Kp_mean_coldday.get() >= 0.0 && v.Kp_mean_coldday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_coldday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_coldday.get());
            assert!(v.Kp_peak_nominal.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_nominal");
            assert!(v.Kp_peak_nominal.get() >= 0.0 && v.Kp_peak_nominal.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_nominal, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_nominal.get());
            assert!(v.Kp_peak_hotmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_hotmean");
            assert!(v.Kp_peak_hotmean.get() >= 0.0 && v.Kp_peak_hotmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_hotmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_hotmean.get());
            assert!(v.Kp_peak_coldmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_coldmean");
            assert!(v.Kp_peak_coldmean.get() >= 0.0 && v.Kp_peak_coldmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_coldmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_coldmean.get());
            assert!(v.Kp_peak_coldday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_coldday");
            assert!(v.Kp_peak_coldday.get() >= 0.0 && v.Kp_peak_coldday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_coldday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5 * scale), Ratio::new(90.0), Ratio::new(2.5)) {
            assert!(v.Kp_peak_hotday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_hotday");
            assert!(v.Kp_peak_hotday.get() >= 0.0 && v.Kp_peak_hotday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_hotday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_hotday.get());
            assert!(v.Kp_mean_nominal.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_nominal");
            assert!(v.Kp_mean_nominal.get() >= 0.0 && v.Kp_mean_nominal.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_nominal, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_nominal.get());
            assert!(v.Kp_mean_hotmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_hotmean");
            assert!(v.Kp_mean_hotmean.get() >= 0.0 && v.Kp_mean_hotmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_hotmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_hotmean.get());
            assert!(v.Kp_mean_coldmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_coldmean");
            assert!(v.Kp_mean_coldmean.get() >= 0.0 && v.Kp_mean_coldmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_coldmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_coldmean.get());
            assert!(v.Kp_mean_hotday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_hotday");
            assert!(v.Kp_mean_hotday.get() >= 0.0 && v.Kp_mean_hotday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_hotday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_hotday.get());
            assert!(v.Kp_mean_coldday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_coldday");
            assert!(v.Kp_mean_coldday.get() >= 0.0 && v.Kp_mean_coldday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_coldday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_coldday.get());
            assert!(v.Kp_peak_nominal.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_nominal");
            assert!(v.Kp_peak_nominal.get() >= 0.0 && v.Kp_peak_nominal.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_nominal, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_nominal.get());
            assert!(v.Kp_peak_hotmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_hotmean");
            assert!(v.Kp_peak_hotmean.get() >= 0.0 && v.Kp_peak_hotmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_hotmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_hotmean.get());
            assert!(v.Kp_peak_coldmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_coldmean");
            assert!(v.Kp_peak_coldmean.get() >= 0.0 && v.Kp_peak_coldmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_coldmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_coldmean.get());
            assert!(v.Kp_peak_coldday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_coldday");
            assert!(v.Kp_peak_coldday.get() >= 0.0 && v.Kp_peak_coldday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_coldday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0 * scale), Ratio::new(2.5)) {
            assert!(v.Kp_peak_hotday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_hotday");
            assert!(v.Kp_peak_hotday.get() >= 0.0 && v.Kp_peak_hotday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_hotday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_hotday.get());
            assert!(v.Kp_mean_nominal.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_nominal");
            assert!(v.Kp_mean_nominal.get() >= 0.0 && v.Kp_mean_nominal.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_nominal, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_nominal.get());
            assert!(v.Kp_mean_hotmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_hotmean");
            assert!(v.Kp_mean_hotmean.get() >= 0.0 && v.Kp_mean_hotmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_hotmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_hotmean.get());
            assert!(v.Kp_mean_coldmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_coldmean");
            assert!(v.Kp_mean_coldmean.get() >= 0.0 && v.Kp_mean_coldmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_coldmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_coldmean.get());
            assert!(v.Kp_mean_hotday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_hotday");
            assert!(v.Kp_mean_hotday.get() >= 0.0 && v.Kp_mean_hotday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_hotday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_hotday.get());
            assert!(v.Kp_mean_coldday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_coldday");
            assert!(v.Kp_mean_coldday.get() >= 0.0 && v.Kp_mean_coldday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_coldday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_coldday.get());
            assert!(v.Kp_peak_nominal.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_nominal");
            assert!(v.Kp_peak_nominal.get() >= 0.0 && v.Kp_peak_nominal.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_nominal, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_nominal.get());
            assert!(v.Kp_peak_hotmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_hotmean");
            assert!(v.Kp_peak_hotmean.get() >= 0.0 && v.Kp_peak_hotmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_hotmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_hotmean.get());
            assert!(v.Kp_peak_coldmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_coldmean");
            assert!(v.Kp_peak_coldmean.get() >= 0.0 && v.Kp_peak_coldmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_coldmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_coldmean.get());
            assert!(v.Kp_peak_coldday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_coldday");
            assert!(v.Kp_peak_coldday.get() >= 0.0 && v.Kp_peak_coldday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_coldday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5 * scale)) {
            assert!(v.Kp_peak_hotday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_hotday");
            assert!(v.Kp_peak_hotday.get() >= 0.0 && v.Kp_peak_hotday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_hotday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_hotday.get());
            assert!(v.Kp_mean_nominal.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_nominal");
            assert!(v.Kp_mean_nominal.get() >= 0.0 && v.Kp_mean_nominal.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_nominal, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_nominal.get());
            assert!(v.Kp_mean_hotmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_hotmean");
            assert!(v.Kp_mean_hotmean.get() >= 0.0 && v.Kp_mean_hotmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_hotmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_hotmean.get());
            assert!(v.Kp_mean_coldmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_coldmean");
            assert!(v.Kp_mean_coldmean.get() >= 0.0 && v.Kp_mean_coldmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_coldmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_coldmean.get());
            assert!(v.Kp_mean_hotday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_hotday");
            assert!(v.Kp_mean_hotday.get() >= 0.0 && v.Kp_mean_hotday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_hotday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_hotday.get());
            assert!(v.Kp_mean_coldday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_mean_coldday");
            assert!(v.Kp_mean_coldday.get() >= 0.0 && v.Kp_mean_coldday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_mean_coldday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_mean_coldday.get());
            assert!(v.Kp_peak_nominal.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_nominal");
            assert!(v.Kp_peak_nominal.get() >= 0.0 && v.Kp_peak_nominal.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_nominal, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_nominal.get());
            assert!(v.Kp_peak_hotmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_hotmean");
            assert!(v.Kp_peak_hotmean.get() >= 0.0 && v.Kp_peak_hotmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_hotmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_hotmean.get());
            assert!(v.Kp_peak_coldmean.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_coldmean");
            assert!(v.Kp_peak_coldmean.get() >= 0.0 && v.Kp_peak_coldmean.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_coldmean, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_coldmean.get());
            assert!(v.Kp_peak_coldday.get().is_finite(), "sw_kp_scenarios produced a value that is not a number for Kp_peak_coldday");
            assert!(v.Kp_peak_coldday.get() >= 0.0 && v.Kp_peak_coldday.get() <= 9.0, "sw_kp_scenarios answered {} for Kp_peak_coldday, outside its declared domain 0 … 9 — the guard did not stop it", v.Kp_peak_coldday.get());
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
    let a = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5));
    let b = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.Kp_peak_hotday.get().to_bits() == y.Kp_peak_hotday.get().to_bits(), "sw_kp_scenarios is not deterministic for Kp_peak_hotday: {} then {}", x.Kp_peak_hotday.get(), y.Kp_peak_hotday.get());
            assert!(x.Kp_mean_nominal.get().to_bits() == y.Kp_mean_nominal.get().to_bits(), "sw_kp_scenarios is not deterministic for Kp_mean_nominal: {} then {}", x.Kp_mean_nominal.get(), y.Kp_mean_nominal.get());
            assert!(x.Kp_mean_hotmean.get().to_bits() == y.Kp_mean_hotmean.get().to_bits(), "sw_kp_scenarios is not deterministic for Kp_mean_hotmean: {} then {}", x.Kp_mean_hotmean.get(), y.Kp_mean_hotmean.get());
            assert!(x.Kp_mean_coldmean.get().to_bits() == y.Kp_mean_coldmean.get().to_bits(), "sw_kp_scenarios is not deterministic for Kp_mean_coldmean: {} then {}", x.Kp_mean_coldmean.get(), y.Kp_mean_coldmean.get());
            assert!(x.Kp_mean_hotday.get().to_bits() == y.Kp_mean_hotday.get().to_bits(), "sw_kp_scenarios is not deterministic for Kp_mean_hotday: {} then {}", x.Kp_mean_hotday.get(), y.Kp_mean_hotday.get());
            assert!(x.Kp_mean_coldday.get().to_bits() == y.Kp_mean_coldday.get().to_bits(), "sw_kp_scenarios is not deterministic for Kp_mean_coldday: {} then {}", x.Kp_mean_coldday.get(), y.Kp_mean_coldday.get());
            assert!(x.Kp_peak_nominal.get().to_bits() == y.Kp_peak_nominal.get().to_bits(), "sw_kp_scenarios is not deterministic for Kp_peak_nominal: {} then {}", x.Kp_peak_nominal.get(), y.Kp_peak_nominal.get());
            assert!(x.Kp_peak_hotmean.get().to_bits() == y.Kp_peak_hotmean.get().to_bits(), "sw_kp_scenarios is not deterministic for Kp_peak_hotmean: {} then {}", x.Kp_peak_hotmean.get(), y.Kp_peak_hotmean.get());
            assert!(x.Kp_peak_coldmean.get().to_bits() == y.Kp_peak_coldmean.get().to_bits(), "sw_kp_scenarios is not deterministic for Kp_peak_coldmean: {} then {}", x.Kp_peak_coldmean.get(), y.Kp_peak_coldmean.get());
            assert!(x.Kp_peak_coldday.get().to_bits() == y.Kp_peak_coldday.get().to_bits(), "sw_kp_scenarios is not deterministic for Kp_peak_coldday: {} then {}", x.Kp_peak_coldday.get(), y.Kp_peak_coldday.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_kp_scenarios refused on one call and answered on the other"),
    }
}

/// The prior implementation, over the grid it was exported on.
///
/// Migrated from `prf_ap2kp.m (01_kernel/sw_study/02_drivers) with prf_density.m's designWindow_ supplying the five Ap`. These numbers are a second opinion and never
/// an expected value: an implementation cannot supply its own, and the
/// prior tool is an implementation. A disagreement is a finding about
/// one of the two.
#[test]
fn agrees_with_the_prior_implementation() {
    const GRID: &str = include_str!("parity.csv");
    const TOL: f64 = 0.0001;

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
    let want = ["ap_nominal", "ap_hotmean", "ap_coldmean", "ap_hotday", "ap_coldday"];
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
        let got = match model::evaluate(Ratio::new(row[col[0]]), Ratio::new(row[col[1]]), Ratio::new(row[col[2]]), Ratio::new(row[col[3]]), Ratio::new(row[col[4]])) {
            Ok(v) => v.Kp_peak_hotday.get(),
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
        "sw_kp_scenarios: {} of {rows} grid row(s) disagree with the prior implementation `prf_ap2kp.m (01_kernel/sw_study/02_drivers) with prf_density.m's designWindow_ supplying the five Ap` beyond {TOL} (worst {worst}).\n{}\nThis is a finding about one of the two implementations, not a build failure and not proof this one is wrong — both classes have been found before. Take it to the node owner. Do not widen parity_tolerance and do not edit parity.csv to agree.",
        findings.len(),
        findings.join("\n")
    );
}

