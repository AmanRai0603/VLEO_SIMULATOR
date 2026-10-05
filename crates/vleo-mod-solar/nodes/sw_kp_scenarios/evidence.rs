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

/// Aman Rai's case «the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.», from their own Rust code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(2.5)).expect("the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: the author's code answers this case and the node refused it");
    let err = relative_error(got.Kp_peak_hotday.get(), 7.9850088183);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: got {} and the author's code gave 7.9850088183; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_hotday.get(), err);
    let err = relative_error(got.Kp_mean_coldday.get(), 0.5416666667);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: Kp_mean_coldday is {} and the author's code gave 0.5416666667; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_coldday.get(), err);
    let err = relative_error(got.Kp_mean_coldmean.get(), 1.9583333333);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: Kp_mean_coldmean is {} and the author's code gave 1.9583333333; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_coldmean.get(), err);
    let err = relative_error(got.Kp_mean_hotday.get(), 5.7893772894);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: Kp_mean_hotday is {} and the author's code gave 5.7893772894; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_hotday.get(), err);
    let err = relative_error(got.Kp_mean_hotmean.get(), 3.7333333333);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: Kp_mean_hotmean is {} and the author's code gave 3.7333333333; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_hotmean.get(), err);
    let err = relative_error(got.Kp_mean_nominal.get(), 2.6245833333);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: Kp_mean_nominal is {} and the author's code gave 2.6245833333; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_nominal.get(), err);
    let err = relative_error(got.Kp_peak_coldday.get(), 1.5);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: Kp_peak_coldday is {} and the author's code gave 1.5; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_coldday.get(), err);
    let err = relative_error(got.Kp_peak_coldmean.get(), 2.9166666667);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: Kp_peak_coldmean is {} and the author's code gave 2.9166666667; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_coldmean.get(), err);
    let err = relative_error(got.Kp_peak_hotmean.get(), 5.0666666667);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: Kp_peak_hotmean is {} and the author's code gave 5.0666666667; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_hotmean.get(), err);
    let err = relative_error(got.Kp_peak_nominal.get(), 3.8366290019);
    assert!(err <= 1e-9, "the primary: the worst slot of the worst day. ap 90 is a bin centre, so the peak offset 1.7469 is exact and only the scale is interpolated, between Kp 6 at ap 80 and Kp 19/3 at ap 94.: Kp_peak_nominal is {} and the author's code gave 3.8366290019; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_nominal.get(), err);
}

/// Aman Rai's case «a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code», from their own Rust code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(57.5), Ratio::new(2.5)).expect("a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: the author's code answers this case and the node refused it");
    let err = relative_error(got.Kp_peak_hotday.get(), 6.924242424242424);
    assert!(err <= 1e-9, "a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: got {} and the author's code gave 6.924242424242424; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_hotday.get(), err);
    let err = relative_error(got.Kp_mean_coldday.get(), 0.5416666666666667);
    assert!(err <= 1e-9, "a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: Kp_mean_coldday is {} and the author's code gave 0.5416666666666667; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_coldday.get(), err);
    let err = relative_error(got.Kp_mean_coldmean.get(), 1.9583333333333335);
    assert!(err <= 1e-9, "a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: Kp_mean_coldmean is {} and the author's code gave 1.9583333333333335; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_coldmean.get(), err);
    let err = relative_error(got.Kp_mean_hotday.get(), 5.045454545454546);
    assert!(err <= 1e-9, "a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: Kp_mean_hotday is {} and the author's code gave 5.045454545454546; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_hotday.get(), err);
    let err = relative_error(got.Kp_mean_hotmean.get(), 3.733333333333334);
    assert!(err <= 1e-9, "a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: Kp_mean_hotmean is {} and the author's code gave 3.733333333333334; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_hotmean.get(), err);
    let err = relative_error(got.Kp_mean_nominal.get(), 2.6245833333333333);
    assert!(err <= 1e-9, "a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: Kp_mean_nominal is {} and the author's code gave 2.6245833333333333; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_nominal.get(), err);
    let err = relative_error(got.Kp_peak_coldday.get(), 1.5);
    assert!(err <= 1e-9, "a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: Kp_peak_coldday is {} and the author's code gave 1.5; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_coldday.get(), err);
    let err = relative_error(got.Kp_peak_coldmean.get(), 2.916666666666667);
    assert!(err <= 1e-9, "a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: Kp_peak_coldmean is {} and the author's code gave 2.916666666666667; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_coldmean.get(), err);
    let err = relative_error(got.Kp_peak_hotmean.get(), 5.066666666666666);
    assert!(err <= 1e-9, "a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: Kp_peak_hotmean is {} and the author's code gave 5.066666666666666; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_hotmean.get(), err);
    let err = relative_error(got.Kp_peak_nominal.get(), 3.836629001883239);
    assert!(err <= 1e-9, "a quieter worst day, ap 57.5: the peak slot of the disturbed single day, run in this node's own code: Kp_peak_nominal is {} and the author's code gave 3.836629001883239; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_nominal.get(), err);
}

/// Aman Rai's case «a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code», from their own Rust code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(20.0), Ratio::new(40.0), Ratio::new(5.0), Ratio::new(154.0), Ratio::new(1.0)).expect("a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: the author's code answers this case and the node refused it");
    let err = relative_error(got.Kp_peak_hotday.get(), 8.913671102559992);
    assert!(err <= 1e-9, "a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: got {} and the author's code gave 8.913671102559992; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_hotday.get(), err);
    let err = relative_error(got.Kp_mean_coldday.get(), 0.20833333333333334);
    assert!(err <= 1e-9, "a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: Kp_mean_coldday is {} and the author's code gave 0.20833333333333334; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_coldday.get(), err);
    let err = relative_error(got.Kp_mean_coldmean.get(), 1.2916666666666665);
    assert!(err <= 1e-9, "a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: Kp_mean_coldmean is {} and the author's code gave 1.2916666666666665; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_coldmean.get(), err);
    let err = relative_error(got.Kp_mean_hotday.get(), 6.869807377807377);
    assert!(err <= 1e-9, "a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: Kp_mean_hotday is {} and the author's code gave 6.869807377807377; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_hotday.get(), err);
    let err = relative_error(got.Kp_mean_hotmean.get(), 4.451166087962963);
    assert!(err <= 1e-9, "a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: Kp_mean_hotmean is {} and the author's code gave 4.451166087962963; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_hotmean.get(), err);
    let err = relative_error(got.Kp_mean_nominal.get(), 3.3814814814814818);
    assert!(err <= 1e-9, "a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: Kp_mean_nominal is {} and the author's code gave 3.3814814814814818; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_mean_nominal.get(), err);
    let err = relative_error(got.Kp_peak_coldday.get(), 1.1666666666666667);
    assert!(err <= 1e-9, "a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: Kp_peak_coldday is {} and the author's code gave 1.1666666666666667; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_coldday.get(), err);
    let err = relative_error(got.Kp_peak_coldmean.get(), 2.25);
    assert!(err <= 1e-9, "a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: Kp_peak_coldmean is {} and the author's code gave 2.25; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_coldmean.get(), err);
    let err = relative_error(got.Kp_peak_hotmean.get(), 6.063552188552189);
    assert!(err <= 1e-9, "a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: Kp_peak_hotmean is {} and the author's code gave 6.063552188552189; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_hotmean.get(), err);
    let err = relative_error(got.Kp_peak_nominal.get(), 4.566666666666666);
    assert!(err <= 1e-9, "a stormier worst day, ap 154, with every other scenario moved too: run in this node's own code: Kp_peak_nominal is {} and the author's code gave 4.566666666666666; relative error {} is more than their tolerance 1e-9. Take it to the author; do not widen the tolerance.", got.Kp_peak_nominal.get(), err);
}

/// Aman Rai's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own Rust code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(90.0), Ratio::new(500.0));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got.as_ref().map(|a| a.Kp_peak_hotday.get()));
}

/// Aman Rai's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own Rust code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(12.5), Ratio::new(25.0), Ratio::new(7.5), Ratio::new(400.0), Ratio::new(2.5));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got.as_ref().map(|a| a.Kp_peak_hotday.get()));
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 23 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_kp_scenarios::evaluate;
    assert_eq!(evaluate(6.25, 12.5, 3.75, 45.0, 1.25).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x4019350295fad40b, [0x3ffaaaaaaaaaaaab, 0x4004ff258bf258bf, 0x3fed555555555555, 0x401273fc3518a6e0, 0x3fd0000000000000, 0x4005000000000000, 0x400eb16a8bcfca01, 0x3ffe000000000000, 0x3ff3555555555555])), "at (6.25, 12.5, 3.75, 45.0, 1.25)");
    assert_eq!(evaluate(11.25, 22.5, 6.75, 81.0, 2.25).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x401edc17c6a8602e, [0x4003d4b17e4b17e4, 0x400c97b425ed097b, 0x3ffd111111111111, 0x40166d9d01328950, 0x3fdd555555555555, 0x400d050fe8dbd780, 0x4013555555555555, 0x4006333333333333, 0x3ff6aaaaaaaaaaaa])), "at (11.25, 22.5, 6.75, 81.0, 2.25)");
    assert_eq!(evaluate(12.375, 24.75, 7.425, 89.1, 2.475).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x401fd4fe50696bf4, [0x4004e14d242e6bdb, 0x400dbd401845c8a2, 0x3fff2c5f92c5f92c, 0x401715a6c4c2f199, 0x3fe1111111111112, 0x400e8694aeb764f3, 0x40142c5f92c5f92c, 0x400740da740da740, 0x3ff7ddddddddddde])), "at (12.375, 24.75, 7.425, 89.1, 2.475)");
    assert_eq!(evaluate(12.5, 25.0, 7.5, 90.0, 2.5).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x401ff0a626d43460, [0x4004ff258bf258bf, 0x400ddddddddddddf, 0x3fff555555555556, 0x4017285285285285, 0x3fe1555555555556, 0x400eb16a8bcfca01, 0x4014444444444444, 0x4007555555555556, 0x3ff8000000000000])), "at (12.5, 25.0, 7.5, 90.0, 2.5)");
    assert_eq!(evaluate(12.625, 25.25, 7.575, 90.9, 2.525).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x40200218baa0de4a, [0x40051ae6bdc80576, 0x400dfb9714eb813e, 0x3fff8a36e2eb1c43, 0x40173e0d4da79c7a, 0x3fe192c5f92c5f93, 0x400ec800c578d10b, 0x40145810624dd2f2, 0x4007779162861a7f, 0x3ff81eb851eb851e])), "at (12.625, 25.25, 7.575, 90.9, 2.525)");
    assert_eq!(evaluate(13.750000000000002, 27.500000000000004, 8.25, 99.00000000000001, 2.75).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x40204f49fe830197, [0x400614b17e4b17e4, 0x400f071a04663f91, 0x4000b3126e978d50, 0x4017ec1b200bb2dd, 0x3fe3bbbbbbbbbbbb, 0x400f9348cc6a1064, 0x40150a3d70a3d70a, 0x4008abadd93d08f8, 0x3ff9333333333333])), "at (13.750000000000002, 27.500000000000004, 8.25, 99.00000000000001, 2.75)");
    assert!(matches!(evaluate(25.0, 50.0, 15.0, 180.0, 5.0), Err(MethodError::Refused(_))), "at (25.0, 50.0, 15.0, 180.0, 5.0)");
    assert_eq!(evaluate(6.25, 12.5, 3.75, 28.75, 1.25).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x40156d3a06d3a06d, [0x3ffaaaaaaaaaaaab, 0x4004ff258bf258bf, 0x3fed555555555555, 0x400f9bb817aa7069, 0x3fd0000000000000, 0x4005000000000000, 0x400eb16a8bcfca01, 0x3ffe000000000000, 0x3ff3555555555555])), "at (6.25, 12.5, 3.75, 28.75, 1.25)");
    assert_eq!(evaluate(11.25, 22.5, 6.75, 51.75, 2.25).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x401a9018d3018d30, [0x4003d4b17e4b17e4, 0x400c97b425ed097b, 0x3ffd111111111111, 0x401365d9d8b57009, 0x3fdd555555555555, 0x400d050fe8dbd780, 0x4013555555555555, 0x4006333333333333, 0x3ff6aaaaaaaaaaaa])), "at (11.25, 22.5, 6.75, 51.75, 2.25)");
    assert_eq!(evaluate(12.375, 24.75, 7.425, 56.925, 2.475).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x401b9a5643a5643a, [0x4004e14d242e6bdb, 0x400dbd401845c8a2, 0x3fff2c5f92c5f92c, 0x40141f6be4cc535a, 0x3fe1111111111112, 0x400e8694aeb764f3, 0x40142c5f92c5f92c, 0x400740da740da740, 0x3ff7ddddddddddde])), "at (12.375, 24.75, 7.425, 56.925, 2.475)");
    assert_eq!(evaluate(12.5, 25.0, 7.5, 57.5, 2.5).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x401bb26c9b26c9b2, [0x4004ff258bf258bf, 0x400ddddddddddddf, 0x3fff555555555556, 0x40142e8ba2e8ba2f, 0x3fe1555555555556, 0x400eb16a8bcfca01, 0x4014444444444444, 0x4007555555555556, 0x3ff8000000000000])), "at (12.5, 25.0, 7.5, 57.5, 2.5)");
    assert_eq!(evaluate(12.625, 25.25, 7.575, 58.075, 2.525).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x401bc7ea9eba7854, [0x40051ae6bdc80576, 0x400dfb9714eb813e, 0x3fff8a36e2eb1c43, 0x40143e4c26a39f94, 0x3fe192c5f92c5f93, 0x400ec800c578d10b, 0x40145810624dd2f2, 0x4007779162861a7f, 0x3ff81eb851eb851e])), "at (12.625, 25.25, 7.575, 58.075, 2.525)");
    assert_eq!(evaluate(13.750000000000002, 27.500000000000004, 8.25, 63.25000000000001, 2.75).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x401c8958beeb9bfb, [0x400614b17e4b17e4, 0x400f071a04663f91, 0x4000b3126e978d50, 0x4014cc10c835b01d, 0x3fe3bbbbbbbbbbbb, 0x400f9348cc6a1064, 0x40150a3d70a3d70a, 0x4008abadd93d08f8, 0x3ff9333333333333])), "at (13.750000000000002, 27.500000000000004, 8.25, 63.25000000000001, 2.75)");
    assert_eq!(evaluate(25.0, 50.0, 15.0, 115.0, 5.0).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x4020d2f223edaa99, [0x400ddddddddddddf, 0x4013237530eca864, 0x40072a3d70a3d70a, 0x40191a45ac1cb247, 0x3ff4aaaaaaaaaaaa, 0x4014444444444444, 0x401a326c9b26c9b2, 0x40103a9386822b64, 0x4002000000000000])), "at (25.0, 50.0, 15.0, 115.0, 5.0)");
    assert_eq!(evaluate(10.0, 20.0, 2.5, 77.0, 0.5).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x401e5b8d7ac828a2, [0x4002aa3d70a3d70a, 0x400b0d4629b7f0d5, 0x3fe1555555555556, 0x4016150150150150, 0x3fc0000000000000, 0x400b58b545e7e501, 0x4012444444444444, 0x3ff8000000000000, 0x3ff1555555555555])), "at (10.0, 20.0, 2.5, 77.0, 0.5)");
    assert_eq!(evaluate(18.0, 36.0, 4.5, 138.6, 0.9).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x402170dae6604e3a, [0x4009c413b8b302a8, 0x40112ed4c9ccf431, 0x3ff2444444444444, 0x401a8f65ead3994d, 0x3fc8888888888889, 0x401162fc962fc963, 0x40175d548d9ac531, 0x4000cccccccccccc, 0x3ff2666666666666])), "at (18.0, 36.0, 4.5, 138.6, 0.9)");
    assert_eq!(evaluate(19.8, 39.6, 4.95, 152.46, 0.99).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x4021c9e7b807dcc1, [0x400aec5ab8043f6a, 0x4011c0b6fc297209, 0x3ff46d3a06d3a06d, 0x401b63277f9a915c, 0x3fca740da740da74, 0x40122dbd194237fa, 0x40182d901583ac03, 0x4001e147ae147ae1, 0x3ff2a3d70a3d70a4])), "at (19.8, 39.6, 4.95, 152.46, 0.99)");
    assert_eq!(evaluate(20.0, 40.0, 5.0, 154.0, 1.0).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x4021d3ccb2e19742, [0x400b0d4629b7f0d5, 0x4011cdfe7ba375f3, 0x3ff4aaaaaaaaaaaa, 0x401b7aaec9060241, 0x3fcaaaaaaaaaaaab, 0x4012444444444444, 0x40184113d32daefa, 0x4002000000000000, 0x3ff2aaaaaaaaaaab])), "at (20.0, 40.0, 5.0, 154.0, 1.0)");
    assert_eq!(evaluate(20.2, 40.4, 5.05, 155.54, 1.01).map(|(v, p)| (v.to_bits(), p.map(f64::to_bits))), Ok((0x4021dc42ad527637, [0x400b2e319b6ba23f, 0x4011db45fb1d79dd, 0x3ff4e81b4e81b4e8, 0x401b8f58119fbc0f, 0x3fcae147ae147ae2, 0x40125acb6f46508e, 0x4018549790d7b1f1, 0x40021eb851eb851e, 0x3ff2b17e4b17e4b2])), "at (20.2, 40.4, 5.05, 155.54, 1.01)");
    assert!(matches!(evaluate(22.0, 44.0, 5.5, 169.4, 1.1), Err(MethodError::Refused(_))), "at (22.0, 44.0, 5.5, 169.4, 1.1)");
    assert!(matches!(evaluate(40.0, 80.0, 10.0, 308.0, 2.0), Err(MethodError::Refused(_))), "at (40.0, 80.0, 10.0, 308.0, 2.0)");
    assert!(matches!(evaluate(12.5, 25.0, 7.5, 90.0, 500.0), Err(MethodError::Refused(_))), "at (12.5, 25.0, 7.5, 90.0, 500.0)");
    assert!(matches!(evaluate(12.5, 25.0, 7.5, 400.0, 2.5), Err(MethodError::Refused(_))), "at (12.5, 25.0, 7.5, 400.0, 2.5)");
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

