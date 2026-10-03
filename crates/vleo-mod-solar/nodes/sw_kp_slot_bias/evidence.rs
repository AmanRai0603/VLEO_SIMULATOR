// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_kp_slot_bias`.
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

/// Ap bin 0-5, centre 2.5 — 2529 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(2.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "Ap bin 0-5, centre 2.5 — 2529 days: got {} want 1.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 5-10, centre 7.5 — 3938 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Ratio::new(7.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.8333333333333333);
    assert!(err <= 1e-12, "Ap bin 5-10, centre 7.5 — 3938 days: got {} want 0.8333333333333333, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 10-15, centre 12.5 — 1836 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Ratio::new(12.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.114406779661017);
    assert!(err <= 1e-12, "Ap bin 10-15, centre 12.5 — 1836 days: got {} want 1.114406779661017, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 15-20, centre 17.5 — 839 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Ratio::new(17.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "Ap bin 15-20, centre 17.5 — 839 days: got {} want 1.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 20-30, centre 25 — 707 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Ratio::new(25.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.2);
    assert!(err <= 1e-12, "Ap bin 20-30, centre 25 — 707 days: got {} want 1.2, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 30-45, centre 37.5 — 288 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Ratio::new(37.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.333333333333333);
    assert!(err <= 1e-12, "Ap bin 30-45, centre 37.5 — 288 days: got {} want 1.333333333333333, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 45-70, centre 57.5 — 115 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Ratio::new(57.5)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.545454545454545);
    assert!(err <= 1e-12, "Ap bin 45-70, centre 57.5 — 115 days: got {} want 1.545454545454545, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 70-110, centre 90 — 25 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Ratio::new(90.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.746913580246914);
    assert!(err <= 1e-12, "Ap bin 70-110, centre 90 — 25 days: got {} want 1.746913580246914, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// Ap bin 110-400, centre 255 — 20 days
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Ratio::new(255.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.317460317460317);
    assert!(err <= 1e-12, "Ap bin 110-400, centre 255 — 20 days: got {} want 1.317460317460317, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// below the first bin centre — Ap 0 holds the 0-to-5 bin rather than extrapolating
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Ratio::new(0.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "below the first bin centre — Ap 0 holds the 0-to-5 bin rather than extrapolating: got {} want 1.0, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// above the last bin centre — Ap 1000 holds the 110-to-400 bin
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Ratio::new(1000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.3174603174603174);
    assert!(err <= 1e-12, "above the last bin centre — Ap 1000 holds the 110-to-400 bin: got {} want 1.3174603174603174, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap bin 0-5, centre 2.5 — 2529 days.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Ratio::new(2.5)).expect("Ap bin 0-5, centre 2.5 — 2529 days.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "Ap bin 0-5, centre 2.5 — 2529 days.: got {} and the author's code gave 1.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap bin 5-10, centre 7.5 — 3938 days.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Ratio::new(7.5)).expect("Ap bin 5-10, centre 7.5 — 3938 days.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 0.8333333333333333);
    assert!(err <= 1e-12, "Ap bin 5-10, centre 7.5 — 3938 days.: got {} and the author's code gave 0.8333333333333333; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap bin 10-15, centre 12.5 — 1836 days.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Ratio::new(12.5)).expect("Ap bin 10-15, centre 12.5 — 1836 days.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.114406779661017);
    assert!(err <= 1e-12, "Ap bin 10-15, centre 12.5 — 1836 days.: got {} and the author's code gave 1.114406779661017; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap bin 15-20, centre 17.5 — 839 days.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Ratio::new(17.5)).expect("Ap bin 15-20, centre 17.5 — 839 days.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "Ap bin 15-20, centre 17.5 — 839 days.: got {} and the author's code gave 1.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap bin 20-30, centre 25 — 707 days.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Ratio::new(25.0)).expect("Ap bin 20-30, centre 25 — 707 days.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.2);
    assert!(err <= 1e-12, "Ap bin 20-30, centre 25 — 707 days.: got {} and the author's code gave 1.2; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap bin 30-45, centre 37.5 — 288 days.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Ratio::new(37.5)).expect("Ap bin 30-45, centre 37.5 — 288 days.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.333333333333333);
    assert!(err <= 1e-12, "Ap bin 30-45, centre 37.5 — 288 days.: got {} and the author's code gave 1.333333333333333; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap bin 45-70, centre 57.5 — 115 days.», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Ratio::new(57.5)).expect("Ap bin 45-70, centre 57.5 — 115 days.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.545454545454545);
    assert!(err <= 1e-12, "Ap bin 45-70, centre 57.5 — 115 days.: got {} and the author's code gave 1.545454545454545; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap bin 70-110, centre 90 — 25 days.», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Ratio::new(90.0)).expect("Ap bin 70-110, centre 90 — 25 days.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.746913580246914);
    assert!(err <= 1e-12, "Ap bin 70-110, centre 90 — 25 days.: got {} and the author's code gave 1.746913580246914; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «Ap bin 110-400, centre 255 — 20 days.», from their own  code.
#[test]
fn case_9() {
    let got = model::evaluate(Ratio::new(255.0)).expect("Ap bin 110-400, centre 255 — 20 days.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.317460317460317);
    assert!(err <= 1e-12, "Ap bin 110-400, centre 255 — 20 days.: got {} and the author's code gave 1.317460317460317; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «below the first bin centre — Ap 0 holds the 0-to-5 bin rather than extrapolating.», from their own  code.
#[test]
fn case_10() {
    let got = model::evaluate(Ratio::new(0.0)).expect("below the first bin centre — Ap 0 holds the 0-to-5 bin rather than extrapolating.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.0);
    assert!(err <= 1e-12, "below the first bin centre — Ap 0 holds the 0-to-5 bin rather than extrapolating.: got {} and the author's code gave 1.0; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «above the last bin centre — Ap 1000 holds the 110-to-400 bin.», from their own  code.
#[test]
fn case_11() {
    let got = model::evaluate(Ratio::new(1000.0)).expect("above the last bin centre — Ap 1000 holds the 110-to-400 bin.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 1.3174603174603174);
    assert!(err <= 1e-12, "above the last bin centre — Ap 1000 holds the 110-to-400 bin.: got {} and the author's code gave 1.3174603174603174; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «outside what the node holds for: its code refuses it, and so must the developer's», from their own  code.
#[test]
fn case_12() {
    let got = model::evaluate(Ratio::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "outside what the node holds for: its code refuses it, and so must the developer's: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 78 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_kp_slot_bias::evaluate;
    assert_eq!(evaluate(1.25).map(f64::to_bits), Ok(0x3ff0000000000000), "at (1.25)");
    assert_eq!(evaluate(2.25).map(f64::to_bits), Ok(0x3ff0000000000000), "at (2.25)");
    assert_eq!(evaluate(2.475).map(f64::to_bits), Ok(0x3ff0000000000000), "at (2.475)");
    assert_eq!(evaluate(2.5).map(f64::to_bits), Ok(0x3ff0000000000000), "at (2.5)");
    assert_eq!(evaluate(2.525).map(f64::to_bits), Ok(0x3feff92c5f92c5f9), "at (2.525)");
    assert_eq!(evaluate(2.75).map(f64::to_bits), Ok(0x3fefbbbbbbbbbbbc), "at (2.75)");
    assert_eq!(evaluate(5.0).map(f64::to_bits), Ok(0x3fed555555555555), "at (5.0)");
    assert_eq!(evaluate(3.75).map(f64::to_bits), Ok(0x3feeaaaaaaaaaaaa), "at (3.75)");
    assert_eq!(evaluate(6.75).map(f64::to_bits), Ok(0x3feb777777777777), "at (6.75)");
    assert_eq!(evaluate(7.425).map(f64::to_bits), Ok(0x3feabf258bf258be), "at (7.425)");
    assert_eq!(evaluate(7.5).map(f64::to_bits), Ok(0x3feaaaaaaaaaaaaa), "at (7.5)");
    assert_eq!(evaluate(7.575).map(f64::to_bits), Ok(0x3feacd34790758eb), "at (7.575)");
    assert_eq!(evaluate(8.25).map(f64::to_bits), Ok(0x3fec040cba497933), "at (8.25)");
    assert_eq!(evaluate(15.0).map(f64::to_bits), Ok(0x3ff0ea4e1a08ad8f), "at (15.0)");
    assert_eq!(evaluate(6.25).map(f64::to_bits), Ok(0x3fec000000000000), "at (6.25)");
    assert_eq!(evaluate(11.25).map(f64::to_bits), Ok(0x3ff0b4ca7c6259ac), "at (11.25)");
    assert_eq!(evaluate(12.375).map(f64::to_bits), Ok(0x3ff1b7d408197492), "at (12.375)");
    assert_eq!(evaluate(12.5).map(f64::to_bits), Ok(0x3ff1d49c34115b1e), "at (12.5)");
    assert_eq!(evaluate(12.625).map(f64::to_bits), Ok(0x3ff1c8e5192a85a4), "at (12.625)");
    assert_eq!(evaluate(13.750000000000002).map(f64::to_bits), Ok(0x3ff15f75270d0456), "at (13.750000000000002)");
    assert_eq!(evaluate(25.0).map(f64::to_bits), Ok(0x3ff3333333333333), "at (25.0)");
    assert_eq!(evaluate(8.75).map(f64::to_bits), Ok(0x3fecea4e1a08ad8e), "at (8.75)");
    assert_eq!(evaluate(15.75).map(f64::to_bits), Ok(0x3ff0a403789facb1), "at (15.75)");
    assert_eq!(evaluate(17.325).map(f64::to_bits), Ok(0x3ff01066bf432adf), "at (17.325)");
    assert_eq!(evaluate(17.5).map(f64::to_bits), Ok(0x3ff0000000000000), "at (17.5)");
    assert_eq!(evaluate(17.675).map(f64::to_bits), Ok(0x3ff0131d5acb6f46), "at (17.675)");
    assert_eq!(evaluate(19.25).map(f64::to_bits), Ok(0x3ff0bf258bf258bf), "at (19.25)");
    assert_eq!(evaluate(35.0).map(f64::to_bits), Ok(0x3ff4e81b4e81b4e8), "at (35.0)");
    assert_eq!(evaluate(12.5).map(f64::to_bits), Ok(0x3ff1d49c34115b1e), "at (12.5)");
    assert_eq!(evaluate(22.5).map(f64::to_bits), Ok(0x3ff2222222222222), "at (22.5)");
    assert_eq!(evaluate(24.75).map(f64::to_bits), Ok(0x3ff317e4b17e4b18), "at (24.75)");
    assert_eq!(evaluate(25.0).map(f64::to_bits), Ok(0x3ff3333333333333), "at (25.0)");
    assert_eq!(evaluate(25.25).map(f64::to_bits), Ok(0x3ff33e1f671529a4), "at (25.25)");
    assert_eq!(evaluate(27.500000000000004).map(f64::to_bits), Ok(0x3ff3a06d3a06d3a0), "at (27.500000000000004)");
    assert_eq!(evaluate(50.0).map(f64::to_bits), Ok(0x3ff7745d1745d174), "at (50.0)");
    assert_eq!(evaluate(18.75).map(f64::to_bits), Ok(0x3ff0888888888888), "at (18.75)");
    assert_eq!(evaluate(33.75).map(f64::to_bits), Ok(0x3ff4b17e4b17e4b1), "at (33.75)");
    assert_eq!(evaluate(37.125).map(f64::to_bits), Ok(0x3ff544f3078263ab), "at (37.125)");
    assert_eq!(evaluate(37.5).map(f64::to_bits), Ok(0x3ff5555555555555), "at (37.5)");
    assert_eq!(evaluate(37.875).map(f64::to_bits), Ok(0x3ff5659fce59fce5), "at (37.875)");
    assert_eq!(evaluate(41.25).map(f64::to_bits), Ok(0x3ff5f83e0f83e0f8), "at (41.25)");
    assert_eq!(evaluate(75.0).map(f64::to_bits), Ok(0x3ffa7681e9699403), "at (75.0)");
    assert_eq!(evaluate(28.75).map(f64::to_bits), Ok(0x3ff3d70a3d70a3d7), "at (28.75)");
    assert_eq!(evaluate(51.75).map(f64::to_bits), Ok(0x3ff7c0634c0634c0), "at (51.75)");
    assert_eq!(evaluate(56.925).map(f64::to_bits), Ok(0x3ff8a133d2133d21), "at (56.925)");
    assert_eq!(evaluate(57.5).map(f64::to_bits), Ok(0x3ff8ba2e8ba2e8ba), "at (57.5)");
    assert_eq!(evaluate(58.075).map(f64::to_bits), Ok(0x3ff8c8c7f57bb8f6), "at (58.075)");
    assert_eq!(evaluate(63.25000000000001).map(f64::to_bits), Ok(0x3ff94c2cae1b0b10), "at (63.25000000000001)");
    assert_eq!(evaluate(115.0).map(f64::to_bits), Ok(0x3ffae8d633be99de), "at (115.0)");
    assert_eq!(evaluate(45.0).map(f64::to_bits), Ok(0x3ff69b26c9b26c9b), "at (45.0)");
    assert_eq!(evaluate(81.0).map(f64::to_bits), Ok(0x3ffb0ed9023ffaa0), "at (81.0)");
    assert_eq!(evaluate(89.1).map(f64::to_bits), Ok(0x3ffbdc81ca2e385a), "at (89.1)");
    assert_eq!(evaluate(90.0).map(f64::to_bits), Ok(0x3ffbf35ba781948b), "at (90.0)");
    assert_eq!(evaluate(90.9).map(f64::to_bits), Ok(0x3ffbe9c364dfefdf), "at (90.9)");
    assert_eq!(evaluate(99.00000000000001).map(f64::to_bits), Ok(0x3ffb93690d3125d1), "at (99.00000000000001)");
    assert_eq!(evaluate(180.0).map(f64::to_bits), Ok(0x3ff833e1a05d414e), "at (180.0)");
    assert_eq!(evaluate(127.5).map(f64::to_bits), Ok(0x3ffa639379dd1c87), "at (127.5)");
    assert_eq!(evaluate(229.5).map(f64::to_bits), Ok(0x3ff6242b4fa2e052), "at (229.5)");
    assert_eq!(evaluate(252.45).map(f64::to_bits), Ok(0x3ff52f80ac88f92d), "at (252.45)");
    assert_eq!(evaluate(255.0).map(f64::to_bits), Ok(0x3ff5145145145145), "at (255.0)");
    assert_eq!(evaluate(257.55).map(f64::to_bits), Ok(0x3ff5145145145145), "at (257.55)");
    assert_eq!(evaluate(280.5).map(f64::to_bits), Ok(0x3ff5145145145145), "at (280.5)");
    assert_eq!(evaluate(510.0).map(f64::to_bits), Ok(0x3ff5145145145145), "at (510.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (0.0)");
    assert_eq!(evaluate(0.0).map(f64::to_bits), Ok(0x3ff0000000000000), "at (0.0)");
    assert_eq!(evaluate(500.0).map(f64::to_bits), Ok(0x3ff5145145145145), "at (500.0)");
    assert_eq!(evaluate(900.0).map(f64::to_bits), Ok(0x3ff5145145145145), "at (900.0)");
    assert_eq!(evaluate(990.0).map(f64::to_bits), Ok(0x3ff5145145145145), "at (990.0)");
    assert_eq!(evaluate(1000.0).map(f64::to_bits), Ok(0x3ff5145145145145), "at (1000.0)");
    assert_eq!(evaluate(1010.0).map(f64::to_bits), Ok(0x3ff5145145145145), "at (1010.0)");
    assert_eq!(evaluate(1100.0).map(f64::to_bits), Ok(0x3ff5145145145145), "at (1100.0)");
    assert_eq!(evaluate(2000.0).map(f64::to_bits), Ok(0x3ff5145145145145), "at (2000.0)");
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
/// Derived from `Ap bin 0-5, centre 2.5 — 2529 days` and the declared domain 0 … 2.
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
        if let Err(f) = model::evaluate(Ratio::new(2.5 * scale)) {
            refused.push(format!("ap x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_kp_slot_bias refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 2 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Ratio::new(2.5 * scale)) {
            assert!(v.get().is_finite(), "sw_kp_slot_bias produced a value that is not a number for dKp_peak");
            assert!(v.get() >= 0.0 && v.get() <= 2.0, "sw_kp_slot_bias answered {} for dKp_peak, outside its declared domain 0 … 2 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Ratio::new(2.5));
    let b = model::evaluate(Ratio::new(2.5));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_kp_slot_bias is not deterministic for dKp_peak: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_kp_slot_bias refused on one call and answered on the other"),
    }
}

