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

/// the author's case «phase 0.025 — 418 days from two cycles — mean F10.7 71.76 sfu.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(0.025)).expect("phase 0.025 — 418 days from two cycles — mean F10.7 71.76 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 71.7632);
    assert!(err <= 1e-12, "phase 0.025 — 418 days from two cycles — mean F10.7 71.76 sfu.: got {} and the author's code gave 71.7632; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.075 — 418 days from two cycles — mean F10.7 83.89 sfu.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(0.075)).expect("phase 0.075 — 418 days from two cycles — mean F10.7 83.89 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 83.8947);
    assert!(err <= 1e-12, "phase 0.075 — 418 days from two cycles — mean F10.7 83.89 sfu.: got {} and the author's code gave 83.8947; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.125 — 418 days from two cycles — mean F10.7 99.22 sfu.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(0.125)).expect("phase 0.125 — 418 days from two cycles — mean F10.7 99.22 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 99.2177);
    assert!(err <= 1e-12, "phase 0.125 — 418 days from two cycles — mean F10.7 99.22 sfu.: got {} and the author's code gave 99.2177; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.175 — 418 days from two cycles — mean F10.7 111.05 sfu.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(0.175)).expect("phase 0.175 — 418 days from two cycles — mean F10.7 111.05 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 111.0502);
    assert!(err <= 1e-12, "phase 0.175 — 418 days from two cycles — mean F10.7 111.05 sfu.: got {} and the author's code gave 111.0502; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.225 — 418 days from two cycles — mean F10.7 135.66 sfu.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(0.225)).expect("phase 0.225 — 418 days from two cycles — mean F10.7 135.66 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 135.6555);
    assert!(err <= 1e-12, "phase 0.225 — 418 days from two cycles — mean F10.7 135.66 sfu.: got {} and the author's code gave 135.6555; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.275 — 418 days from two cycles — mean F10.7 159.66 sfu.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Ratio::new(0.275)).expect("phase 0.275 — 418 days from two cycles — mean F10.7 159.66 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 159.6603);
    assert!(err <= 1e-12, "phase 0.275 — 418 days from two cycles — mean F10.7 159.66 sfu.: got {} and the author's code gave 159.6603; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.325 — 416 days from two cycles — mean F10.7 145.37 sfu.», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Ratio::new(0.325)).expect("phase 0.325 — 416 days from two cycles — mean F10.7 145.37 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 145.3702);
    assert!(err <= 1e-12, "phase 0.325 — 416 days from two cycles — mean F10.7 145.37 sfu.: got {} and the author's code gave 145.3702; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.375 — 418 days from two cycles — mean F10.7 146.73 sfu.», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Ratio::new(0.375)).expect("phase 0.375 — 418 days from two cycles — mean F10.7 146.73 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 146.7273);
    assert!(err <= 1e-12, "phase 0.375 — 418 days from two cycles — mean F10.7 146.73 sfu.: got {} and the author's code gave 146.7273; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.425 — 418 days from two cycles — mean F10.7 165.28 sfu.», from their own  code.
#[test]
fn case_9() {
    let got = model::evaluate(Ratio::new(0.425)).expect("phase 0.425 — 418 days from two cycles — mean F10.7 165.28 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 165.2799);
    assert!(err <= 1e-12, "phase 0.425 — 418 days from two cycles — mean F10.7 165.28 sfu.: got {} and the author's code gave 165.2799; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.475 — 415 days from two cycles — mean F10.7 160.33 sfu.», from their own  code.
#[test]
fn case_10() {
    let got = model::evaluate(Ratio::new(0.475)).expect("phase 0.475 — 415 days from two cycles — mean F10.7 160.33 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 160.3301);
    assert!(err <= 1e-12, "phase 0.475 — 415 days from two cycles — mean F10.7 160.33 sfu.: got {} and the author's code gave 160.3301; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.525 — 418 days from two cycles — mean F10.7 135.32 sfu.», from their own  code.
#[test]
fn case_11() {
    let got = model::evaluate(Ratio::new(0.525)).expect("phase 0.525 — 418 days from two cycles — mean F10.7 135.32 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 135.3182);
    assert!(err <= 1e-12, "phase 0.525 — 418 days from two cycles — mean F10.7 135.32 sfu.: got {} and the author's code gave 135.3182; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.575 — 418 days from two cycles — mean F10.7 126.23 sfu.», from their own  code.
#[test]
fn case_12() {
    let got = model::evaluate(Ratio::new(0.575)).expect("phase 0.575 — 418 days from two cycles — mean F10.7 126.23 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 126.2321);
    assert!(err <= 1e-12, "phase 0.575 — 418 days from two cycles — mean F10.7 126.23 sfu.: got {} and the author's code gave 126.2321; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.625 — 418 days from two cycles — mean F10.7 105.84 sfu.», from their own  code.
#[test]
fn case_13() {
    let got = model::evaluate(Ratio::new(0.625)).expect("phase 0.625 — 418 days from two cycles — mean F10.7 105.84 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 105.8445);
    assert!(err <= 1e-12, "phase 0.625 — 418 days from two cycles — mean F10.7 105.84 sfu.: got {} and the author's code gave 105.8445; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.675 — 417 days from two cycles — mean F10.7 95.51 sfu.», from their own  code.
#[test]
fn case_14() {
    let got = model::evaluate(Ratio::new(0.675)).expect("phase 0.675 — 417 days from two cycles — mean F10.7 95.51 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 95.5084);
    assert!(err <= 1e-12, "phase 0.675 — 417 days from two cycles — mean F10.7 95.51 sfu.: got {} and the author's code gave 95.5084; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.725 — 358 days from two cycles — mean F10.7 86.99 sfu.», from their own  code.
#[test]
fn case_15() {
    let got = model::evaluate(Ratio::new(0.725)).expect("phase 0.725 — 358 days from two cycles — mean F10.7 86.99 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 86.9916);
    assert!(err <= 1e-12, "phase 0.725 — 358 days from two cycles — mean F10.7 86.99 sfu.: got {} and the author's code gave 86.9916; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.775 — 217 days from two cycles — mean F10.7 80.92 sfu.», from their own  code.
#[test]
fn case_16() {
    let got = model::evaluate(Ratio::new(0.775)).expect("phase 0.775 — 217 days from two cycles — mean F10.7 80.92 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 80.9171);
    assert!(err <= 1e-12, "phase 0.775 — 217 days from two cycles — mean F10.7 80.92 sfu.: got {} and the author's code gave 80.9171; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.825 — 406 days from two cycles — mean F10.7 76.21 sfu.», from their own  code.
#[test]
fn case_17() {
    let got = model::evaluate(Ratio::new(0.825)).expect("phase 0.825 — 406 days from two cycles — mean F10.7 76.21 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 76.2094);
    assert!(err <= 1e-12, "phase 0.825 — 406 days from two cycles — mean F10.7 76.21 sfu.: got {} and the author's code gave 76.2094; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.875 — 418 days from two cycles — mean F10.7 71.05 sfu.», from their own  code.
#[test]
fn case_18() {
    let got = model::evaluate(Ratio::new(0.875)).expect("phase 0.875 — 418 days from two cycles — mean F10.7 71.05 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 71.0478);
    assert!(err <= 1e-12, "phase 0.875 — 418 days from two cycles — mean F10.7 71.05 sfu.: got {} and the author's code gave 71.0478; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.925 — 418 days from two cycles — mean F10.7 71.54 sfu.», from their own  code.
#[test]
fn case_19() {
    let got = model::evaluate(Ratio::new(0.925)).expect("phase 0.925 — 418 days from two cycles — mean F10.7 71.54 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 71.5383);
    assert!(err <= 1e-12, "phase 0.925 — 418 days from two cycles — mean F10.7 71.54 sfu.: got {} and the author's code gave 71.5383; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «phase 0.975 — 416 days from two cycles — mean F10.7 67.65 sfu.», from their own  code.
#[test]
fn case_20() {
    let got = model::evaluate(Ratio::new(0.975)).expect("phase 0.975 — 416 days from two cycles — mean F10.7 67.65 sfu.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 67.6538);
    assert!(err <= 1e-12, "phase 0.975 — 416 days from two cycles — mean F10.7 67.65 sfu.: got {} and the author's code gave 67.6538; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_21() {
    let got = model::evaluate(Ratio::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 141 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_mean_cycle_level::evaluate;
    assert_eq!(evaluate(0.0125).map(f64::to_bits), Ok(0x4051f0d844d013a9), "at (0.0125)");
    assert_eq!(evaluate(0.022500000000000003).map(f64::to_bits), Ok(0x4051f0d844d013a9), "at (0.022500000000000003)");
    assert_eq!(evaluate(0.02475).map(f64::to_bits), Ok(0x4051f0d844d013a9), "at (0.02475)");
    assert_eq!(evaluate(0.025).map(f64::to_bits), Ok(0x4051f0d844d013a9), "at (0.025)");
    assert_eq!(evaluate(0.02525).map(f64::to_bits), Ok(0x4051f4ba14cec41e), "at (0.02525)");
    assert_eq!(evaluate(0.027500000000000004).map(f64::to_bits), Ok(0x405217aa64c2f838), "at (0.027500000000000004)");
    assert_eq!(evaluate(0.05).map(f64::to_bits), Ok(0x4053750d844d013b), "at (0.05)");
    assert_eq!(evaluate(0.0375).map(f64::to_bits), Ok(0x4052b2f2e48e8a72), "at (0.0375)");
    assert_eq!(evaluate(0.0675).map(f64::to_bits), Ok(0x405484cc63f14120), "at (0.0675)");
    assert_eq!(evaluate(0.07425).map(f64::to_bits), Ok(0x4054ed9d53cddd6e), "at (0.07425)");
    assert_eq!(evaluate(0.075).map(f64::to_bits), Ok(0x4054f942c3c9eecc), "at (0.075)");
    assert_eq!(evaluate(0.07575).map(f64::to_bits), Ok(0x405507f88b977857), "at (0.07575)");
    assert_eq!(evaluate(0.0825).map(f64::to_bits), Ok(0x40558c5c91d14e3c), "at (0.0825)");
    assert_eq!(evaluate(0.15).map(f64::to_bits), Ok(0x405a4892a3055326), "at (0.15)");
    assert_eq!(evaluate(0.0625).map(f64::to_bits), Ok(0x40543728240b7803), "at (0.0625)");
    assert_eq!(evaluate(0.1125).map(f64::to_bits), Ok(0x4057d8c3c9eecbfb), "at (0.1125)");
    assert_eq!(evaluate(0.12375).map(f64::to_bits), Ok(0x4058b56a7ef9db22), "at (0.12375)");
    assert_eq!(evaluate(0.125).map(f64::to_bits), Ok(0x4058cdeecbfb15b5), "at (0.125)");
    assert_eq!(evaluate(0.12625).map(f64::to_bits), Ok(0x4058e0dd63886594), "at (0.12625)");
    assert_eq!(evaluate(0.1375).map(f64::to_bits), Ok(0x40598b40b780346e), "at (0.1375)");
    assert_eq!(evaluate(0.25).map(f64::to_bits), Ok(0x4062750d844d013a), "at (0.25)");
    assert_eq!(evaluate(0.0875).map(f64::to_bits), Ok(0x4055ee6dc5d63886), "at (0.0875)");
    assert_eq!(evaluate(0.1575).map(f64::to_bits), Ok(0x405aba2a30553262), "at (0.1575)");
    assert_eq!(evaluate(0.17325).map(f64::to_bits), Ok(0x405ba8b53f7ced92), "at (0.17325)");
    assert_eq!(evaluate(0.175).map(f64::to_bits), Ok(0x405bc3367a0f9097), "at (0.175)");
    assert_eq!(evaluate(0.17675).map(f64::to_bits), Ok(0x405bfa5423d9231d), "at (0.17675)");
    assert_eq!(evaluate(0.1925).map(f64::to_bits), Ok(0x405dea5f1bef49d0), "at (0.1925)");
    assert_eq!(evaluate(0.35).map(f64::to_bits), Ok(0x4062418f5c28f5c3), "at (0.35)");
    assert_eq!(evaluate(0.1125).map(f64::to_bits), Ok(0x4057d8c3c9eecbfb), "at (0.1125)");
    assert_eq!(evaluate(0.2025).map(f64::to_bits), Ok(0x405f2551c193b3a7), "at (0.2025)");
    assert_eq!(evaluate(0.22275).map(f64::to_bits), Ok(0x4060d18b5bb384fd), "at (0.22275)");
    assert_eq!(evaluate(0.225).map(f64::to_bits), Ok(0x4060f4f9db22d0e5), "at (0.225)");
    assert_eq!(evaluate(0.22725).map(f64::to_bits), Ok(0x4061178afc47e49b), "at (0.22725)");
    assert_eq!(evaluate(0.24750000000000003).map(f64::to_bits), Ok(0x40624ea5269595ff), "at (0.24750000000000003)");
    assert_eq!(evaluate(0.45).map(f64::to_bits), Ok(0x406459c28f5c28f5), "at (0.45)");
    assert_eq!(evaluate(0.1375).map(f64::to_bits), Ok(0x40598b40b780346e), "at (0.1375)");
    assert_eq!(evaluate(0.24750000000000003).map(f64::to_bits), Ok(0x40624ea5269595ff), "at (0.24750000000000003)");
    assert_eq!(evaluate(0.27225).map(f64::to_bits), Ok(0x4063cae1932d6ece), "at (0.27225)");
    assert_eq!(evaluate(0.275).map(f64::to_bits), Ok(0x4063f5212d773190), "at (0.275)");
    assert_eq!(evaluate(0.27775000000000005).map(f64::to_bits), Ok(0x4063dbfaa1511e00), "at (0.27775000000000005)");
    assert_eq!(evaluate(0.30250000000000005).map(f64::to_bits), Ok(0x4062f99fb3fa6df0), "at (0.30250000000000005)");
    assert_eq!(evaluate(0.55).map(f64::to_bits), Ok(0x406058ce075f6fd2), "at (0.55)");
    assert_eq!(evaluate(0.1625).map(f64::to_bits), Ok(0x405b05e48e8a71df), "at (0.1625)");
    assert_eq!(evaluate(0.29250000000000004).map(f64::to_bits), Ok(0x406355149a5657fc), "at (0.29250000000000004)");
    assert_eq!(evaluate(0.32175).map(f64::to_bits), Ok(0x40624991deefe500), "at (0.32175)");
    assert_eq!(evaluate(0.325).map(f64::to_bits), Ok(0x40622bd8adab9f56), "at (0.325)");
    assert_eq!(evaluate(0.32825000000000004).map(f64::to_bits), Ok(0x40622eab4e981388), "at (0.32825000000000004)");
    assert_eq!(evaluate(0.35750000000000004).map(f64::to_bits), Ok(0x40624812f6e8294a), "at (0.35750000000000004)");
    assert_eq!(evaluate(0.65).map(f64::to_bits), Ok(0x40592b4af4f0d844), "at (0.65)");
    assert_eq!(evaluate(0.1875).map(f64::to_bits), Ok(0x405d4ce5c91d14e4), "at (0.1875)");
    assert_eq!(evaluate(0.3375).map(f64::to_bits), Ok(0x406236b404ea4a8d), "at (0.3375)");
    assert_eq!(evaluate(0.37124999999999997).map(f64::to_bits), Ok(0x406254043d46b26c), "at (0.37124999999999997)");
    assert_eq!(evaluate(0.375).map(f64::to_bits), Ok(0x406257460aa64c30), "at (0.375)");
    assert_eq!(evaluate(0.37875000000000003).map(f64::to_bits), Ok(0x406283ccc2507209), "at (0.37875000000000003)");
    assert_eq!(evaluate(0.41250000000000003).map(f64::to_bits), Ok(0x40641489374bc6a9), "at (0.41250000000000003)");
    assert_eq!(evaluate(0.75).map(f64::to_bits), Ok(0x4054fd141205bc02), "at (0.75)");
    assert_eq!(evaluate(0.2125).map(f64::to_bits), Ok(0x40603022339c0ebe), "at (0.2125)");
    assert_eq!(evaluate(0.3825).map(f64::to_bits), Ok(0x4062b05379fa97e2), "at (0.3825)");
    assert_eq!(evaluate(0.42075).map(f64::to_bits), Ok(0x4064767e64f54d1f), "at (0.42075)");
    assert_eq!(evaluate(0.425).map(f64::to_bits), Ok(0x4064a8f4f0d844d0), "at (0.425)");
    assert_eq!(evaluate(0.42924999999999996).map(f64::to_bits), Ok(0x40649b7e4bcad9ad), "at (0.42924999999999996)");
    assert_eq!(evaluate(0.4675).map(f64::to_bits), Ok(0x406422527e521576), "at (0.4675)");
    assert_eq!(evaluate(0.85).map(f64::to_bits), Ok(0x4052683afb7e9100), "at (0.85)");
    assert_eq!(evaluate(0.2375).map(f64::to_bits), Ok(0x4061b503afb7e90f), "at (0.2375)");
    assert_eq!(evaluate(0.4275).map(f64::to_bits), Ok(0x4064a10980b24207), "at (0.4275)");
    assert_eq!(evaluate(0.47025).map(f64::to_bits), Ok(0x4064199c4fc1df32), "at (0.47025)");
    assert_eq!(evaluate(0.475).map(f64::to_bits), Ok(0x40640a902de00d1b), "at (0.475)");
    assert_eq!(evaluate(0.47974999999999995).map(f64::to_bits), Ok(0x4063be86eb0b7c35), "at (0.47974999999999995)");
    assert_eq!(evaluate(0.5225).map(f64::to_bits), Ok(0x406112339192641c), "at (0.5225)");
    assert_eq!(evaluate(0.95).map(f64::to_bits), Ok(0x40516625aee631f9), "at (0.95)");
    assert_eq!(evaluate(0.2625).map(f64::to_bits), Ok(0x4063351758e21965), "at (0.2625)");
    assert_eq!(evaluate(0.47250000000000003).map(f64::to_bits), Ok(0x4064127b9e060fe4), "at (0.47250000000000003)");
    assert_eq!(evaluate(0.51975).map(f64::to_bits), Ok(0x40613e38ee286726), "at (0.51975)");
    assert_eq!(evaluate(0.525).map(f64::to_bits), Ok(0x4060ea2eb1c432ca), "at (0.525)");
    assert_eq!(evaluate(0.53025).map(f64::to_bits), Ok(0x4060cba731d2e0e3), "at (0.53025)");
    assert_eq!(evaluate(0.5775000000000001).map(f64::to_bits), Ok(0x405f4d9d3458cd1c), "at (0.5775000000000001)");
    assert_eq!(evaluate(1.05).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.05)");
    assert_eq!(evaluate(0.2875).map(f64::to_bits), Ok(0x406382cf0d844d02), "at (0.2875)");
    assert_eq!(evaluate(0.5175).map(f64::to_bits), Ok(0x4061623d512ec6be), "at (0.5175)");
    assert_eq!(evaluate(0.5692499999999999).map(f64::to_bits), Ok(0x405fd1ba647fdc5a), "at (0.5692499999999999)");
    assert_eq!(evaluate(0.575).map(f64::to_bits), Ok(0x405f8edab9f559b4), "at (0.575)");
    assert_eq!(evaluate(0.58075).map(f64::to_bits), Ok(0x405ef8cd39da1661), "at (0.58075)");
    assert_eq!(evaluate(0.6325).map(f64::to_bits), Ok(0x405a12d249e44fa1), "at (0.6325)");
    assert_eq!(evaluate(1.15).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.15)");
    assert_eq!(evaluate(0.3125).map(f64::to_bits), Ok(0x40629e2acd9e83e5), "at (0.3125)");
    assert_eq!(evaluate(0.5625).map(f64::to_bits), Ok(0x4060101db22d0e56), "at (0.5625)");
    assert_eq!(evaluate(0.61875).map(f64::to_bits), Ok(0x405b192617c1bda4), "at (0.61875)");
    assert_eq!(evaluate(0.625).map(f64::to_bits), Ok(0x405a760c49ba5e35), "at (0.625)");
    assert_eq!(evaluate(0.63125).map(f64::to_bits), Ok(0x405a235bf487fcb9), "at (0.63125)");
    assert_eq!(evaluate(0.6875).map(f64::to_bits), Ok(0x40575844d013a92a), "at (0.6875)");
    assert_eq!(evaluate(1.25).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.25)");
    assert_eq!(evaluate(0.3375).map(f64::to_bits), Ok(0x406236b404ea4a8d), "at (0.3375)");
    assert_eq!(evaluate(0.6075).map(f64::to_bits), Ok(0x405c3ebaf1023639), "at (0.6075)");
    assert_eq!(evaluate(0.66825).map(f64::to_bits), Ok(0x405839d76cce5f74), "at (0.66825)");
    assert_eq!(evaluate(0.675).map(f64::to_bits), Ok(0x4057e089a0275254), "at (0.675)");
    assert_eq!(evaluate(0.6817500000000001).map(f64::to_bits), Ok(0x405796f3d3a1d323), "at (0.6817500000000001)");
    assert_eq!(evaluate(0.7425000000000002).map(f64::to_bits), Ok(0x40553764c2f837b4), "at (0.7425000000000002)");
    assert_eq!(evaluate(1.35).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.35)");
    assert_eq!(evaluate(0.3625).map(f64::to_bits), Ok(0x40624c6ab367a0f9), "at (0.3625)");
    assert_eq!(evaluate(0.6525).map(f64::to_bits), Ok(0x40590a379fa97e14), "at (0.6525)");
    assert_eq!(evaluate(0.71775).map(f64::to_bits), Ok(0x40560e7f90d9d777), "at (0.71775)");
    assert_eq!(evaluate(0.725).map(f64::to_bits), Ok(0x4055bf765fd8adac), "at (0.725)");
    assert_eq!(evaluate(0.73225).map(f64::to_bits), Ok(0x405587174e65bea1), "at (0.73225)");
    assert_eq!(evaluate(0.7975).map(f64::to_bits), Ok(0x4053b31cd5f99c39), "at (0.7975)");
    assert_eq!(evaluate(1.45).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.45)");
    assert_eq!(evaluate(0.3875).map(f64::to_bits), Ok(0x4062ebb1c432ca58), "at (0.3875)");
    assert_eq!(evaluate(0.6975).map(f64::to_bits), Ok(0x4056eb40f66a5508), "at (0.6975)");
    assert_eq!(evaluate(0.76725).map(f64::to_bits), Ok(0x405476f414a4d2b3), "at (0.76725)");
    assert_eq!(evaluate(0.775).map(f64::to_bits), Ok(0x40543ab1c432ca58), "at (0.775)");
    assert_eq!(evaluate(0.7827500000000001).map(f64::to_bits), Ok(0x40540bfe77d523b3), "at (0.7827500000000001)");
    assert_eq!(evaluate(0.8525000000000001).map(f64::to_bits), Ok(0x405257b69984a0e3), "at (0.8525000000000001)");
    assert_eq!(evaluate(1.55).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.55)");
    assert_eq!(evaluate(0.4125).map(f64::to_bits), Ok(0x40641489374bc6a8), "at (0.4125)");
    assert_eq!(evaluate(0.7424999999999999).map(f64::to_bits), Ok(0x40553764c2f837b5), "at (0.7424999999999999)");
    assert_eq!(evaluate(0.81675).map(f64::to_bits), Ok(0x40533f1d6adf71eb), "at (0.81675)");
    assert_eq!(evaluate(0.825).map(f64::to_bits), Ok(0x40530d66cf41f213), "at (0.825)");
    assert_eq!(evaluate(0.8332499999999999).map(f64::to_bits), Ok(0x4052d6e525892685), "at (0.8332499999999999)");
    assert_eq!(evaluate(0.9075).map(f64::to_bits), Ok(0x4051d776c8b43958), "at (0.9075)");
    assert_eq!(evaluate(1.65).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.65)");
    assert_eq!(evaluate(0.4375).map(f64::to_bits), Ok(0x4064815bc01a36e3), "at (0.4375)");
    assert_eq!(evaluate(0.7875).map(f64::to_bits), Ok(0x4053ef5f06f69447), "at (0.7875)");
    assert_eq!(evaluate(0.86625).map(f64::to_bits), Ok(0x4051fcde7ea5f84d), "at (0.86625)");
    assert_eq!(evaluate(0.875).map(f64::to_bits), Ok(0x4051c30f27bb2fec), "at (0.875)");
    assert_eq!(evaluate(0.88375).map(f64::to_bits), Ok(0x4051c88d844d013a), "at (0.88375)");
    assert_eq!(evaluate(0.9625000000000001).map(f64::to_bits), Ok(0x405127fec56d5cfa), "at (0.9625000000000001)");
    assert_eq!(evaluate(1.75).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.75)");
    assert_eq!(evaluate(0.4625).map(f64::to_bits), Ok(0x406432295e9e1b08), "at (0.4625)");
    assert_eq!(evaluate(0.8325).map(f64::to_bits), Ok(0x4052dbd9a95421c0), "at (0.8325)");
    assert_eq!(evaluate(0.9157500000000001).map(f64::to_bits), Ok(0x4051dca4c8366517), "at (0.9157500000000001)");
    assert_eq!(evaluate(0.925).map(f64::to_bits), Ok(0x4051e27381d7dbf5), "at (0.925)");
    assert_eq!(evaluate(0.93425).map(f64::to_bits), Ok(0x4051b4756eac8606), "at (0.93425)");
    assert_eq!(evaluate(1.0175).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.0175)");
    assert_eq!(evaluate(1.85).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.85)");
    assert_eq!(evaluate(0.4875).map(f64::to_bits), Ok(0x40634277ced91687), "at (0.4875)");
    assert_eq!(evaluate(0.8775).map(f64::to_bits), Ok(0x4051c4a0f9096bb9), "at (0.8775)");
    assert_eq!(evaluate(0.9652499999999999).map(f64::to_bits), Ok(0x40511a525edd052a), "at (0.9652499999999999)");
    assert_eq!(evaluate(0.975).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (0.975)");
    assert_eq!(evaluate(0.98475).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (0.98475)");
    assert_eq!(evaluate(1.0725).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.0725)");
    assert_eq!(evaluate(1.95).map(f64::to_bits), Ok(0x4050e9d7dbf487fd), "at (1.95)");
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
            assert!(v.get().is_finite(), "sw_mean_cycle_level produced a value that is not a number for F107_cyc");
            assert!(v.get() >= 60.0 && v.get() <= 400.0, "sw_mean_cycle_level answered {} for F107_cyc, outside its declared domain 60 … 400 — the guard did not stop it", v.get());
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
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_mean_cycle_level is not deterministic for F107_cyc: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_mean_cycle_level refused on one call and answered on the other"),
    }
}

