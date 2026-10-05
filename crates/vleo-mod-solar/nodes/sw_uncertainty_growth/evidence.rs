// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_uncertainty_growth`.
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

/// lead 183 days (0.50 yr) — 9947 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(15811200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 57.0);
    assert!(err <= 1e-12, "lead 183 days (0.50 yr) — 9947 observed pairs: got {} want 57.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 365 days (1.00 yr) — 9675 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(31536000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 68.3);
    assert!(err <= 1e-12, "lead 365 days (1.00 yr) — 9675 observed pairs: got {} want 68.3, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 548 days (1.50 yr) — 9492 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(47347200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 77.0);
    assert!(err <= 1e-12, "lead 548 days (1.50 yr) — 9492 observed pairs: got {} want 77.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 730 days (2.00 yr) — 9310 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(63072000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 91.0);
    assert!(err <= 1e-12, "lead 730 days (2.00 yr) — 9310 observed pairs: got {} want 91.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 1096 days (3.00 yr) — 8946 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(94694400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 107.0);
    assert!(err <= 1e-12, "lead 1096 days (3.00 yr) — 8946 observed pairs: got {} want 107.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 1461 days (4.00 yr) — 8579 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Time::new(126230400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 114.0);
    assert!(err <= 1e-12, "lead 1461 days (4.00 yr) — 8579 observed pairs: got {} want 114.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 1826 days (5.00 yr) — 8215 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Time::new(157766400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 113.3);
    assert!(err <= 1e-12, "lead 1826 days (5.00 yr) — 8215 observed pairs: got {} want 113.3, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 2191 days (6.00 yr) — 7849 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Time::new(189302400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 104.0);
    assert!(err <= 1e-12, "lead 2191 days (6.00 yr) — 7849 observed pairs: got {} want 104.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 2557 days (7.00 yr) — 7483 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Time::new(220924800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 92.0);
    assert!(err <= 1e-12, "lead 2557 days (7.00 yr) — 7483 observed pairs: got {} want 92.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 2922 days (8.00 yr) — 7118 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Time::new(252460800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 88.0);
    assert!(err <= 1e-12, "lead 2922 days (8.00 yr) — 7118 observed pairs: got {} want 88.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 3287 days (9.00 yr) — 7026 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Time::new(283996800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 78.0);
    assert!(err <= 1e-12, "lead 3287 days (9.00 yr) — 7026 observed pairs: got {} want 78.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 3653 days (10.00 yr) — 6660 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_11() {
    let got = model::evaluate(Time::new(315619200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 66.0);
    assert!(err <= 1e-12, "lead 3653 days (10.00 yr) — 6660 observed pairs: got {} want 66.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 4018 days (11.00 yr) — 6295 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_12() {
    let got = model::evaluate(Time::new(347155200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 66.0);
    assert!(err <= 1e-12, "lead 4018 days (11.00 yr) — 6295 observed pairs: got {} want 66.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 4383 days (12.00 yr) — 5932 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_13() {
    let got = model::evaluate(Time::new(378691200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 76.0);
    assert!(err <= 1e-12, "lead 4383 days (12.00 yr) — 5932 observed pairs: got {} want 76.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 4748 days (13.00 yr) — 5567 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_14() {
    let got = model::evaluate(Time::new(410227200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 91.0);
    assert!(err <= 1e-12, "lead 4748 days (13.00 yr) — 5567 observed pairs: got {} want 91.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 5113 days (14.00 yr) — 5203 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_15() {
    let got = model::evaluate(Time::new(441763200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 106.0);
    assert!(err <= 1e-12, "lead 5113 days (14.00 yr) — 5203 observed pairs: got {} want 106.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 5478 days (15.00 yr) — 4838 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_16() {
    let got = model::evaluate(Time::new(473299200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 115.0);
    assert!(err <= 1e-12, "lead 5478 days (15.00 yr) — 4838 observed pairs: got {} want 115.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// below the first measured lead — 100 days holds the 183-day value rather than extrapolating
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_17() {
    let got = model::evaluate(Time::new(8640000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 57.0);
    assert!(err <= 1e-12, "below the first measured lead — 100 days holds the 183-day value rather than extrapolating: got {} want 57.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// above the last measured lead — 20 years holds the 15-year value
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_18() {
    let got = model::evaluate(Time::new(631152000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 115.0);
    assert!(err <= 1e-12, "above the last measured lead — 20 years holds the 15-year value: got {} want 115.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 183 days (0.50 yr) — 9947 observed pairs.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Time::new(15811200.0)).expect("lead 183 days (0.50 yr) — 9947 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 57.0);
    assert!(err <= 1e-12, "lead 183 days (0.50 yr) — 9947 observed pairs.: got {} and the author's code gave 57.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 365 days (1.00 yr) — 9675 observed pairs.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Time::new(31536000.0)).expect("lead 365 days (1.00 yr) — 9675 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 68.3);
    assert!(err <= 1e-12, "lead 365 days (1.00 yr) — 9675 observed pairs.: got {} and the author's code gave 68.3; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 548 days (1.50 yr) — 9492 observed pairs.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Time::new(47347200.0)).expect("lead 548 days (1.50 yr) — 9492 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 77.0);
    assert!(err <= 1e-12, "lead 548 days (1.50 yr) — 9492 observed pairs.: got {} and the author's code gave 77.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 730 days (2.00 yr) — 9310 observed pairs.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Time::new(63072000.0)).expect("lead 730 days (2.00 yr) — 9310 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 91.0);
    assert!(err <= 1e-12, "lead 730 days (2.00 yr) — 9310 observed pairs.: got {} and the author's code gave 91.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 1096 days (3.00 yr) — 8946 observed pairs.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Time::new(94694400.0)).expect("lead 1096 days (3.00 yr) — 8946 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 107.0);
    assert!(err <= 1e-12, "lead 1096 days (3.00 yr) — 8946 observed pairs.: got {} and the author's code gave 107.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 1461 days (4.00 yr) — 8579 observed pairs.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Time::new(126230400.0)).expect("lead 1461 days (4.00 yr) — 8579 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 114.0);
    assert!(err <= 1e-12, "lead 1461 days (4.00 yr) — 8579 observed pairs.: got {} and the author's code gave 114.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 1826 days (5.00 yr) — 8215 observed pairs.», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Time::new(157766400.0)).expect("lead 1826 days (5.00 yr) — 8215 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 113.3);
    assert!(err <= 1e-12, "lead 1826 days (5.00 yr) — 8215 observed pairs.: got {} and the author's code gave 113.3; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 2191 days (6.00 yr) — 7849 observed pairs.», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Time::new(189302400.0)).expect("lead 2191 days (6.00 yr) — 7849 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 104.0);
    assert!(err <= 1e-12, "lead 2191 days (6.00 yr) — 7849 observed pairs.: got {} and the author's code gave 104.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 2557 days (7.00 yr) — 7483 observed pairs.», from their own  code.
#[test]
fn case_9() {
    let got = model::evaluate(Time::new(220924800.0)).expect("lead 2557 days (7.00 yr) — 7483 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 92.0);
    assert!(err <= 1e-12, "lead 2557 days (7.00 yr) — 7483 observed pairs.: got {} and the author's code gave 92.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 2922 days (8.00 yr) — 7118 observed pairs.», from their own  code.
#[test]
fn case_10() {
    let got = model::evaluate(Time::new(252460800.0)).expect("lead 2922 days (8.00 yr) — 7118 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 88.0);
    assert!(err <= 1e-12, "lead 2922 days (8.00 yr) — 7118 observed pairs.: got {} and the author's code gave 88.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 3287 days (9.00 yr) — 7026 observed pairs.», from their own  code.
#[test]
fn case_11() {
    let got = model::evaluate(Time::new(283996800.0)).expect("lead 3287 days (9.00 yr) — 7026 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 78.0);
    assert!(err <= 1e-12, "lead 3287 days (9.00 yr) — 7026 observed pairs.: got {} and the author's code gave 78.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 3653 days (10.00 yr) — 6660 observed pairs.», from their own  code.
#[test]
fn case_12() {
    let got = model::evaluate(Time::new(315619200.0)).expect("lead 3653 days (10.00 yr) — 6660 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 66.0);
    assert!(err <= 1e-12, "lead 3653 days (10.00 yr) — 6660 observed pairs.: got {} and the author's code gave 66.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 4018 days (11.00 yr) — 6295 observed pairs.», from their own  code.
#[test]
fn case_13() {
    let got = model::evaluate(Time::new(347155200.0)).expect("lead 4018 days (11.00 yr) — 6295 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 66.0);
    assert!(err <= 1e-12, "lead 4018 days (11.00 yr) — 6295 observed pairs.: got {} and the author's code gave 66.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 4383 days (12.00 yr) — 5932 observed pairs.», from their own  code.
#[test]
fn case_14() {
    let got = model::evaluate(Time::new(378691200.0)).expect("lead 4383 days (12.00 yr) — 5932 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 76.0);
    assert!(err <= 1e-12, "lead 4383 days (12.00 yr) — 5932 observed pairs.: got {} and the author's code gave 76.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 4748 days (13.00 yr) — 5567 observed pairs.», from their own  code.
#[test]
fn case_15() {
    let got = model::evaluate(Time::new(410227200.0)).expect("lead 4748 days (13.00 yr) — 5567 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 91.0);
    assert!(err <= 1e-12, "lead 4748 days (13.00 yr) — 5567 observed pairs.: got {} and the author's code gave 91.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 5113 days (14.00 yr) — 5203 observed pairs.», from their own  code.
#[test]
fn case_16() {
    let got = model::evaluate(Time::new(441763200.0)).expect("lead 5113 days (14.00 yr) — 5203 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 106.0);
    assert!(err <= 1e-12, "lead 5113 days (14.00 yr) — 5203 observed pairs.: got {} and the author's code gave 106.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 5478 days (15.00 yr) — 4838 observed pairs.», from their own  code.
#[test]
fn case_17() {
    let got = model::evaluate(Time::new(473299200.0)).expect("lead 5478 days (15.00 yr) — 4838 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 115.0);
    assert!(err <= 1e-12, "lead 5478 days (15.00 yr) — 4838 observed pairs.: got {} and the author's code gave 115.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «below the first measured lead — 100 days holds the 183-day value rather than extrapolating.», from their own  code.
#[test]
fn case_18() {
    let got = model::evaluate(Time::new(8640000.0)).expect("below the first measured lead — 100 days holds the 183-day value rather than extrapolating.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 57.0);
    assert!(err <= 1e-12, "below the first measured lead — 100 days holds the 183-day value rather than extrapolating.: got {} and the author's code gave 57.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «above the last measured lead — 20 years holds the 15-year value.», from their own  code.
#[test]
fn case_19() {
    let got = model::evaluate(Time::new(631152000.0)).expect("above the last measured lead — 20 years holds the 15-year value.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 115.0);
    assert!(err <= 1e-12, "above the last measured lead — 20 years holds the 15-year value.: got {} and the author's code gave 115.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_20() {
    let got = model::evaluate(Time::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 134 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_uncertainty_growth::evaluate;
    assert_eq!(evaluate(7905600.0).map(f64::to_bits), Ok(0x404c800000000000), "at (7905600.0)");
    assert_eq!(evaluate(14230080.0).map(f64::to_bits), Ok(0x404c800000000000), "at (14230080.0)");
    assert_eq!(evaluate(15653088.0).map(f64::to_bits), Ok(0x404c800000000000), "at (15653088.0)");
    assert_eq!(evaluate(15811200.0).map(f64::to_bits), Ok(0x404c800000000000), "at (15811200.0)");
    assert_eq!(evaluate(15969312.0).map(f64::to_bits), Ok(0x404c8e8b2103fbc5), "at (15969312.0)");
    assert_eq!(evaluate(17392320.0).map(f64::to_bits), Ok(0x404d116f4a27d5b1), "at (17392320.0)");
    assert_eq!(evaluate(31622400.0).map(f64::to_bits), Ok(0x4051163e1c89a707), "at (31622400.0)");
    assert_eq!(evaluate(15768000.0).map(f64::to_bits), Ok(0x404c800000000000), "at (15768000.0)");
    assert_eq!(evaluate(28382400.0).map(f64::to_bits), Ok(0x40508229a29a29a2), "at (28382400.0)");
    assert_eq!(evaluate(31220640.0).map(f64::to_bits), Ok(0x405104b23e5718a4), "at (31220640.0)");
    assert_eq!(evaluate(31536000.0).map(f64::to_bits), Ok(0x4051133333333333), "at (31536000.0)");
    assert_eq!(evaluate(31851360.0).map(f64::to_bits), Ok(0x40511e4e3a1526c7), "at (31851360.0)");
    assert_eq!(evaluate(34689600.0).map(f64::to_bits), Ok(0x405182417806b6fa), "at (34689600.0)");
    assert_eq!(evaluate(63072000.0).map(f64::to_bits), Ok(0x4056c00000000000), "at (63072000.0)");
    assert_eq!(evaluate(23673600.0).map(f64::to_bits), Ok(0x404f533333333333), "at (23673600.0)");
    assert_eq!(evaluate(42612480.0).map(f64::to_bits), Ok(0x40529943a6b19af1), "at (42612480.0)");
    assert_eq!(evaluate(46873728.0).map(f64::to_bits), Ok(0x40532f5390ab5c4b), "at (46873728.0)");
    assert_eq!(evaluate(47347200.0).map(f64::to_bits), Ok(0x4053400000000000), "at (47347200.0)");
    assert_eq!(evaluate(47820672.0).map(f64::to_bits), Ok(0x40535afa7c749416), "at (47820672.0)");
    assert_eq!(evaluate(52081920.00000001).map(f64::to_bits), Ok(0x40544dc8dc8dc8dd), "at (52081920.00000001)");
    assert_eq!(evaluate(94694400.0).map(f64::to_bits), Ok(0x405ac00000000000), "at (94694400.0)");
    assert_eq!(evaluate(31536000.0).map(f64::to_bits), Ok(0x4051133333333333), "at (31536000.0)");
    assert_eq!(evaluate(56764800.0).map(f64::to_bits), Ok(0x4055589d89d89d8a), "at (56764800.0)");
    assert_eq!(evaluate(62441280.0).map(f64::to_bits), Ok(0x40569c0fc0fc0fc1), "at (62441280.0)");
    assert_eq!(evaluate(63072000.0).map(f64::to_bits), Ok(0x4056c00000000000), "at (63072000.0)");
    assert_eq!(evaluate(63702720.0).map(f64::to_bits), Ok(0x4056d46c8e2103a3), "at (63702720.0)");
    assert_eq!(evaluate(69379200.0).map(f64::to_bits), Ok(0x40578c3d8d4a245f), "at (69379200.0)");
    assert_eq!(evaluate(126144000.0).map(f64::to_bits), Ok(0x405c7ec5c94b0bec), "at (126144000.0)");
    assert_eq!(evaluate(47347200.0).map(f64::to_bits), Ok(0x4053400000000000), "at (47347200.0)");
    assert_eq!(evaluate(85224960.0).map(f64::to_bits), Ok(0x40598d5c0c4f753a), "at (85224960.0)");
    assert_eq!(evaluate(93747456.0).map(f64::to_bits), Ok(0x405aa156013b2553), "at (93747456.0)");
    assert_eq!(evaluate(94694400.0).map(f64::to_bits), Ok(0x405ac00000000000), "at (94694400.0)");
    assert_eq!(evaluate(95641344.0).map(f64::to_bits), Ok(0x405acd73c83a2129), "at (95641344.0)");
    assert_eq!(evaluate(104163840.00000001).map(f64::to_bits), Ok(0x405b4685d2454b9c), "at (104163840.00000001)");
    assert_eq!(evaluate(189388800.0).map(f64::to_bits), Ok(0x4059fde6d1d60865), "at (189388800.0)");
    assert_eq!(evaluate(63115200.0).map(f64::to_bits), Ok(0x4056c1661ec6a512), "at (63115200.0)");
    assert_eq!(evaluate(113607360.0).map(f64::to_bits), Ok(0x405bccad60ede798), "at (113607360.0)");
    assert_eq!(evaluate(124968096.0).map(f64::to_bits), Ok(0x405c6e11567e30c2), "at (124968096.0)");
    assert_eq!(evaluate(126230400.0).map(f64::to_bits), Ok(0x405c800000000000), "at (126230400.0)");
    assert_eq!(evaluate(127492704.0).map(f64::to_bits), Ok(0x405c7e34ef0c9e7a), "at (127492704.0)");
    assert_eq!(evaluate(138853440.0).map(f64::to_bits), Ok(0x405c6e11567e30c2), "at (138853440.0)");
    assert_eq!(evaluate(252460800.0).map(f64::to_bits), Ok(0x4056000000000000), "at (252460800.0)");
    assert_eq!(evaluate(78883200.0).map(f64::to_bits), Ok(0x4058c00000000000), "at (78883200.0)");
    assert_eq!(evaluate(141989760.0).map(f64::to_bits), Ok(0x405c699cbdfcb085), "at (141989760.0)");
    assert_eq!(evaluate(156188736.0).map(f64::to_bits), Ok(0x405c5570f4475955), "at (156188736.0)");
    assert_eq!(evaluate(157766400.0).map(f64::to_bits), Ok(0x405c533333333333), "at (157766400.0)");
    assert_eq!(evaluate(159344064.0).map(f64::to_bits), Ok(0x405c356c772781bb), "at (159344064.0)");
    assert_eq!(evaluate(173543040.0).map(f64::to_bits), Ok(0x405b296fdabe4482), "at (173543040.0)");
    assert_eq!(evaluate(315532800.0).map(f64::to_bits), Ok(0x405082192e29f79b), "at (315532800.0)");
    assert_eq!(evaluate(94651200.0).map(f64::to_bits), Ok(0x405abe99e1395aee), "at (94651200.0)");
    assert_eq!(evaluate(170372160.0).map(f64::to_bits), Ok(0x405b6548772d409c), "at (170372160.0)");
    assert_eq!(evaluate(187409376.0).map(f64::to_bits), Ok(0x405a23ba72515343), "at (187409376.0)");
    assert_eq!(evaluate(189302400.0).map(f64::to_bits), Ok(0x405a000000000000), "at (189302400.0)");
    assert_eq!(evaluate(191195424.0).map(f64::to_bits), Ok(0x4059d206610737e6), "at (191195424.0)");
    assert_eq!(evaluate(208232640.00000003).map(f64::to_bits), Ok(0x4058343fca482f00), "at (208232640.00000003)");
    assert_eq!(evaluate(378604800.0).map(f64::to_bits), Ok(0x4052fe3f1f8fc7e4), "at (378604800.0)");
    assert_eq!(evaluate(110462400.0).map(f64::to_bits), Ok(0x405ba00000000000), "at (110462400.0)");
    assert_eq!(evaluate(198832320.0).map(f64::to_bits), Ok(0x4059188d02849dcc), "at (198832320.0)");
    assert_eq!(evaluate(218715552.0).map(f64::to_bits), Ok(0x405735a7b373a961), "at (218715552.0)");
    assert_eq!(evaluate(220924800.0).map(f64::to_bits), Ok(0x4057000000000000), "at (220924800.0)");
    assert_eq!(evaluate(223134048.0).map(f64::to_bits), Ok(0x4056ee10e3949b33), "at (223134048.0)");
    assert_eq!(evaluate(243017280.00000003).map(f64::to_bits), Ok(0x40564ca8e3ce0ffe), "at (243017280.00000003)");
    assert_eq!(evaluate(441849600.0).map(f64::to_bits), Ok(0x405a8193fd31cc19), "at (441849600.0)");
    assert_eq!(evaluate(126230400.0).map(f64::to_bits), Ok(0x405c800000000000), "at (126230400.0)");
    assert_eq!(evaluate(227214720.0).map(f64::to_bits), Ok(0x4056ccf0b5cb899c), "at (227214720.0)");
    assert_eq!(evaluate(249936192.0).map(f64::to_bits), Ok(0x4056147e78945a90), "at (249936192.0)");
    assert_eq!(evaluate(252460800.0).map(f64::to_bits), Ok(0x4056000000000000), "at (252460800.0)");
    assert_eq!(evaluate(254985408.0).map(f64::to_bits), Ok(0x4055ccc3d28d1d99), "at (254985408.0)");
    assert_eq!(evaluate(277706880.0).map(f64::to_bits), Ok(0x4053ffa6398327fa), "at (277706880.0)");
    assert_eq!(evaluate(504921600.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (504921600.0)");
    assert_eq!(evaluate(141998400.0).map(f64::to_bits), Ok(0x405c69999999999a), "at (141998400.0)");
    assert_eq!(evaluate(255597120.0).map(f64::to_bits), Ok(0x4055c059c67cd806), "at (255597120.0)");
    assert_eq!(evaluate(281156832.0).map(f64::to_bits), Ok(0x4053b9a293d948cd), "at (281156832.0)");
    assert_eq!(evaluate(283996800.0).map(f64::to_bits), Ok(0x4053800000000000), "at (283996800.0)");
    assert_eq!(evaluate(286836768.0).map(f64::to_bits), Ok(0x40533b06e1f393e4), "at (286836768.0)");
    assert_eq!(evaluate(312396480.0).map(f64::to_bits), Ok(0x4050ce44d383c6ed), "at (312396480.0)");
    assert_eq!(evaluate(567993600.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (567993600.0)");
    assert_eq!(evaluate(157809600.0).map(f64::to_bits), Ok(0x405c526278ea89c0), "at (157809600.0)");
    assert_eq!(evaluate(284057280.0).map(f64::to_bits), Ok(0x40537e87f94905e0), "at (284057280.0)");
    assert_eq!(evaluate(312463008.0).map(f64::to_bits), Ok(0x4050cca732874d63), "at (312463008.0)");
    assert_eq!(evaluate(315619200.0).map(f64::to_bits), Ok(0x4050800000000000), "at (315619200.0)");
    assert_eq!(evaluate(318775392.0).map(f64::to_bits), Ok(0x4050800000000000), "at (318775392.0)");
    assert_eq!(evaluate(347181120.0).map(f64::to_bits), Ok(0x40508086a9bb4408), "at (347181120.0)");
    assert_eq!(evaluate(631238400.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (631238400.0)");
    assert_eq!(evaluate(173577600.0).map(f64::to_bits), Ok(0x405b28c8df50f026), "at (173577600.0)");
    assert_eq!(evaluate(312439680.0).map(f64::to_bits), Ok(0x4050cd383c6ecb1f), "at (312439680.0)");
    assert_eq!(evaluate(343683648.0).map(f64::to_bits), Ok(0x4050800000000000), "at (343683648.0)");
    assert_eq!(evaluate(347155200.0).map(f64::to_bits), Ok(0x4050800000000000), "at (347155200.0)");
    assert_eq!(evaluate(350626752.0).map(f64::to_bits), Ok(0x4050c673ddc5ed34), "at (350626752.0)");
    assert_eq!(evaluate(381870720.00000006).map(f64::to_bits), Ok(0x405360c9fe98e60f), "at (381870720.00000006)");
    assert_eq!(evaluate(694310400.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (694310400.0)");
    assert_eq!(evaluate(189345600.0).map(f64::to_bits), Ok(0x4059fef368eb0432), "at (189345600.0)");
    assert_eq!(evaluate(340822080.0).map(f64::to_bits), Ok(0x4050800000000000), "at (340822080.0)");
    assert_eq!(evaluate(374904288.0).map(f64::to_bits), Ok(0x4052b325bbd3ac66), "at (374904288.0)");
    assert_eq!(evaluate(378691200.0).map(f64::to_bits), Ok(0x4053000000000000), "at (378691200.0)");
    assert_eq!(evaluate(382478112.0).map(f64::to_bits), Ok(0x4053734766427d68), "at (382478112.0)");
    assert_eq!(evaluate(416560320.00000006).map(f64::to_bits), Ok(0x405780c9fe98e60f), "at (416560320.00000006)");
    assert_eq!(evaluate(757382400.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (757382400.0)");
    assert_eq!(evaluate(205113600.0).map(f64::to_bits), Ok(0x4058800000000000), "at (205113600.0)");
    assert_eq!(evaluate(369204480.0).map(f64::to_bits), Ok(0x40523f795644bbf8), "at (369204480.0)");
    assert_eq!(evaluate(406124928.0).map(f64::to_bits), Ok(0x4056431f0023e8ff), "at (406124928.0)");
    assert_eq!(evaluate(410227200.0).map(f64::to_bits), Ok(0x4056c00000000000), "at (410227200.0)");
    assert_eq!(evaluate(414329472.0).map(f64::to_bits), Ok(0x40573ce0ffdc1701), "at (414329472.0)");
    assert_eq!(evaluate(451249920.00000006).map(f64::to_bits), Ok(0x405b2d45ff288a09), "at (451249920.00000006)");
    assert_eq!(evaluate(820454400.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (820454400.0)");
    assert_eq!(evaluate(220881600.0).map(f64::to_bits), Ok(0x4057010c9714fbce), "at (220881600.0)");
    assert_eq!(evaluate(397586880.0).map(f64::to_bits), Ok(0x40553f36016719f3), "at (397586880.0)");
    assert_eq!(evaluate(437345568.0).map(f64::to_bits), Ok(0x4059f985668a4f65), "at (437345568.0)");
    assert_eq!(evaluate(441763200.0).map(f64::to_bits), Ok(0x405a800000000000), "at (441763200.0)");
    assert_eq!(evaluate(446180832.0).map(f64::to_bits), Ok(0x405ad0aff5ad0390), "at (446180832.0)");
    assert_eq!(evaluate(485939520.00000006).map(f64::to_bits), Ok(0x405cc00000000000), "at (485939520.00000006)");
    assert_eq!(evaluate(883526400.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (883526400.0)");
    assert_eq!(evaluate(236649600.0).map(f64::to_bits), Ok(0x40568059c67cd806), "at (236649600.0)");
    assert_eq!(evaluate(425969280.0).map(f64::to_bits), Ok(0x40589f36016719f3), "at (425969280.0)");
    assert_eq!(evaluate(468566208.0).map(f64::to_bits), Ok(0x405c698d7af6d37a), "at (468566208.0)");
    assert_eq!(evaluate(473299200.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (473299200.0)");
    assert_eq!(evaluate(478032192.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (478032192.0)");
    assert_eq!(evaluate(520629120.00000006).map(f64::to_bits), Ok(0x405cc00000000000), "at (520629120.00000006)");
    assert_eq!(evaluate(946598400.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (946598400.0)");
    assert_eq!(evaluate(4320000.0).map(f64::to_bits), Ok(0x404c800000000000), "at (4320000.0)");
    assert_eq!(evaluate(7776000.0).map(f64::to_bits), Ok(0x404c800000000000), "at (7776000.0)");
    assert_eq!(evaluate(8553600.0).map(f64::to_bits), Ok(0x404c800000000000), "at (8553600.0)");
    assert_eq!(evaluate(8640000.0).map(f64::to_bits), Ok(0x404c800000000000), "at (8640000.0)");
    assert_eq!(evaluate(8726400.0).map(f64::to_bits), Ok(0x404c800000000000), "at (8726400.0)");
    assert_eq!(evaluate(9504000.0).map(f64::to_bits), Ok(0x404c800000000000), "at (9504000.0)");
    assert_eq!(evaluate(17280000.0).map(f64::to_bits), Ok(0x404d071a71a71a72), "at (17280000.0)");
    assert_eq!(evaluate(315576000.0).map(f64::to_bits), Ok(0x4050810c9714fbce), "at (315576000.0)");
    assert_eq!(evaluate(568036800.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (568036800.0)");
    assert_eq!(evaluate(624840480.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (624840480.0)");
    assert_eq!(evaluate(631152000.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (631152000.0)");
    assert_eq!(evaluate(637463520.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (637463520.0)");
    assert_eq!(evaluate(694267200.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (694267200.0)");
    assert_eq!(evaluate(1262304000.0).map(f64::to_bits), Ok(0x405cc00000000000), "at (1262304000.0)");
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
/// Derived from `lead 183 days (0.50 yr) — 9947 observed pairs` and the declared domain 0 … 120.
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
        if let Err(f) = model::evaluate(Time::new(15811200.0 * scale)) {
            refused.push(format!("lead x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_uncertainty_growth refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 120 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Time::new(15811200.0 * scale)) {
            assert!(v.get().is_finite(), "sw_uncertainty_growth produced a value that is not a number for dF107_p95");
            assert!(v.get() >= 0.0 && v.get() <= 120.0, "sw_uncertainty_growth answered {} for dF107_p95, outside its declared domain 0 … 120 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Time::new(15811200.0));
    let b = model::evaluate(Time::new(15811200.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_uncertainty_growth is not deterministic for dF107_p95: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_uncertainty_growth refused on one call and answered on the other"),
    }
}

