// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_horizon_climatology`.
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

/// lead 1 day — 10312 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Time::new(86400.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.3929);
    assert!(err <= 1e-12, "lead 1 day — 10312 observed pairs: got {} want 44.3929, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 2 days — 10309 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(172800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.3953);
    assert!(err <= 1e-12, "lead 2 days — 10309 observed pairs: got {} want 44.3953, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 3 days — 10307 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(259200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.3968);
    assert!(err <= 1e-12, "lead 3 days — 10307 observed pairs: got {} want 44.3968, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 5 days — 10303 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(432000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.3993);
    assert!(err <= 1e-12, "lead 5 days — 10303 observed pairs: got {} want 44.3993, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 7 days — 10299 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(604800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4012);
    assert!(err <= 1e-12, "lead 7 days — 10299 observed pairs: got {} want 44.4012, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 10 days — 10293 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Time::new(864000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4036);
    assert!(err <= 1e-12, "lead 10 days — 10293 observed pairs: got {} want 44.4036, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 14 days — 10285 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Time::new(1209600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4069);
    assert!(err <= 1e-12, "lead 14 days — 10285 observed pairs: got {} want 44.4069, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 20 days — 10273 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Time::new(1728000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4071);
    assert!(err <= 1e-12, "lead 20 days — 10273 observed pairs: got {} want 44.4071, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 27 days — 10259 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Time::new(2332800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4147);
    assert!(err <= 1e-12, "lead 27 days — 10259 observed pairs: got {} want 44.4147, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 40 days — 10233 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Time::new(3456000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4242);
    assert!(err <= 1e-12, "lead 40 days — 10233 observed pairs: got {} want 44.4242, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 60 days — 10193 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Time::new(5184000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4344);
    assert!(err <= 1e-12, "lead 60 days — 10193 observed pairs: got {} want 44.4344, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 90 days — 10133 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_11() {
    let got = model::evaluate(Time::new(7776000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4458);
    assert!(err <= 1e-12, "lead 90 days — 10133 observed pairs: got {} want 44.4458, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 135 days — 10043 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_12() {
    let got = model::evaluate(Time::new(11664000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4635);
    assert!(err <= 1e-12, "lead 135 days — 10043 observed pairs: got {} want 44.4635, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 180 days — 9953 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_13() {
    let got = model::evaluate(Time::new(15552000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.4714);
    assert!(err <= 1e-12, "lead 180 days — 9953 observed pairs: got {} want 44.4714, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 270 days — 9773 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_14() {
    let got = model::evaluate(Time::new(23328000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.5344);
    assert!(err <= 1e-12, "lead 270 days — 9773 observed pairs: got {} want 44.5344, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 365 days — 9675 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_15() {
    let got = model::evaluate(Time::new(31536000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 44.6734);
    assert!(err <= 1e-12, "lead 365 days — 9675 observed pairs: got {} want 44.6734, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 547 days — 9493 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_16() {
    let got = model::evaluate(Time::new(47260800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 45.0333);
    assert!(err <= 1e-12, "lead 547 days — 9493 observed pairs: got {} want 45.0333, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 730 days — 9310 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_17() {
    let got = model::evaluate(Time::new(63072000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 45.3201);
    assert!(err <= 1e-12, "lead 730 days — 9310 observed pairs: got {} want 45.3201, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// beyond the last measured lead — five years holds the two-year value rather than extrapolating
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_18() {
    let got = model::evaluate(Time::new(157788000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 45.3201);
    assert!(err <= 1e-12, "beyond the last measured lead — five years holds the two-year value rather than extrapolating: got {} want 45.3201, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 1 day — 10312 observed pairs.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Time::new(86400.0)).expect("lead 1 day — 10312 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.3929);
    assert!(err <= 1e-12, "lead 1 day — 10312 observed pairs.: got {} and the author's code gave 44.3929; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 2 days — 10309 observed pairs.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Time::new(172800.0)).expect("lead 2 days — 10309 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.3953);
    assert!(err <= 1e-12, "lead 2 days — 10309 observed pairs.: got {} and the author's code gave 44.3953; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 3 days — 10307 observed pairs.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Time::new(259200.0)).expect("lead 3 days — 10307 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.3968);
    assert!(err <= 1e-12, "lead 3 days — 10307 observed pairs.: got {} and the author's code gave 44.3968; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 5 days — 10303 observed pairs.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Time::new(432000.0)).expect("lead 5 days — 10303 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.3993);
    assert!(err <= 1e-12, "lead 5 days — 10303 observed pairs.: got {} and the author's code gave 44.3993; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 7 days — 10299 observed pairs.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Time::new(604800.0)).expect("lead 7 days — 10299 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.4012);
    assert!(err <= 1e-12, "lead 7 days — 10299 observed pairs.: got {} and the author's code gave 44.4012; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 10 days — 10293 observed pairs.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Time::new(864000.0)).expect("lead 10 days — 10293 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.4036);
    assert!(err <= 1e-12, "lead 10 days — 10293 observed pairs.: got {} and the author's code gave 44.4036; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 14 days — 10285 observed pairs.», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Time::new(1209600.0)).expect("lead 14 days — 10285 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.4069);
    assert!(err <= 1e-12, "lead 14 days — 10285 observed pairs.: got {} and the author's code gave 44.4069; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 20 days — 10273 observed pairs.», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Time::new(1728000.0)).expect("lead 20 days — 10273 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.4071);
    assert!(err <= 1e-12, "lead 20 days — 10273 observed pairs.: got {} and the author's code gave 44.4071; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 27 days — 10259 observed pairs.», from their own  code.
#[test]
fn case_9() {
    let got = model::evaluate(Time::new(2332800.0)).expect("lead 27 days — 10259 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.4147);
    assert!(err <= 1e-12, "lead 27 days — 10259 observed pairs.: got {} and the author's code gave 44.4147; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 40 days — 10233 observed pairs.», from their own  code.
#[test]
fn case_10() {
    let got = model::evaluate(Time::new(3456000.0)).expect("lead 40 days — 10233 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.4242);
    assert!(err <= 1e-12, "lead 40 days — 10233 observed pairs.: got {} and the author's code gave 44.4242; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 60 days — 10193 observed pairs.», from their own  code.
#[test]
fn case_11() {
    let got = model::evaluate(Time::new(5184000.0)).expect("lead 60 days — 10193 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.4344);
    assert!(err <= 1e-12, "lead 60 days — 10193 observed pairs.: got {} and the author's code gave 44.4344; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 90 days — 10133 observed pairs.», from their own  code.
#[test]
fn case_12() {
    let got = model::evaluate(Time::new(7776000.0)).expect("lead 90 days — 10133 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.4458);
    assert!(err <= 1e-12, "lead 90 days — 10133 observed pairs.: got {} and the author's code gave 44.4458; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 135 days — 10043 observed pairs.», from their own  code.
#[test]
fn case_13() {
    let got = model::evaluate(Time::new(11664000.0)).expect("lead 135 days — 10043 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.4635);
    assert!(err <= 1e-12, "lead 135 days — 10043 observed pairs.: got {} and the author's code gave 44.4635; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 180 days — 9953 observed pairs.», from their own  code.
#[test]
fn case_14() {
    let got = model::evaluate(Time::new(15552000.0)).expect("lead 180 days — 9953 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.4714);
    assert!(err <= 1e-12, "lead 180 days — 9953 observed pairs.: got {} and the author's code gave 44.4714; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 270 days — 9773 observed pairs.», from their own  code.
#[test]
fn case_15() {
    let got = model::evaluate(Time::new(23328000.0)).expect("lead 270 days — 9773 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.5344);
    assert!(err <= 1e-12, "lead 270 days — 9773 observed pairs.: got {} and the author's code gave 44.5344; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 365 days — 9675 observed pairs.», from their own  code.
#[test]
fn case_16() {
    let got = model::evaluate(Time::new(31536000.0)).expect("lead 365 days — 9675 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 44.6734);
    assert!(err <= 1e-12, "lead 365 days — 9675 observed pairs.: got {} and the author's code gave 44.6734; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 547 days — 9493 observed pairs.», from their own  code.
#[test]
fn case_17() {
    let got = model::evaluate(Time::new(47260800.0)).expect("lead 547 days — 9493 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 45.0333);
    assert!(err <= 1e-12, "lead 547 days — 9493 observed pairs.: got {} and the author's code gave 45.0333; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 730 days — 9310 observed pairs.», from their own  code.
#[test]
fn case_18() {
    let got = model::evaluate(Time::new(63072000.0)).expect("lead 730 days — 9310 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 45.3201);
    assert!(err <= 1e-12, "lead 730 days — 9310 observed pairs.: got {} and the author's code gave 45.3201; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «beyond the last measured lead — five years holds the two-year value rather than extrapolating.», from their own  code.
#[test]
fn case_19() {
    let got = model::evaluate(Time::new(157788000.0)).expect("beyond the last measured lead — five years holds the two-year value rather than extrapolating.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 45.3201);
    assert!(err <= 1e-12, "beyond the last measured lead — five years holds the two-year value rather than extrapolating.: got {} and the author's code gave 45.3201; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
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
    use vleo_core::physics::methods::sw_horizon_climatology::evaluate;
    assert_eq!(evaluate(43200.0).map(f64::to_bits), Ok(0x4046324a8c154c98), "at (43200.0)");
    assert_eq!(evaluate(77760.0).map(f64::to_bits), Ok(0x4046324a8c154c98), "at (77760.0)");
    assert_eq!(evaluate(85536.0).map(f64::to_bits), Ok(0x4046324a8c154c98), "at (85536.0)");
    assert_eq!(evaluate(86400.0).map(f64::to_bits), Ok(0x4046324a8c154c98), "at (86400.0)");
    assert_eq!(evaluate(87264.0).map(f64::to_bits), Ok(0x4046324b5568e821), "at (87264.0)");
    assert_eq!(evaluate(95040.00000000001).map(f64::to_bits), Ok(0x4046325269595fed), "at (95040.00000000001)");
    assert_eq!(evaluate(172800.0).map(f64::to_bits), Ok(0x4046329930be0ded), "at (172800.0)");
    assert_eq!(evaluate(86400.0).map(f64::to_bits), Ok(0x4046324a8c154c98), "at (86400.0)");
    assert_eq!(evaluate(155520.0).map(f64::to_bits), Ok(0x404632897635e742), "at (155520.0)");
    assert_eq!(evaluate(171072.0).map(f64::to_bits), Ok(0x404632979e16d6dc), "at (171072.0)");
    assert_eq!(evaluate(172800.0).map(f64::to_bits), Ok(0x4046329930be0ded), "at (172800.0)");
    assert_eq!(evaluate(174528.0).map(f64::to_bits), Ok(0x4046329a2c669058), "at (174528.0)");
    assert_eq!(evaluate(190080.00000000003).map(f64::to_bits), Ok(0x404632a305532618), "at (190080.00000000003)");
    assert_eq!(evaluate(345600.0).map(f64::to_bits), Ok(0x404632f34d6a161e), "at (345600.0)");
    assert_eq!(evaluate(129600.0).map(f64::to_bits), Ok(0x40463271de69ad42), "at (129600.0)");
    assert_eq!(evaluate(233280.0).map(f64::to_bits), Ok(0x404632bb98c7e282), "at (233280.0)");
    assert_eq!(evaluate(256608.0).map(f64::to_bits), Ok(0x404632c8de2ac322), "at (256608.0)");
    assert_eq!(evaluate(259200.0).map(f64::to_bits), Ok(0x404632ca57a786c2), "at (259200.0)");
    assert_eq!(evaluate(261792.0).map(f64::to_bits), Ok(0x404632cb923a29c7), "at (261792.0)");
    assert_eq!(evaluate(285120.0).map(f64::to_bits), Ok(0x404632d6a161e4f7), "at (285120.0)");
    assert_eq!(evaluate(518400.0).map(f64::to_bits), Ok(0x4046333b645a1cac), "at (518400.0)");
    assert_eq!(evaluate(216000.0).map(f64::to_bits), Ok(0x404632b1c432ca58), "at (216000.0)");
    assert_eq!(evaluate(388800.0).map(f64::to_bits), Ok(0x40463307c84b5dcc), "at (388800.0)");
    assert_eq!(evaluate(427680.0).map(f64::to_bits), Ok(0x4046331a36e2eb1c), "at (427680.0)");
    assert_eq!(evaluate(432000.0).map(f64::to_bits), Ok(0x4046331c432ca57a), "at (432000.0)");
    assert_eq!(evaluate(436320.0).map(f64::to_bits), Ok(0x4046331dd1a21ea3), "at (436320.0)");
    assert_eq!(evaluate(475200.00000000006).map(f64::to_bits), Ok(0x4046332bd3c36113), "at (475200.00000000006)");
    assert_eq!(evaluate(864000.0).map(f64::to_bits), Ok(0x404633a92a305532), "at (864000.0)");
    assert_eq!(evaluate(302400.0).map(f64::to_bits), Ok(0x404632ded288ce70), "at (302400.0)");
    assert_eq!(evaluate(544320.0).map(f64::to_bits), Ok(0x40463344bb1af3a1), "at (544320.0)");
    assert_eq!(evaluate(598752.0).map(f64::to_bits), Ok(0x4046335857afea3e), "at (598752.0)");
    assert_eq!(evaluate(604800.0).map(f64::to_bits), Ok(0x4046335a858793de), "at (604800.0)");
    assert_eq!(evaluate(610848.0).map(f64::to_bits), Ok(0x4046335c5b4aa972), "at (610848.0)");
    assert_eq!(evaluate(665280.0).map(f64::to_bits), Ok(0x4046336cdf266ba5), "at (665280.0)");
    assert_eq!(evaluate(1209600.0).map(f64::to_bits), Ok(0x404634154c985f07), "at (1209600.0)");
    assert_eq!(evaluate(432000.0).map(f64::to_bits), Ok(0x4046331c432ca57a), "at (432000.0)");
    assert_eq!(evaluate(777600.0).map(f64::to_bits), Ok(0x4046338ef34d6a16), "at (777600.0)");
    assert_eq!(evaluate(855360.0).map(f64::to_bits), Ok(0x404633a68b19a416), "at (855360.0)");
    assert_eq!(evaluate(864000.0).map(f64::to_bits), Ok(0x404633a92a305532), "at (864000.0)");
    assert_eq!(evaluate(872640.0).map(f64::to_bits), Ok(0x404633abde3fbbd7), "at (872640.0)");
    assert_eq!(evaluate(950400.0000000001).map(f64::to_bits), Ok(0x404633c432ca57a7), "at (950400.0000000001)");
    assert_eq!(evaluate(1728000.0).map(f64::to_bits), Ok(0x4046341bda5119ce), "at (1728000.0)");
    assert_eq!(evaluate(604800.0).map(f64::to_bits), Ok(0x4046335a858793de), "at (604800.0)");
    assert_eq!(evaluate(1088640.0).map(f64::to_bits), Ok(0x404633ef73c0c1fc), "at (1088640.0)");
    assert_eq!(evaluate(1197504.0).map(f64::to_bits), Ok(0x4046341183b60286), "at (1197504.0)");
    assert_eq!(evaluate(1209600.0).map(f64::to_bits), Ok(0x404634154c985f07), "at (1209600.0)");
    assert_eq!(evaluate(1221696.0).map(f64::to_bits), Ok(0x4046341573bdf629), "at (1221696.0)");
    assert_eq!(evaluate(1330560.0).map(f64::to_bits), Ok(0x40463416d4104658), "at (1330560.0)");
    assert_eq!(evaluate(2419200.0).map(f64::to_bits), Ok(0x4046352cd5dfcc95), "at (2419200.0)");
    assert_eq!(evaluate(864000.0).map(f64::to_bits), Ok(0x404633a92a305532), "at (864000.0)");
    assert_eq!(evaluate(1555200.0).map(f64::to_bits), Ok(0x40463419ab138636), "at (1555200.0)");
    assert_eq!(evaluate(1710720.0).map(f64::to_bits), Ok(0x4046341ba2648b0c), "at (1710720.0)");
    assert_eq!(evaluate(1728000.0).map(f64::to_bits), Ok(0x4046341bda5119ce), "at (1728000.0)");
    assert_eq!(evaluate(1745280.0).map(f64::to_bits), Ok(0x40463422f7d7d5f7), "at (1745280.0)");
    assert_eq!(evaluate(1900800.0000000002).map(f64::to_bits), Ok(0x4046346301947364), "at (1900800.0000000002)");
    assert_eq!(evaluate(3456000.0).map(f64::to_bits), Ok(0x4046364c2f837b4a), "at (3456000.0)");
    assert_eq!(evaluate(1166400.0).map(f64::to_bits), Ok(0x40463407c84b5dcc), "at (1166400.0)");
    assert_eq!(evaluate(2099520.0).map(f64::to_bits), Ok(0x404634b4d521e737), "at (2099520.0)");
    assert_eq!(evaluate(2309472.0).map(f64::to_bits), Ok(0x4046350b48ad5557), "at (2309472.0)");
    assert_eq!(evaluate(2332800.0).map(f64::to_bits), Ok(0x40463514e3bcd35b), "at (2332800.0)");
    assert_eq!(evaluate(2356128.0).map(f64::to_bits), Ok(0x4046351b5adfde54), "at (2356128.0)");
    assert_eq!(evaluate(2566080.0).map(f64::to_bits), Ok(0x404635558b1b4111), "at (2566080.0)");
    assert_eq!(evaluate(4665600.0).map(f64::to_bits), Ok(0x40463736262cba73), "at (4665600.0)");
    assert_eq!(evaluate(1728000.0).map(f64::to_bits), Ok(0x4046341bda5119ce), "at (1728000.0)");
    assert_eq!(evaluate(3110400.0).map(f64::to_bits), Ok(0x404635ec66f79663), "at (3110400.0)");
    assert_eq!(evaluate(3421440.0).map(f64::to_bits), Ok(0x404636429b757e00), "at (3421440.0)");
    assert_eq!(evaluate(3456000.0).map(f64::to_bits), Ok(0x4046364c2f837b4a), "at (3456000.0)");
    assert_eq!(evaluate(3490560.0).map(f64::to_bits), Ok(0x40463652deca2552), "at (3490560.0)");
    assert_eq!(evaluate(3801600.0000000005).map(f64::to_bits), Ok(0x4046368f08461f9f), "at (3801600.0000000005)");
    assert_eq!(evaluate(6912000.0).map(f64::to_bits), Ok(0x4046389374bc6a7f), "at (6912000.0)");
    assert_eq!(evaluate(2592000.0).map(f64::to_bits), Ok(0x4046355cba25bf08), "at (2592000.0)");
    assert_eq!(evaluate(4665600.0).map(f64::to_bits), Ok(0x40463736262cba73), "at (4665600.0)");
    assert_eq!(evaluate(5132160.0).map(f64::to_bits), Ok(0x404637906466b1e5), "at (5132160.0)");
    assert_eq!(evaluate(5184000.0).map(f64::to_bits), Ok(0x4046379a6b50b0f2), "at (5184000.0)");
    assert_eq!(evaluate(5235840.0).map(f64::to_bits), Ok(0x404637a1e3eaf683), "at (5235840.0)");
    assert_eq!(evaluate(5702400.0).map(f64::to_bits), Ok(0x404637e52157689c), "at (5702400.0)");
    assert_eq!(evaluate(10368000.0).map(f64::to_bits), Ok(0x40463a92a3055326), "at (10368000.0)");
    assert_eq!(evaluate(3888000.0).map(f64::to_bits), Ok(0x4046369fbe76c8b4), "at (3888000.0)");
    assert_eq!(evaluate(6998400.0).map(f64::to_bits), Ok(0x4046389fe86833c6), "at (6998400.0)");
    assert_eq!(evaluate(7698240.0).map(f64::to_bits), Ok(0x40463904c48adeeb), "at (7698240.0)");
    assert_eq!(evaluate(7776000.0).map(f64::to_bits), Ok(0x4046390ff9724745), "at (7776000.0)");
    assert_eq!(evaluate(7853760.0).map(f64::to_bits), Ok(0x4046391b93037d63), "at (7853760.0)");
    assert_eq!(evaluate(8553600.0).map(f64::to_bits), Ok(0x40463983f91e646f), "at (8553600.0)");
    assert_eq!(evaluate(15552000.0).map(f64::to_bits), Ok(0x40463c56d5cfaace), "at (15552000.0)");
    assert_eq!(evaluate(5832000.0).map(f64::to_bits), Ok(0x404637f7ced91687), "at (5832000.0)");
    assert_eq!(evaluate(10497600.0).map(f64::to_bits), Ok(0x40463aa5f84cad58), "at (10497600.0)");
    assert_eq!(evaluate(11547360.0).map(f64::to_bits), Ok(0x40463b42917507ea), "at (11547360.0)");
    assert_eq!(evaluate(11664000.0).map(f64::to_bits), Ok(0x40463b53f7ced917), "at (11664000.0)");
    assert_eq!(evaluate(11780640.0).map(f64::to_bits), Ok(0x40463b5bbbe878fb), "at (11780640.0)");
    assert_eq!(evaluate(12830400.000000002).map(f64::to_bits), Ok(0x40463ba1a0cf1801), "at (12830400.000000002)");
    assert_eq!(evaluate(23328000.0).map(f64::to_bits), Ok(0x40464467381d7dbf), "at (23328000.0)");
    assert_eq!(evaluate(7776000.0).map(f64::to_bits), Ok(0x4046390ff9724745), "at (7776000.0)");
    assert_eq!(evaluate(13996800.0).map(f64::to_bits), Ok(0x40463bef49cf56eb), "at (13996800.0)");
    assert_eq!(evaluate(15396480.0).map(f64::to_bits), Ok(0x40463c4c7b02d59e), "at (15396480.0)");
    assert_eq!(evaluate(15552000.0).map(f64::to_bits), Ok(0x40463c56d5cfaace), "at (15552000.0)");
    assert_eq!(evaluate(15707520.0).map(f64::to_bits), Ok(0x40463c801f75104e), "at (15707520.0)");
    assert_eq!(evaluate(17107200.0).map(f64::to_bits), Ok(0x40463df3b645a1cb), "at (17107200.0)");
    assert_eq!(evaluate(31104000.0).map(f64::to_bits), Ok(0x404655423f564bd7), "at (31104000.0)");
    assert_eq!(evaluate(11664000.0).map(f64::to_bits), Ok(0x40463b53f7ced917), "at (11664000.0)");
    assert_eq!(evaluate(20995200.0).map(f64::to_bits), Ok(0x404641fbe76c8b44), "at (20995200.0)");
    assert_eq!(evaluate(23094720.0).map(f64::to_bits), Ok(0x4046442949a5657f), "at (23094720.0)");
    assert_eq!(evaluate(23328000.0).map(f64::to_bits), Ok(0x40464467381d7dbf), "at (23328000.0)");
    assert_eq!(evaluate(23561280.0).map(f64::to_bits), Ok(0x404644e8ab8827c5), "at (23561280.0)");
    assert_eq!(evaluate(25660800.000000004).map(f64::to_bits), Ok(0x40464975ba4821fa), "at (25660800.000000004)");
    assert_eq!(evaluate(46656000.0).map(f64::to_bits), Ok(0x4046827d96f40eff), "at (46656000.0)");
    assert_eq!(evaluate(15768000.0).map(f64::to_bits), Ok(0x40463c902de00d1c), "at (15768000.0)");
    assert_eq!(evaluate(28382400.0).map(f64::to_bits), Ok(0x40464f5bfccf36e8), "at (28382400.0)");
    assert_eq!(evaluate(31220640.0).map(f64::to_bits), Ok(0x40465582f90ba0da), "at (31220640.0)");
    assert_eq!(evaluate(31536000.0).map(f64::to_bits), Ok(0x40465631f8a0902e), "at (31536000.0)");
    assert_eq!(evaluate(31851360.0).map(f64::to_bits), Ok(0x4046571e7bb5c2bc), "at (31851360.0)");
    assert_eq!(evaluate(34689600.0).map(f64::to_bits), Ok(0x40465f6f177489b7), "at (34689600.0)");
    assert_eq!(evaluate(63072000.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (63072000.0)");
    assert_eq!(evaluate(23630400.0).map(f64::to_bits), Ok(0x4046450f069e472f), "at (23630400.0)");
    assert_eq!(evaluate(42534720.0).map(f64::to_bits), Ok(0x4046766abbd1031b), "at (42534720.0)");
    assert_eq!(evaluate(46788192.0).map(f64::to_bits), Ok(0x404682e0baf6a1bb), "at (46788192.0)");
    assert_eq!(evaluate(47260800.0).map(f64::to_bits), Ok(0x404684432ca57a78), "at (47260800.0)");
    assert_eq!(evaluate(47733408.0).map(f64::to_bits), Ok(0x4046855c154b7ebb), "at (47733408.0)");
    assert_eq!(evaluate(51986880.00000001).map(f64::to_bits), Ok(0x40468f3c4321a51b), "at (51986880.00000001)");
    assert_eq!(evaluate(94521600.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (94521600.0)");
    assert_eq!(evaluate(31536000.0).map(f64::to_bits), Ok(0x40465631f8a0902e), "at (31536000.0)");
    assert_eq!(evaluate(56764800.0).map(f64::to_bits), Ok(0x40469a5429a8889a), "at (56764800.0)");
    assert_eq!(evaluate(62441280.0).map(f64::to_bits), Ok(0x4046a782263e9b0d), "at (62441280.0)");
    assert_eq!(evaluate(63072000.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (63072000.0)");
    assert_eq!(evaluate(63702720.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (63702720.0)");
    assert_eq!(evaluate(69379200.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (69379200.0)");
    assert_eq!(evaluate(126144000.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (126144000.0)");
    assert_eq!(evaluate(78894000.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (78894000.0)");
    assert_eq!(evaluate(142009200.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (142009200.0)");
    assert_eq!(evaluate(156210120.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (156210120.0)");
    assert_eq!(evaluate(157788000.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (157788000.0)");
    assert_eq!(evaluate(159365880.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (159365880.0)");
    assert_eq!(evaluate(173566800.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (173566800.0)");
    assert_eq!(evaluate(315576000.0).map(f64::to_bits), Ok(0x4046a8f9096bb98c), "at (315576000.0)");
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
/// Derived from `lead 1 day — 10312 observed pairs` and the declared domain 0 … 50.
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
        if let Err(f) = model::evaluate(Time::new(86400.0 * scale)) {
            refused.push(format!("lead x{scale} -> {f}"));
        }
    }
    assert!(
        refused.is_empty(),
        "sw_horizon_climatology refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 50 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
        if let Ok(v) = model::evaluate(Time::new(86400.0 * scale)) {
            assert!(v.get().is_finite(), "sw_horizon_climatology produced a value that is not a number for D_clim");
            assert!(v.get() >= 0.0 && v.get() <= 50.0, "sw_horizon_climatology answered {} for D_clim, outside its declared domain 0 … 50 — the guard did not stop it", v.get());
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
    let a = model::evaluate(Time::new(86400.0));
    let b = model::evaluate(Time::new(86400.0));
    match (a, b) {
        (Ok(x), Ok(y)) => {
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_horizon_climatology is not deterministic for D_clim: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_horizon_climatology refused on one call and answered on the other"),
    }
}

