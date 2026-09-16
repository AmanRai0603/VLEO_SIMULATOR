// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `l3_solar_interface`.
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

/// the sustained hot F10.7 crosses unchanged, from sw_f107_design_long
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.F107_hotmean.get(), 175.5520201913);
    assert!(err <= 1e-12, "the sustained hot F10.7 crosses unchanged, from sw_f107_design_long: got {} want 175.5520201913, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.F107_hotmean.get(), err);
}

/// the nominal F10.7 crosses unchanged, from sw_central_expectation
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.F107_nominal.get(), 158.3304112313);
    assert!(err <= 1e-12, "the nominal F10.7 crosses unchanged, from sw_central_expectation: got {} want 158.3304112313, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.F107_nominal.get(), err);
}

/// the sustained cold F10.7 crosses unchanged, from sw_f107_cold_long
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.F107_coldmean.get(), 141.1088022713);
    assert!(err <= 1e-12, "the sustained cold F10.7 crosses unchanged, from sw_f107_cold_long: got {} want 141.1088022713, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.F107_coldmean.get(), err);
}

/// the hot single day crosses unchanged, from sw_f107_design_short
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.F107_hotday.get(), 209.7835016728);
    assert!(err <= 1e-12, "the hot single day crosses unchanged, from sw_f107_design_short: got {} want 209.7835016728, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.F107_hotday.get(), err);
}

/// the cold single day crosses unchanged, from sw_f107_cold_short
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.F107_coldday.get(), 109.8365800491);
    assert!(err <= 1e-12, "the cold single day crosses unchanged, from sw_f107_cold_short: got {} want 109.8365800491, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.F107_coldday.get(), err);
}

/// the nominal scenario's 81-day mean is its own value — a *mean scenario IS its mean
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.F107bar_nominal.get(), 158.3304112313);
    assert!(err <= 1e-12, "the nominal scenario's 81-day mean is its own value — a *mean scenario IS its mean: got {} want 158.3304112313, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.F107bar_nominal.get(), err);
}

/// the hot sustained scenario's 81-day mean is its own value
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.F107bar_hotmean.get(), 175.5520201913);
    assert!(err <= 1e-12, "the hot sustained scenario's 81-day mean is its own value: got {} want 175.5520201913, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.F107bar_hotmean.get(), err);
}

/// the cold sustained scenario's 81-day mean is its own value
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.F107bar_coldmean.get(), 141.1088022713);
    assert!(err <= 1e-12, "the cold sustained scenario's 81-day mean is its own value: got {} want 141.1088022713, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.F107bar_coldmean.get(), err);
}

/// THE HOT DAY RIDES ON THE HOT MEAN — its 81-day mean is 175.55, not its own 209.78. The one member that is not a copy of its namesake input
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.F107bar_hotday.get(), 175.5520201913);
    assert!(err <= 1e-12, "THE HOT DAY RIDES ON THE HOT MEAN — its 81-day mean is 175.55, not its own 209.78. The one member that is not a copy of its namesake input: got {} want 175.5520201913, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.F107bar_hotday.get(), err);
}

/// THE COLD DAY RIDES ON THE COLD MEAN — its 81-day mean is 141.11, not its own 109.84
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.F107bar_coldday.get(), 141.1088022713);
    assert!(err <= 1e-12, "THE COLD DAY RIDES ON THE COLD MEAN — its 81-day mean is 141.11, not its own 109.84: got {} want 141.1088022713, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.F107bar_coldday.get(), err);
}

/// the nominal Ap crosses unchanged, from sw_ap_central_expectation
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.Ap_nominal.get(), 22.095389);
    assert!(err <= 1e-12, "the nominal Ap crosses unchanged, from sw_ap_central_expectation: got {} want 22.095389, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Ap_nominal.get(), err);
}

/// the sustained disturbed Ap crosses unchanged, from sw_ap_design_long
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_11() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.Ap_hotmean.get(), 26.69530964);
    assert!(err <= 1e-12, "the sustained disturbed Ap crosses unchanged, from sw_ap_design_long: got {} want 26.69530964, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Ap_hotmean.get(), err);
}

/// the sustained quiet Ap crosses unchanged, from sw_ap_cold_long
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_12() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.Ap_coldmean.get(), 17.49546836);
    assert!(err <= 1e-12, "the sustained quiet Ap crosses unchanged, from sw_ap_cold_long: got {} want 17.49546836, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Ap_coldmean.get(), err);
}

/// the most disturbed single day crosses unchanged, from sw_ap_design_short
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_13() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.Ap_hotday.get(), 41.6971614919);
    assert!(err <= 1e-12, "the most disturbed single day crosses unchanged, from sw_ap_design_short: got {} want 41.6971614919, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Ap_hotday.get(), err);
}

/// the quietest single day crosses unchanged, from sw_ap_cold_short — the member closest to a declared bound, seven units clear of zero
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_14() {
    let got = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)).expect("the fixture case must not be refused");
    let err = relative_error(got.Ap_coldday.get(), 6.9065794711);
    assert!(err <= 1e-12, "the quietest single day crosses unchanged, from sw_ap_cold_short — the member closest to a declared bound, seven units clear of zero: got {} want 6.9065794711, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.Ap_coldday.get(), err);
}

// ---- properties, generated from the declared domain ---------------------
//
// The fixture above checks one point. A wrong constant moves that point and
// is caught there; a wrong shape can pass one point and be wrong everywhere
// else. These ask the part of that question that is the same for every
// node, so it is derived rather than written.

/// One per cent either side of the known-good point, this node still answers.
///
/// Derived from `the sustained hot F10.7 crosses unchanged, from sw_f107_design_long` and the declared domain 60 … 400.
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
        if let Err(f) = model::evaluate(Ratio::new(158.3304112313 * scale), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            refused.push(format!("f107_centre x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913 * scale), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            refused.push(format!("f107_hot_long x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713 * scale), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            refused.push(format!("f107_cold_long x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728 * scale), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            refused.push(format!("f107_hot_day x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491 * scale), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            refused.push(format!("f107_cold_day x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389 * scale), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            refused.push(format!("ap_centre x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964 * scale), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            refused.push(format!("ap_hot_long x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836 * scale), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            refused.push(format!("ap_cold_long x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919 * scale), Ratio::new(6.9065794711)) {
            refused.push(format!("ap_hot_day x{scale} -> {f}"));
        }
    }
    for scale in [0.99_f64, 1.01] {
        if let Err(f) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711 * scale)) {
            refused.push(format!("ap_cold_day x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "l3_solar_interface refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 60 … 400 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(158.3304112313 * scale), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            assert!(v.F107_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotmean");
            assert!(v.F107_hotmean.get() >= 60.0 && v.F107_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotmean.get());
            assert!(v.F107_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_nominal");
            assert!(v.F107_nominal.get() >= 60.0 && v.F107_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_nominal.get());
            assert!(v.F107_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldmean");
            assert!(v.F107_coldmean.get() >= 60.0 && v.F107_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldmean.get());
            assert!(v.F107_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotday");
            assert!(v.F107_hotday.get() >= 60.0 && v.F107_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotday.get());
            assert!(v.F107_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldday");
            assert!(v.F107_coldday.get() >= 60.0 && v.F107_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldday.get());
            assert!(v.F107bar_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_nominal");
            assert!(v.F107bar_nominal.get() >= 60.0 && v.F107bar_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107bar_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_nominal.get());
            assert!(v.F107bar_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotmean");
            assert!(v.F107bar_hotmean.get() >= 60.0 && v.F107bar_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotmean.get());
            assert!(v.F107bar_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldmean");
            assert!(v.F107bar_coldmean.get() >= 60.0 && v.F107bar_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldmean.get());
            assert!(v.F107bar_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotday");
            assert!(v.F107bar_hotday.get() >= 60.0 && v.F107bar_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotday.get());
            assert!(v.F107bar_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldday");
            assert!(v.F107bar_coldday.get() >= 60.0 && v.F107bar_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldday.get());
            assert!(v.Ap_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_nominal");
            assert!(v.Ap_nominal.get() >= 0.0 && v.Ap_nominal.get() <= 400.0, "l3_solar_interface answered {} for Ap_nominal, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_nominal.get());
            assert!(v.Ap_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotmean");
            assert!(v.Ap_hotmean.get() >= 0.0 && v.Ap_hotmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotmean.get());
            assert!(v.Ap_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldmean");
            assert!(v.Ap_coldmean.get() >= 0.0 && v.Ap_coldmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldmean.get());
            assert!(v.Ap_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotday");
            assert!(v.Ap_hotday.get() >= 0.0 && v.Ap_hotday.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotday.get());
            assert!(v.Ap_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldday");
            assert!(v.Ap_coldday.get() >= 0.0 && v.Ap_coldday.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913 * scale), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            assert!(v.F107_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotmean");
            assert!(v.F107_hotmean.get() >= 60.0 && v.F107_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotmean.get());
            assert!(v.F107_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_nominal");
            assert!(v.F107_nominal.get() >= 60.0 && v.F107_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_nominal.get());
            assert!(v.F107_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldmean");
            assert!(v.F107_coldmean.get() >= 60.0 && v.F107_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldmean.get());
            assert!(v.F107_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotday");
            assert!(v.F107_hotday.get() >= 60.0 && v.F107_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotday.get());
            assert!(v.F107_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldday");
            assert!(v.F107_coldday.get() >= 60.0 && v.F107_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldday.get());
            assert!(v.F107bar_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_nominal");
            assert!(v.F107bar_nominal.get() >= 60.0 && v.F107bar_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107bar_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_nominal.get());
            assert!(v.F107bar_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotmean");
            assert!(v.F107bar_hotmean.get() >= 60.0 && v.F107bar_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotmean.get());
            assert!(v.F107bar_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldmean");
            assert!(v.F107bar_coldmean.get() >= 60.0 && v.F107bar_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldmean.get());
            assert!(v.F107bar_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotday");
            assert!(v.F107bar_hotday.get() >= 60.0 && v.F107bar_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotday.get());
            assert!(v.F107bar_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldday");
            assert!(v.F107bar_coldday.get() >= 60.0 && v.F107bar_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldday.get());
            assert!(v.Ap_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_nominal");
            assert!(v.Ap_nominal.get() >= 0.0 && v.Ap_nominal.get() <= 400.0, "l3_solar_interface answered {} for Ap_nominal, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_nominal.get());
            assert!(v.Ap_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotmean");
            assert!(v.Ap_hotmean.get() >= 0.0 && v.Ap_hotmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotmean.get());
            assert!(v.Ap_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldmean");
            assert!(v.Ap_coldmean.get() >= 0.0 && v.Ap_coldmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldmean.get());
            assert!(v.Ap_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotday");
            assert!(v.Ap_hotday.get() >= 0.0 && v.Ap_hotday.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotday.get());
            assert!(v.Ap_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldday");
            assert!(v.Ap_coldday.get() >= 0.0 && v.Ap_coldday.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713 * scale), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            assert!(v.F107_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotmean");
            assert!(v.F107_hotmean.get() >= 60.0 && v.F107_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotmean.get());
            assert!(v.F107_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_nominal");
            assert!(v.F107_nominal.get() >= 60.0 && v.F107_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_nominal.get());
            assert!(v.F107_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldmean");
            assert!(v.F107_coldmean.get() >= 60.0 && v.F107_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldmean.get());
            assert!(v.F107_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotday");
            assert!(v.F107_hotday.get() >= 60.0 && v.F107_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotday.get());
            assert!(v.F107_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldday");
            assert!(v.F107_coldday.get() >= 60.0 && v.F107_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldday.get());
            assert!(v.F107bar_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_nominal");
            assert!(v.F107bar_nominal.get() >= 60.0 && v.F107bar_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107bar_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_nominal.get());
            assert!(v.F107bar_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotmean");
            assert!(v.F107bar_hotmean.get() >= 60.0 && v.F107bar_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotmean.get());
            assert!(v.F107bar_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldmean");
            assert!(v.F107bar_coldmean.get() >= 60.0 && v.F107bar_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldmean.get());
            assert!(v.F107bar_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotday");
            assert!(v.F107bar_hotday.get() >= 60.0 && v.F107bar_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotday.get());
            assert!(v.F107bar_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldday");
            assert!(v.F107bar_coldday.get() >= 60.0 && v.F107bar_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldday.get());
            assert!(v.Ap_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_nominal");
            assert!(v.Ap_nominal.get() >= 0.0 && v.Ap_nominal.get() <= 400.0, "l3_solar_interface answered {} for Ap_nominal, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_nominal.get());
            assert!(v.Ap_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotmean");
            assert!(v.Ap_hotmean.get() >= 0.0 && v.Ap_hotmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotmean.get());
            assert!(v.Ap_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldmean");
            assert!(v.Ap_coldmean.get() >= 0.0 && v.Ap_coldmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldmean.get());
            assert!(v.Ap_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotday");
            assert!(v.Ap_hotday.get() >= 0.0 && v.Ap_hotday.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotday.get());
            assert!(v.Ap_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldday");
            assert!(v.Ap_coldday.get() >= 0.0 && v.Ap_coldday.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728 * scale), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            assert!(v.F107_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotmean");
            assert!(v.F107_hotmean.get() >= 60.0 && v.F107_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotmean.get());
            assert!(v.F107_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_nominal");
            assert!(v.F107_nominal.get() >= 60.0 && v.F107_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_nominal.get());
            assert!(v.F107_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldmean");
            assert!(v.F107_coldmean.get() >= 60.0 && v.F107_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldmean.get());
            assert!(v.F107_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotday");
            assert!(v.F107_hotday.get() >= 60.0 && v.F107_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotday.get());
            assert!(v.F107_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldday");
            assert!(v.F107_coldday.get() >= 60.0 && v.F107_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldday.get());
            assert!(v.F107bar_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_nominal");
            assert!(v.F107bar_nominal.get() >= 60.0 && v.F107bar_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107bar_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_nominal.get());
            assert!(v.F107bar_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotmean");
            assert!(v.F107bar_hotmean.get() >= 60.0 && v.F107bar_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotmean.get());
            assert!(v.F107bar_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldmean");
            assert!(v.F107bar_coldmean.get() >= 60.0 && v.F107bar_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldmean.get());
            assert!(v.F107bar_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotday");
            assert!(v.F107bar_hotday.get() >= 60.0 && v.F107bar_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotday.get());
            assert!(v.F107bar_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldday");
            assert!(v.F107bar_coldday.get() >= 60.0 && v.F107bar_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldday.get());
            assert!(v.Ap_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_nominal");
            assert!(v.Ap_nominal.get() >= 0.0 && v.Ap_nominal.get() <= 400.0, "l3_solar_interface answered {} for Ap_nominal, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_nominal.get());
            assert!(v.Ap_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotmean");
            assert!(v.Ap_hotmean.get() >= 0.0 && v.Ap_hotmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotmean.get());
            assert!(v.Ap_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldmean");
            assert!(v.Ap_coldmean.get() >= 0.0 && v.Ap_coldmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldmean.get());
            assert!(v.Ap_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotday");
            assert!(v.Ap_hotday.get() >= 0.0 && v.Ap_hotday.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotday.get());
            assert!(v.Ap_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldday");
            assert!(v.Ap_coldday.get() >= 0.0 && v.Ap_coldday.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491 * scale), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            assert!(v.F107_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotmean");
            assert!(v.F107_hotmean.get() >= 60.0 && v.F107_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotmean.get());
            assert!(v.F107_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_nominal");
            assert!(v.F107_nominal.get() >= 60.0 && v.F107_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_nominal.get());
            assert!(v.F107_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldmean");
            assert!(v.F107_coldmean.get() >= 60.0 && v.F107_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldmean.get());
            assert!(v.F107_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotday");
            assert!(v.F107_hotday.get() >= 60.0 && v.F107_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotday.get());
            assert!(v.F107_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldday");
            assert!(v.F107_coldday.get() >= 60.0 && v.F107_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldday.get());
            assert!(v.F107bar_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_nominal");
            assert!(v.F107bar_nominal.get() >= 60.0 && v.F107bar_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107bar_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_nominal.get());
            assert!(v.F107bar_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotmean");
            assert!(v.F107bar_hotmean.get() >= 60.0 && v.F107bar_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotmean.get());
            assert!(v.F107bar_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldmean");
            assert!(v.F107bar_coldmean.get() >= 60.0 && v.F107bar_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldmean.get());
            assert!(v.F107bar_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotday");
            assert!(v.F107bar_hotday.get() >= 60.0 && v.F107bar_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotday.get());
            assert!(v.F107bar_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldday");
            assert!(v.F107bar_coldday.get() >= 60.0 && v.F107bar_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldday.get());
            assert!(v.Ap_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_nominal");
            assert!(v.Ap_nominal.get() >= 0.0 && v.Ap_nominal.get() <= 400.0, "l3_solar_interface answered {} for Ap_nominal, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_nominal.get());
            assert!(v.Ap_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotmean");
            assert!(v.Ap_hotmean.get() >= 0.0 && v.Ap_hotmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotmean.get());
            assert!(v.Ap_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldmean");
            assert!(v.Ap_coldmean.get() >= 0.0 && v.Ap_coldmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldmean.get());
            assert!(v.Ap_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotday");
            assert!(v.Ap_hotday.get() >= 0.0 && v.Ap_hotday.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotday.get());
            assert!(v.Ap_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldday");
            assert!(v.Ap_coldday.get() >= 0.0 && v.Ap_coldday.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389 * scale), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            assert!(v.F107_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotmean");
            assert!(v.F107_hotmean.get() >= 60.0 && v.F107_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotmean.get());
            assert!(v.F107_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_nominal");
            assert!(v.F107_nominal.get() >= 60.0 && v.F107_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_nominal.get());
            assert!(v.F107_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldmean");
            assert!(v.F107_coldmean.get() >= 60.0 && v.F107_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldmean.get());
            assert!(v.F107_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotday");
            assert!(v.F107_hotday.get() >= 60.0 && v.F107_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotday.get());
            assert!(v.F107_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldday");
            assert!(v.F107_coldday.get() >= 60.0 && v.F107_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldday.get());
            assert!(v.F107bar_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_nominal");
            assert!(v.F107bar_nominal.get() >= 60.0 && v.F107bar_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107bar_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_nominal.get());
            assert!(v.F107bar_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotmean");
            assert!(v.F107bar_hotmean.get() >= 60.0 && v.F107bar_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotmean.get());
            assert!(v.F107bar_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldmean");
            assert!(v.F107bar_coldmean.get() >= 60.0 && v.F107bar_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldmean.get());
            assert!(v.F107bar_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotday");
            assert!(v.F107bar_hotday.get() >= 60.0 && v.F107bar_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotday.get());
            assert!(v.F107bar_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldday");
            assert!(v.F107bar_coldday.get() >= 60.0 && v.F107bar_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldday.get());
            assert!(v.Ap_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_nominal");
            assert!(v.Ap_nominal.get() >= 0.0 && v.Ap_nominal.get() <= 400.0, "l3_solar_interface answered {} for Ap_nominal, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_nominal.get());
            assert!(v.Ap_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotmean");
            assert!(v.Ap_hotmean.get() >= 0.0 && v.Ap_hotmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotmean.get());
            assert!(v.Ap_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldmean");
            assert!(v.Ap_coldmean.get() >= 0.0 && v.Ap_coldmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldmean.get());
            assert!(v.Ap_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotday");
            assert!(v.Ap_hotday.get() >= 0.0 && v.Ap_hotday.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotday.get());
            assert!(v.Ap_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldday");
            assert!(v.Ap_coldday.get() >= 0.0 && v.Ap_coldday.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964 * scale), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            assert!(v.F107_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotmean");
            assert!(v.F107_hotmean.get() >= 60.0 && v.F107_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotmean.get());
            assert!(v.F107_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_nominal");
            assert!(v.F107_nominal.get() >= 60.0 && v.F107_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_nominal.get());
            assert!(v.F107_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldmean");
            assert!(v.F107_coldmean.get() >= 60.0 && v.F107_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldmean.get());
            assert!(v.F107_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotday");
            assert!(v.F107_hotday.get() >= 60.0 && v.F107_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotday.get());
            assert!(v.F107_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldday");
            assert!(v.F107_coldday.get() >= 60.0 && v.F107_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldday.get());
            assert!(v.F107bar_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_nominal");
            assert!(v.F107bar_nominal.get() >= 60.0 && v.F107bar_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107bar_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_nominal.get());
            assert!(v.F107bar_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotmean");
            assert!(v.F107bar_hotmean.get() >= 60.0 && v.F107bar_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotmean.get());
            assert!(v.F107bar_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldmean");
            assert!(v.F107bar_coldmean.get() >= 60.0 && v.F107bar_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldmean.get());
            assert!(v.F107bar_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotday");
            assert!(v.F107bar_hotday.get() >= 60.0 && v.F107bar_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotday.get());
            assert!(v.F107bar_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldday");
            assert!(v.F107bar_coldday.get() >= 60.0 && v.F107bar_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldday.get());
            assert!(v.Ap_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_nominal");
            assert!(v.Ap_nominal.get() >= 0.0 && v.Ap_nominal.get() <= 400.0, "l3_solar_interface answered {} for Ap_nominal, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_nominal.get());
            assert!(v.Ap_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotmean");
            assert!(v.Ap_hotmean.get() >= 0.0 && v.Ap_hotmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotmean.get());
            assert!(v.Ap_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldmean");
            assert!(v.Ap_coldmean.get() >= 0.0 && v.Ap_coldmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldmean.get());
            assert!(v.Ap_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotday");
            assert!(v.Ap_hotday.get() >= 0.0 && v.Ap_hotday.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotday.get());
            assert!(v.Ap_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldday");
            assert!(v.Ap_coldday.get() >= 0.0 && v.Ap_coldday.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836 * scale), Ratio::new(41.6971614919), Ratio::new(6.9065794711)) {
            assert!(v.F107_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotmean");
            assert!(v.F107_hotmean.get() >= 60.0 && v.F107_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotmean.get());
            assert!(v.F107_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_nominal");
            assert!(v.F107_nominal.get() >= 60.0 && v.F107_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_nominal.get());
            assert!(v.F107_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldmean");
            assert!(v.F107_coldmean.get() >= 60.0 && v.F107_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldmean.get());
            assert!(v.F107_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotday");
            assert!(v.F107_hotday.get() >= 60.0 && v.F107_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotday.get());
            assert!(v.F107_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldday");
            assert!(v.F107_coldday.get() >= 60.0 && v.F107_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldday.get());
            assert!(v.F107bar_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_nominal");
            assert!(v.F107bar_nominal.get() >= 60.0 && v.F107bar_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107bar_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_nominal.get());
            assert!(v.F107bar_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotmean");
            assert!(v.F107bar_hotmean.get() >= 60.0 && v.F107bar_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotmean.get());
            assert!(v.F107bar_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldmean");
            assert!(v.F107bar_coldmean.get() >= 60.0 && v.F107bar_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldmean.get());
            assert!(v.F107bar_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotday");
            assert!(v.F107bar_hotday.get() >= 60.0 && v.F107bar_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotday.get());
            assert!(v.F107bar_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldday");
            assert!(v.F107bar_coldday.get() >= 60.0 && v.F107bar_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldday.get());
            assert!(v.Ap_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_nominal");
            assert!(v.Ap_nominal.get() >= 0.0 && v.Ap_nominal.get() <= 400.0, "l3_solar_interface answered {} for Ap_nominal, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_nominal.get());
            assert!(v.Ap_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotmean");
            assert!(v.Ap_hotmean.get() >= 0.0 && v.Ap_hotmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotmean.get());
            assert!(v.Ap_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldmean");
            assert!(v.Ap_coldmean.get() >= 0.0 && v.Ap_coldmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldmean.get());
            assert!(v.Ap_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotday");
            assert!(v.Ap_hotday.get() >= 0.0 && v.Ap_hotday.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotday.get());
            assert!(v.Ap_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldday");
            assert!(v.Ap_coldday.get() >= 0.0 && v.Ap_coldday.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919 * scale), Ratio::new(6.9065794711)) {
            assert!(v.F107_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotmean");
            assert!(v.F107_hotmean.get() >= 60.0 && v.F107_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotmean.get());
            assert!(v.F107_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_nominal");
            assert!(v.F107_nominal.get() >= 60.0 && v.F107_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_nominal.get());
            assert!(v.F107_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldmean");
            assert!(v.F107_coldmean.get() >= 60.0 && v.F107_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldmean.get());
            assert!(v.F107_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotday");
            assert!(v.F107_hotday.get() >= 60.0 && v.F107_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotday.get());
            assert!(v.F107_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldday");
            assert!(v.F107_coldday.get() >= 60.0 && v.F107_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldday.get());
            assert!(v.F107bar_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_nominal");
            assert!(v.F107bar_nominal.get() >= 60.0 && v.F107bar_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107bar_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_nominal.get());
            assert!(v.F107bar_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotmean");
            assert!(v.F107bar_hotmean.get() >= 60.0 && v.F107bar_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotmean.get());
            assert!(v.F107bar_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldmean");
            assert!(v.F107bar_coldmean.get() >= 60.0 && v.F107bar_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldmean.get());
            assert!(v.F107bar_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotday");
            assert!(v.F107bar_hotday.get() >= 60.0 && v.F107bar_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotday.get());
            assert!(v.F107bar_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldday");
            assert!(v.F107bar_coldday.get() >= 60.0 && v.F107bar_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldday.get());
            assert!(v.Ap_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_nominal");
            assert!(v.Ap_nominal.get() >= 0.0 && v.Ap_nominal.get() <= 400.0, "l3_solar_interface answered {} for Ap_nominal, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_nominal.get());
            assert!(v.Ap_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotmean");
            assert!(v.Ap_hotmean.get() >= 0.0 && v.Ap_hotmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotmean.get());
            assert!(v.Ap_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldmean");
            assert!(v.Ap_coldmean.get() >= 0.0 && v.Ap_coldmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldmean.get());
            assert!(v.Ap_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotday");
            assert!(v.Ap_hotday.get() >= 0.0 && v.Ap_hotday.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotday.get());
            assert!(v.Ap_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldday");
            assert!(v.Ap_coldday.get() >= 0.0 && v.Ap_coldday.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldday.get());
        }
    }
    for scale in [0.001_f64, 0.1, 1.0, 10.0, 1000.0] {
        if let Ok(v) = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711 * scale)) {
            assert!(v.F107_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotmean");
            assert!(v.F107_hotmean.get() >= 60.0 && v.F107_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotmean.get());
            assert!(v.F107_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_nominal");
            assert!(v.F107_nominal.get() >= 60.0 && v.F107_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_nominal.get());
            assert!(v.F107_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldmean");
            assert!(v.F107_coldmean.get() >= 60.0 && v.F107_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldmean.get());
            assert!(v.F107_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_hotday");
            assert!(v.F107_hotday.get() >= 60.0 && v.F107_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_hotday.get());
            assert!(v.F107_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107_coldday");
            assert!(v.F107_coldday.get() >= 60.0 && v.F107_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107_coldday.get());
            assert!(v.F107bar_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_nominal");
            assert!(v.F107bar_nominal.get() >= 60.0 && v.F107bar_nominal.get() <= 400.0, "l3_solar_interface answered {} for F107bar_nominal, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_nominal.get());
            assert!(v.F107bar_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotmean");
            assert!(v.F107bar_hotmean.get() >= 60.0 && v.F107bar_hotmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotmean.get());
            assert!(v.F107bar_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldmean");
            assert!(v.F107bar_coldmean.get() >= 60.0 && v.F107bar_coldmean.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldmean, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldmean.get());
            assert!(v.F107bar_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_hotday");
            assert!(v.F107bar_hotday.get() >= 60.0 && v.F107bar_hotday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_hotday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_hotday.get());
            assert!(v.F107bar_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for F107bar_coldday");
            assert!(v.F107bar_coldday.get() >= 60.0 && v.F107bar_coldday.get() <= 400.0, "l3_solar_interface answered {} for F107bar_coldday, outside its declared domain 60 … 400 — the guard did not stop it", v.F107bar_coldday.get());
            assert!(v.Ap_nominal.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_nominal");
            assert!(v.Ap_nominal.get() >= 0.0 && v.Ap_nominal.get() <= 400.0, "l3_solar_interface answered {} for Ap_nominal, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_nominal.get());
            assert!(v.Ap_hotmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotmean");
            assert!(v.Ap_hotmean.get() >= 0.0 && v.Ap_hotmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotmean.get());
            assert!(v.Ap_coldmean.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldmean");
            assert!(v.Ap_coldmean.get() >= 0.0 && v.Ap_coldmean.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldmean, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldmean.get());
            assert!(v.Ap_hotday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_hotday");
            assert!(v.Ap_hotday.get() >= 0.0 && v.Ap_hotday.get() <= 400.0, "l3_solar_interface answered {} for Ap_hotday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_hotday.get());
            assert!(v.Ap_coldday.get().is_finite(), "l3_solar_interface produced a value that is not a number for Ap_coldday");
            assert!(v.Ap_coldday.get() >= 0.0 && v.Ap_coldday.get() <= 400.0, "l3_solar_interface answered {} for Ap_coldday, outside its declared domain 0 … 400 — the guard did not stop it", v.Ap_coldday.get());
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
    let a = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711));
    let b = model::evaluate(Ratio::new(158.3304112313), Ratio::new(175.5520201913), Ratio::new(141.1088022713), Ratio::new(209.7835016728), Ratio::new(109.8365800491), Ratio::new(22.095389), Ratio::new(26.69530964), Ratio::new(17.49546836), Ratio::new(41.6971614919), Ratio::new(6.9065794711));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.F107_hotmean.get().to_bits() == y.F107_hotmean.get().to_bits(), "l3_solar_interface is not deterministic for F107_hotmean: {} then {}", x.F107_hotmean.get(), y.F107_hotmean.get());
            assert!(x.F107_nominal.get().to_bits() == y.F107_nominal.get().to_bits(), "l3_solar_interface is not deterministic for F107_nominal: {} then {}", x.F107_nominal.get(), y.F107_nominal.get());
            assert!(x.F107_coldmean.get().to_bits() == y.F107_coldmean.get().to_bits(), "l3_solar_interface is not deterministic for F107_coldmean: {} then {}", x.F107_coldmean.get(), y.F107_coldmean.get());
            assert!(x.F107_hotday.get().to_bits() == y.F107_hotday.get().to_bits(), "l3_solar_interface is not deterministic for F107_hotday: {} then {}", x.F107_hotday.get(), y.F107_hotday.get());
            assert!(x.F107_coldday.get().to_bits() == y.F107_coldday.get().to_bits(), "l3_solar_interface is not deterministic for F107_coldday: {} then {}", x.F107_coldday.get(), y.F107_coldday.get());
            assert!(x.F107bar_nominal.get().to_bits() == y.F107bar_nominal.get().to_bits(), "l3_solar_interface is not deterministic for F107bar_nominal: {} then {}", x.F107bar_nominal.get(), y.F107bar_nominal.get());
            assert!(x.F107bar_hotmean.get().to_bits() == y.F107bar_hotmean.get().to_bits(), "l3_solar_interface is not deterministic for F107bar_hotmean: {} then {}", x.F107bar_hotmean.get(), y.F107bar_hotmean.get());
            assert!(x.F107bar_coldmean.get().to_bits() == y.F107bar_coldmean.get().to_bits(), "l3_solar_interface is not deterministic for F107bar_coldmean: {} then {}", x.F107bar_coldmean.get(), y.F107bar_coldmean.get());
            assert!(x.F107bar_hotday.get().to_bits() == y.F107bar_hotday.get().to_bits(), "l3_solar_interface is not deterministic for F107bar_hotday: {} then {}", x.F107bar_hotday.get(), y.F107bar_hotday.get());
            assert!(x.F107bar_coldday.get().to_bits() == y.F107bar_coldday.get().to_bits(), "l3_solar_interface is not deterministic for F107bar_coldday: {} then {}", x.F107bar_coldday.get(), y.F107bar_coldday.get());
            assert!(x.Ap_nominal.get().to_bits() == y.Ap_nominal.get().to_bits(), "l3_solar_interface is not deterministic for Ap_nominal: {} then {}", x.Ap_nominal.get(), y.Ap_nominal.get());
            assert!(x.Ap_hotmean.get().to_bits() == y.Ap_hotmean.get().to_bits(), "l3_solar_interface is not deterministic for Ap_hotmean: {} then {}", x.Ap_hotmean.get(), y.Ap_hotmean.get());
            assert!(x.Ap_coldmean.get().to_bits() == y.Ap_coldmean.get().to_bits(), "l3_solar_interface is not deterministic for Ap_coldmean: {} then {}", x.Ap_coldmean.get(), y.Ap_coldmean.get());
            assert!(x.Ap_hotday.get().to_bits() == y.Ap_hotday.get().to_bits(), "l3_solar_interface is not deterministic for Ap_hotday: {} then {}", x.Ap_hotday.get(), y.Ap_hotday.get());
            assert!(x.Ap_coldday.get().to_bits() == y.Ap_coldday.get().to_bits(), "l3_solar_interface is not deterministic for Ap_coldday: {} then {}", x.Ap_coldday.get(), y.Ap_coldday.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("l3_solar_interface refused on one call and answered on the other"),
    }
}

