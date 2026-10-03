// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `sw_horizon_persistence`.
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
    let err = relative_error(got.get(), 7.0784);
    assert!(err <= 1e-12, "lead 1 day — 10312 observed pairs: got {} want 7.0784, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 2 days — 10309 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Time::new(172800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 10.3796);
    assert!(err <= 1e-12, "lead 2 days — 10309 observed pairs: got {} want 10.3796, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 3 days — 10307 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Time::new(259200.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 13.6671);
    assert!(err <= 1e-12, "lead 3 days — 10307 observed pairs: got {} want 13.6671, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 5 days — 10303 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_3() {
    let got = model::evaluate(Time::new(432000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 19.3021);
    assert!(err <= 1e-12, "lead 5 days — 10303 observed pairs: got {} want 19.3021, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 7 days — 10299 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_4() {
    let got = model::evaluate(Time::new(604800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 23.6294);
    assert!(err <= 1e-12, "lead 7 days — 10299 observed pairs: got {} want 23.6294, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 10 days — 10293 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_5() {
    let got = model::evaluate(Time::new(864000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 27.6511);
    assert!(err <= 1e-12, "lead 10 days — 10293 observed pairs: got {} want 27.6511, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 14 days — 10285 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_6() {
    let got = model::evaluate(Time::new(1209600.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 29.1328);
    assert!(err <= 1e-12, "lead 14 days — 10285 observed pairs: got {} want 29.1328, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 20 days — 10273 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_7() {
    let got = model::evaluate(Time::new(1728000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 25.4695);
    assert!(err <= 1e-12, "lead 20 days — 10273 observed pairs: got {} want 25.4695, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 27 days — 10259 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_8() {
    let got = model::evaluate(Time::new(2332800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 22.4321);
    assert!(err <= 1e-12, "lead 27 days — 10259 observed pairs: got {} want 22.4321, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 40 days — 10233 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_9() {
    let got = model::evaluate(Time::new(3456000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 29.602);
    assert!(err <= 1e-12, "lead 40 days — 10233 observed pairs: got {} want 29.602, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 60 days — 10193 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_10() {
    let got = model::evaluate(Time::new(5184000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 27.9575);
    assert!(err <= 1e-12, "lead 60 days — 10193 observed pairs: got {} want 27.9575, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 90 days — 10133 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_11() {
    let got = model::evaluate(Time::new(7776000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 30.1549);
    assert!(err <= 1e-12, "lead 90 days — 10133 observed pairs: got {} want 30.1549, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 135 days — 10043 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_12() {
    let got = model::evaluate(Time::new(11664000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 31.0394);
    assert!(err <= 1e-12, "lead 135 days — 10043 observed pairs: got {} want 31.0394, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 180 days — 9953 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_13() {
    let got = model::evaluate(Time::new(15552000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 32.9634);
    assert!(err <= 1e-12, "lead 180 days — 9953 observed pairs: got {} want 32.9634, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 270 days — 9773 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_14() {
    let got = model::evaluate(Time::new(23328000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 35.7538);
    assert!(err <= 1e-12, "lead 270 days — 9773 observed pairs: got {} want 35.7538, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 365 days — 9675 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_15() {
    let got = model::evaluate(Time::new(31536000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 38.8277);
    assert!(err <= 1e-12, "lead 365 days — 9675 observed pairs: got {} want 38.8277, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 547 days — 9493 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_16() {
    let got = model::evaluate(Time::new(47260800.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 43.7693);
    assert!(err <= 1e-12, "lead 547 days — 9493 observed pairs: got {} want 43.7693, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// lead 730 days — 9310 observed pairs
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_17() {
    let got = model::evaluate(Time::new(63072000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 49.6285);
    assert!(err <= 1e-12, "lead 730 days — 9310 observed pairs: got {} want 49.6285, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// beyond the last measured lead — five years holds the two-year value rather than extrapolating
///
/// Provenance: `independent-derivation`, source `noaa_swpc`.
#[test]
fn fixture_18() {
    let got = model::evaluate(Time::new(157788000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 49.6285);
    assert!(err <= 1e-12, "beyond the last measured lead — five years holds the two-year value rather than extrapolating: got {} want 49.6285, relative error {} exceeds the declared tolerance 1e-12. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 1 day — 10312 observed pairs.», from their own  code.
#[test]
fn case_1() {
    let got = model::evaluate(Time::new(86400.0)).expect("lead 1 day — 10312 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 7.0784);
    assert!(err <= 1e-12, "lead 1 day — 10312 observed pairs.: got {} and the author's code gave 7.0784; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 2 days — 10309 observed pairs.», from their own  code.
#[test]
fn case_2() {
    let got = model::evaluate(Time::new(172800.0)).expect("lead 2 days — 10309 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 10.3796);
    assert!(err <= 1e-12, "lead 2 days — 10309 observed pairs.: got {} and the author's code gave 10.3796; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 3 days — 10307 observed pairs.», from their own  code.
#[test]
fn case_3() {
    let got = model::evaluate(Time::new(259200.0)).expect("lead 3 days — 10307 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 13.6671);
    assert!(err <= 1e-12, "lead 3 days — 10307 observed pairs.: got {} and the author's code gave 13.6671; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 5 days — 10303 observed pairs.», from their own  code.
#[test]
fn case_4() {
    let got = model::evaluate(Time::new(432000.0)).expect("lead 5 days — 10303 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 19.3021);
    assert!(err <= 1e-12, "lead 5 days — 10303 observed pairs.: got {} and the author's code gave 19.3021; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 7 days — 10299 observed pairs.», from their own  code.
#[test]
fn case_5() {
    let got = model::evaluate(Time::new(604800.0)).expect("lead 7 days — 10299 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 23.6294);
    assert!(err <= 1e-12, "lead 7 days — 10299 observed pairs.: got {} and the author's code gave 23.6294; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 10 days — 10293 observed pairs.», from their own  code.
#[test]
fn case_6() {
    let got = model::evaluate(Time::new(864000.0)).expect("lead 10 days — 10293 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 27.6511);
    assert!(err <= 1e-12, "lead 10 days — 10293 observed pairs.: got {} and the author's code gave 27.6511; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 14 days — 10285 observed pairs.», from their own  code.
#[test]
fn case_7() {
    let got = model::evaluate(Time::new(1209600.0)).expect("lead 14 days — 10285 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 29.1328);
    assert!(err <= 1e-12, "lead 14 days — 10285 observed pairs.: got {} and the author's code gave 29.1328; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 20 days — 10273 observed pairs.», from their own  code.
#[test]
fn case_8() {
    let got = model::evaluate(Time::new(1728000.0)).expect("lead 20 days — 10273 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 25.4695);
    assert!(err <= 1e-12, "lead 20 days — 10273 observed pairs.: got {} and the author's code gave 25.4695; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 27 days — 10259 observed pairs.», from their own  code.
#[test]
fn case_9() {
    let got = model::evaluate(Time::new(2332800.0)).expect("lead 27 days — 10259 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 22.4321);
    assert!(err <= 1e-12, "lead 27 days — 10259 observed pairs.: got {} and the author's code gave 22.4321; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 40 days — 10233 observed pairs.», from their own  code.
#[test]
fn case_10() {
    let got = model::evaluate(Time::new(3456000.0)).expect("lead 40 days — 10233 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 29.602);
    assert!(err <= 1e-12, "lead 40 days — 10233 observed pairs.: got {} and the author's code gave 29.602; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 60 days — 10193 observed pairs.», from their own  code.
#[test]
fn case_11() {
    let got = model::evaluate(Time::new(5184000.0)).expect("lead 60 days — 10193 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 27.9575);
    assert!(err <= 1e-12, "lead 60 days — 10193 observed pairs.: got {} and the author's code gave 27.9575; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 90 days — 10133 observed pairs.», from their own  code.
#[test]
fn case_12() {
    let got = model::evaluate(Time::new(7776000.0)).expect("lead 90 days — 10133 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 30.1549);
    assert!(err <= 1e-12, "lead 90 days — 10133 observed pairs.: got {} and the author's code gave 30.1549; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 135 days — 10043 observed pairs.», from their own  code.
#[test]
fn case_13() {
    let got = model::evaluate(Time::new(11664000.0)).expect("lead 135 days — 10043 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 31.0394);
    assert!(err <= 1e-12, "lead 135 days — 10043 observed pairs.: got {} and the author's code gave 31.0394; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 180 days — 9953 observed pairs.», from their own  code.
#[test]
fn case_14() {
    let got = model::evaluate(Time::new(15552000.0)).expect("lead 180 days — 9953 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 32.9634);
    assert!(err <= 1e-12, "lead 180 days — 9953 observed pairs.: got {} and the author's code gave 32.9634; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 270 days — 9773 observed pairs.», from their own  code.
#[test]
fn case_15() {
    let got = model::evaluate(Time::new(23328000.0)).expect("lead 270 days — 9773 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 35.7538);
    assert!(err <= 1e-12, "lead 270 days — 9773 observed pairs.: got {} and the author's code gave 35.7538; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 365 days — 9675 observed pairs.», from their own  code.
#[test]
fn case_16() {
    let got = model::evaluate(Time::new(31536000.0)).expect("lead 365 days — 9675 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 38.8277);
    assert!(err <= 1e-12, "lead 365 days — 9675 observed pairs.: got {} and the author's code gave 38.8277; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 547 days — 9493 observed pairs.», from their own  code.
#[test]
fn case_17() {
    let got = model::evaluate(Time::new(47260800.0)).expect("lead 547 days — 9493 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 43.7693);
    assert!(err <= 1e-12, "lead 547 days — 9493 observed pairs.: got {} and the author's code gave 43.7693; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «lead 730 days — 9310 observed pairs.», from their own  code.
#[test]
fn case_18() {
    let got = model::evaluate(Time::new(63072000.0)).expect("lead 730 days — 9310 observed pairs.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 49.6285);
    assert!(err <= 1e-12, "lead 730 days — 9310 observed pairs.: got {} and the author's code gave 49.6285; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «beyond the last measured lead — five years holds the two-year value rather than extrapolating.», from their own  code.
#[test]
fn case_19() {
    let got = model::evaluate(Time::new(157788000.0)).expect("beyond the last measured lead — five years holds the two-year value rather than extrapolating.: the author's code answers this case and the node refused it");
    let err = relative_error(got.get(), 49.6285);
    assert!(err <= 1e-12, "beyond the last measured lead — five years holds the two-year value rather than extrapolating.: got {} and the author's code gave 49.6285; relative error {} is more than their tolerance 1e-12. Take it to the author; do not widen the tolerance.", got.get(), err);
}

/// the author's case «a value that is not a number: the node must refuse it, as its code does», from their own  code.
#[test]
fn case_20() {
    let got = model::evaluate(Time::new(f64::NAN));
    assert!(matches!(got, Err(vleo_core::fault::Fault::Refused { .. })), "a value that is not a number: the node must refuse it, as its code does: the author's code refuses this case and the node gave {:?}. Take it to the author.", got);
}

/// The kernel translation of this node's method gives the method's own
/// answer, to the bit, at 134 points around the author's cases. A translator
/// check, not evidence: the numbers are the method's, run by the interpreter
/// when this file was generated.
#[test]
fn the_translation_gives_the_methods_answers() {
    use vleo_core::physics::method::MethodError;
    use vleo_core::physics::methods::sw_horizon_persistence::evaluate;
    assert_eq!(evaluate(43200.0).map(f64::to_bits), Ok(0x401c504816f0068e), "at (43200.0)");
    assert_eq!(evaluate(77760.0).map(f64::to_bits), Ok(0x401c504816f0068e), "at (77760.0)");
    assert_eq!(evaluate(85536.0).map(f64::to_bits), Ok(0x401c504816f0068e), "at (85536.0)");
    assert_eq!(evaluate(86400.0).map(f64::to_bits), Ok(0x401c504816f0068e), "at (86400.0)");
    assert_eq!(evaluate(87264.0).map(f64::to_bits), Ok(0x401c7215fcc1871f), "at (87264.0)");
    assert_eq!(evaluate(95040.00000000001).map(f64::to_bits), Ok(0x401da253111f0c36), "at (95040.00000000001)");
    assert_eq!(evaluate(172800.0).map(f64::to_bits), Ok(0x4024c25aee631f8a), "at (172800.0)");
    assert_eq!(evaluate(86400.0).map(f64::to_bits), Ok(0x401c504816f0068e), "at (86400.0)");
    assert_eq!(evaluate(155520.0).map(f64::to_bits), Ok(0x4023704ff43419e3), "at (155520.0)");
    assert_eq!(evaluate(171072.0).map(f64::to_bits), Ok(0x4024a08d08919ef9), "at (171072.0)");
    assert_eq!(evaluate(172800.0).map(f64::to_bits), Ok(0x4024c25aee631f8a), "at (172800.0)");
    assert_eq!(evaluate(174528.0).map(f64::to_bits), Ok(0x4024e404ea4a8c15), "at (174528.0)");
    assert_eq!(evaluate(190080.00000000003).map(f64::to_bits), Ok(0x402612fec56d5cfb), "at (190080.00000000003)");
    assert_eq!(evaluate(345600.0).map(f64::to_bits), Ok(0x40307c0ebedfa440), "at (345600.0)");
    assert_eq!(evaluate(129600.0).map(f64::to_bits), Ok(0x4021753f7ced9168), "at (129600.0)");
    assert_eq!(evaluate(233280.0).map(f64::to_bits), Ok(0x40295c985f06f694), "at (233280.0)");
    assert_eq!(evaluate(256608.0).map(f64::to_bits), Ok(0x402b230f27bb2fec), "at (256608.0)");
    assert_eq!(evaluate(259200.0).map(f64::to_bits), Ok(0x402b558e219652bd), "at (259200.0)");
    assert_eq!(evaluate(261792.0).map(f64::to_bits), Ok(0x402b80d4fdf3b645), "at (261792.0)");
    assert_eq!(evaluate(285120.0).map(f64::to_bits), Ok(0x402d0652bd3c3611), "at (285120.0)");
    assert_eq!(evaluate(518400.0).map(f64::to_bits), Ok(0x4035773b645a1cac), "at (518400.0)");
    assert_eq!(evaluate(216000.0).map(f64::to_bits), Ok(0x40280bf487fcb924), "at (216000.0)");
    assert_eq!(evaluate(388800.0).map(f64::to_bits), Ok(0x4031e4b295e9e1b0), "at (388800.0)");
    assert_eq!(evaluate(427680.0).map(f64::to_bits), Ok(0x403329460aa64c2f), "at (427680.0)");
    assert_eq!(evaluate(432000.0).map(f64::to_bits), Ok(0x40334d566cf41f21), "at (432000.0)");
    assert_eq!(evaluate(436320.0).map(f64::to_bits), Ok(0x40336908461f9f02), "at (436320.0)");
    assert_eq!(evaluate(475200.00000000006).map(f64::to_bits), Ok(0x40346248e8a71de7), "at (475200.00000000006)");
    assert_eq!(evaluate(864000.0).map(f64::to_bits), Ok(0x403ba6ae7d566cf4), "at (864000.0)");
    assert_eq!(evaluate(302400.0).map(f64::to_bits), Ok(0x402e26d5cfaacd9e), "at (302400.0)");
    assert_eq!(evaluate(544320.0).map(f64::to_bits), Ok(0x40361d667b5f1bef), "at (544320.0)");
    assert_eq!(evaluate(598752.0).map(f64::to_bits), Ok(0x40377a5a921ccd96), "at (598752.0)");
    assert_eq!(evaluate(604800.0).map(f64::to_bits), Ok(0x4037a1205bc01a37), "at (604800.0)");
    assert_eq!(evaluate(610848.0).map(f64::to_bits), Ok(0x4037b9263c1b80d7), "at (610848.0)");
    assert_eq!(evaluate(665280.0).map(f64::to_bits), Ok(0x4038915b1f521c74), "at (665280.0)");
    assert_eq!(evaluate(1209600.0).map(f64::to_bits), Ok(0x403d21ff2e48e8a7), "at (1209600.0)");
    assert_eq!(evaluate(432000.0).map(f64::to_bits), Ok(0x40334d566cf41f21), "at (432000.0)");
    assert_eq!(evaluate(777600.0).map(f64::to_bits), Ok(0x403a4f7f1ccefc0a), "at (777600.0)");
    assert_eq!(evaluate(855360.0).map(f64::to_bits), Ok(0x403b845cf3af4810), "at (855360.0)");
    assert_eq!(evaluate(864000.0).map(f64::to_bits), Ok(0x403ba6ae7d566cf4), "at (864000.0)");
    assert_eq!(evaluate(872640.0).map(f64::to_bits), Ok(0x403bb02a1b5c7cd8), "at (872640.0)");
    assert_eq!(evaluate(950400.0000000001).map(f64::to_bits), Ok(0x403c0582a9930be1), "at (950400.0000000001)");
    assert_eq!(evaluate(1728000.0).map(f64::to_bits), Ok(0x4039783126e978d5), "at (1728000.0)");
    assert_eq!(evaluate(604800.0).map(f64::to_bits), Ok(0x4037a1205bc01a37), "at (604800.0)");
    assert_eq!(evaluate(1088640.0).map(f64::to_bits), Ok(0x403c9d3c89f40a28), "at (1088640.0)");
    assert_eq!(evaluate(1197504.0).map(f64::to_bits), Ok(0x403d14b884406c01), "at (1197504.0)");
    assert_eq!(evaluate(1209600.0).map(f64::to_bits), Ok(0x403d21ff2e48e8a7), "at (1209600.0)");
    assert_eq!(evaluate(1221696.0).map(f64::to_bits), Ok(0x403d0c1d5c31593e), "at (1221696.0)");
    assert_eq!(evaluate(1330560.0).map(f64::to_bits), Ok(0x403c472cf95d4e90), "at (1330560.0)");
    assert_eq!(evaluate(2419200.0).map(f64::to_bits), Ok(0x4036fbcf39e12f7b), "at (2419200.0)");
    assert_eq!(evaluate(864000.0).map(f64::to_bits), Ok(0x403ba6ae7d566cf4), "at (864000.0)");
    assert_eq!(evaluate(1555200.0).map(f64::to_bits), Ok(0x403ab0cb295e9e1b), "at (1555200.0)");
    assert_eq!(evaluate(1710720.0).map(f64::to_bits), Ok(0x40399773c0c1fc8f), "at (1710720.0)");
    assert_eq!(evaluate(1728000.0).map(f64::to_bits), Ok(0x4039783126e978d5), "at (1728000.0)");
    assert_eq!(evaluate(1745280.0).map(f64::to_bits), Ok(0x403961f9c02c30a3), "at (1745280.0)");
    assert_eq!(evaluate(1900800.0000000002).map(f64::to_bits), Ok(0x40389a072384a6e2), "at (1900800.0000000002)");
    assert_eq!(evaluate(3456000.0).map(f64::to_bits), Ok(0x403d9a1cac083127), "at (3456000.0)");
    assert_eq!(evaluate(1166400.0).map(f64::to_bits), Ok(0x403cf295182a9931), "at (1166400.0)");
    assert_eq!(evaluate(2099520.0).map(f64::to_bits), Ok(0x40379a8a0603e8a4), "at (2099520.0)");
    assert_eq!(evaluate(2309472.0).map(f64::to_bits), Ok(0x40368c9c18ee8845), "at (2309472.0)");
    assert_eq!(evaluate(2332800.0).map(f64::to_bits), Ok(0x40366e9e1b089a02), "at (2332800.0)");
    assert_eq!(evaluate(2356128.0).map(f64::to_bits), Ok(0x403694bd49c31449), "at (2356128.0)");
    assert_eq!(evaluate(2566080.0).map(f64::to_bits), Ok(0x4037ebd5ee5160c9), "at (2566080.0)");
    assert_eq!(evaluate(4665600.0).map(f64::to_bits), Ok(0x403c736ae7d566cf), "at (4665600.0)");
    assert_eq!(evaluate(1728000.0).map(f64::to_bits), Ok(0x4039783126e978d5), "at (1728000.0)");
    assert_eq!(evaluate(3110400.0).map(f64::to_bits), Ok(0x403b655830a5db43), "at (3110400.0)");
    assert_eq!(evaluate(3421440.0).map(f64::to_bits), Ok(0x403d61a29fb18ef6), "at (3421440.0)");
    assert_eq!(evaluate(3456000.0).map(f64::to_bits), Ok(0x403d9a1cac083127), "at (3456000.0)");
    assert_eq!(evaluate(3490560.0).map(f64::to_bits), Ok(0x403d91b13165d39a), "at (3490560.0)");
    assert_eq!(evaluate(3801600.0000000005).map(f64::to_bits), Ok(0x403d45e9e1b089a0), "at (3801600.0000000005)");
    assert_eq!(evaluate(6912000.0).map(f64::to_bits), Ok(0x403d6c249747682d), "at (6912000.0)");
    assert_eq!(evaluate(2592000.0).map(f64::to_bits), Ok(0x4038163177925a6d), "at (2592000.0)");
    assert_eq!(evaluate(4665600.0).map(f64::to_bits), Ok(0x403c736ae7d566cf), "at (4665600.0)");
    assert_eq!(evaluate(5132160.0).map(f64::to_bits), Ok(0x403c01bff04577d9), "at (5132160.0)");
    assert_eq!(evaluate(5184000.0).map(f64::to_bits), Ok(0x403bf51eb851eb85), "at (5184000.0)");
    assert_eq!(evaluate(5235840.0).map(f64::to_bits), Ok(0x403c005ee568a505), "at (5235840.0)");
    assert_eq!(evaluate(5702400.0).map(f64::to_bits), Ok(0x403c65a07b352a84), "at (5702400.0)");
    assert_eq!(evaluate(10368000.0).map(f64::to_bits), Ok(0x403ebe9bebcb066b), "at (10368000.0)");
    assert_eq!(evaluate(3888000.0).map(f64::to_bits), Ok(0x403d30dd2f1a9fbe), "at (3888000.0)");
    assert_eq!(evaluate(6998400.0).map(f64::to_bits), Ok(0x403d7ee4e26d4802), "at (6998400.0)");
    assert_eq!(evaluate(7698240.0).map(f64::to_bits), Ok(0x403e16c743201041), "at (7698240.0)");
    assert_eq!(evaluate(7776000.0).map(f64::to_bits), Ok(0x403e27a786c22681), "at (7776000.0)");
    assert_eq!(evaluate(7853760.0).map(f64::to_bits), Ok(0x403e2c2edbb59ddc), "at (7853760.0)");
    assert_eq!(evaluate(8553600.0).map(f64::to_bits), Ok(0x403e54f0d844d014), "at (8553600.0)");
    assert_eq!(evaluate(15552000.0).map(f64::to_bits), Ok(0x40407b50b0f27bb3), "at (15552000.0)");
    assert_eq!(evaluate(5832000.0).map(f64::to_bits), Ok(0x403c81c0ebedfa44), "at (5832000.0)");
    assert_eq!(evaluate(10497600.0).map(f64::to_bits), Ok(0x403ec628240b7804), "at (10497600.0)");
    assert_eq!(evaluate(11547360.0).map(f64::to_bits), Ok(0x403f034b1ee24357), "at (11547360.0)");
    assert_eq!(evaluate(11664000.0).map(f64::to_bits), Ok(0x403f0a161e4f7660), "at (11664000.0)");
    assert_eq!(evaluate(11780640.0).map(f64::to_bits), Ok(0x403f18dcdb37c99b), "at (11780640.0)");
    assert_eq!(evaluate(12830400.000000002).map(f64::to_bits), Ok(0x403f9dd97f62b6af), "at (12830400.000000002)");
    assert_eq!(evaluate(23328000.0).map(f64::to_bits), Ok(0x4041e07c84b5dcc6), "at (23328000.0)");
    assert_eq!(evaluate(7776000.0).map(f64::to_bits), Ok(0x403e27a786c22681), "at (7776000.0)");
    assert_eq!(evaluate(13996800.0).map(f64::to_bits), Ok(0x404018ce703afb7f), "at (13996800.0)");
    assert_eq!(evaluate(15396480.0).map(f64::to_bits), Ok(0x40407176ddaceee1), "at (15396480.0)");
    assert_eq!(evaluate(15552000.0).map(f64::to_bits), Ok(0x40407b50b0f27bb3), "at (15552000.0)");
    assert_eq!(evaluate(15707520.0).map(f64::to_bits), Ok(0x404082756861e929), "at (15707520.0)");
    assert_eq!(evaluate(17107200.0).map(f64::to_bits), Ok(0x4040c2bfdb4cc250), "at (17107200.0)");
    assert_eq!(evaluate(31104000.0).map(f64::to_bits), Ok(0x4043553cba84d6fa), "at (31104000.0)");
    assert_eq!(evaluate(11664000.0).map(f64::to_bits), Ok(0x403f0a161e4f7660), "at (11664000.0)");
    assert_eq!(evaluate(20995200.0).map(f64::to_bits), Ok(0x40417555c52e72da), "at (20995200.0)");
    assert_eq!(evaluate(23094720.0).map(f64::to_bits), Ok(0x4041d5c5718eb895), "at (23094720.0)");
    assert_eq!(evaluate(23328000.0).map(f64::to_bits), Ok(0x4041e07c84b5dcc6), "at (23328000.0)");
    assert_eq!(evaluate(23561280.0).map(f64::to_bits), Ok(0x4041ebab3ea5081f), "at (23561280.0)");
    assert_eq!(evaluate(25660800.000000004).map(f64::to_bits), Ok(0x4042504fc80d8e3c), "at (25660800.000000004)");
    assert_eq!(evaluate(46656000.0).map(f64::to_bits), Ok(0x4045ca247c647250), "at (46656000.0)");
    assert_eq!(evaluate(15768000.0).map(f64::to_bits), Ok(0x4040853c934621f4), "at (15768000.0)");
    assert_eq!(evaluate(28382400.0).map(f64::to_bits), Ok(0x4042d2c64149329b), "at (28382400.0)");
    assert_eq!(evaluate(31220640.0).map(f64::to_bits), Ok(0x40435ad4177c6ca6), "at (31220640.0)");
    assert_eq!(evaluate(31536000.0).map(f64::to_bits), Ok(0x404369f212d77319), "at (31536000.0)");
    assert_eq!(evaluate(31851360.0).map(f64::to_bits), Ok(0x404376a17f648ea8), "at (31851360.0)");
    assert_eq!(evaluate(34689600.0).map(f64::to_bits), Ok(0x4043e8cc505a86af), "at (34689600.0)");
    assert_eq!(evaluate(63072000.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (63072000.0)");
    assert_eq!(evaluate(23630400.0).map(f64::to_bits), Ok(0x4041eefb75bc7d42), "at (23630400.0)");
    assert_eq!(evaluate(42534720.0).map(f64::to_bits), Ok(0x4045245d8c17d58f), "at (42534720.0)");
    assert_eq!(evaluate(46788192.0).map(f64::to_bits), Ok(0x4045cf75bc215964), "at (46788192.0)");
    assert_eq!(evaluate(47260800.0).map(f64::to_bits), Ok(0x4045e2786c22680a), "at (47260800.0)");
    assert_eq!(evaluate(47733408.0).map(f64::to_bits), Ok(0x4045f8e344738ccd), "at (47733408.0)");
    assert_eq!(evaluate(51986880.00000001).map(f64::to_bits), Ok(0x4046c2a4df4dd7a5), "at (51986880.00000001)");
    assert_eq!(evaluate(94521600.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (94521600.0)");
    assert_eq!(evaluate(31536000.0).map(f64::to_bits), Ok(0x404369f212d77319), "at (31536000.0)");
    assert_eq!(evaluate(56764800.0).map(f64::to_bits), Ok(0x4047a546cfc24bc0), "at (56764800.0)");
    assert_eq!(evaluate(62441280.0).map(f64::to_bits), Ok(0x4048b287e67db886), "at (62441280.0)");
    assert_eq!(evaluate(63072000.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (63072000.0)");
    assert_eq!(evaluate(63702720.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (63702720.0)");
    assert_eq!(evaluate(69379200.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (69379200.0)");
    assert_eq!(evaluate(126144000.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (126144000.0)");
    assert_eq!(evaluate(78894000.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (78894000.0)");
    assert_eq!(evaluate(142009200.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (142009200.0)");
    assert_eq!(evaluate(156210120.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (156210120.0)");
    assert_eq!(evaluate(157788000.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (157788000.0)");
    assert_eq!(evaluate(159365880.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (159365880.0)");
    assert_eq!(evaluate(173566800.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (173566800.0)");
    assert_eq!(evaluate(315576000.0).map(f64::to_bits), Ok(0x4048d072b020c49c), "at (315576000.0)");
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
/// Derived from `lead 1 day — 10312 observed pairs` and the declared domain 0 … 65.
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
        "sw_horizon_persistence refuses near its own known-good point: {:?}. Either the relation is wrong in shape, or the declared domain 0 … 65 is narrower than the physics. Both are sheet questions for the node owner, not tolerances to widen.",
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
            assert!(v.get().is_finite(), "sw_horizon_persistence produced a value that is not a number for D_pers");
            assert!(v.get() >= 0.0 && v.get() <= 65.0, "sw_horizon_persistence answered {} for D_pers, outside its declared domain 0 … 65 — the guard did not stop it", v.get());
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
            assert!(x.get().to_bits() == y.get().to_bits(), "sw_horizon_persistence is not deterministic for D_pers: {} then {}", x.get(), y.get());
        }
        (Err(_), Err(_)) => {}
        _ => panic!("sw_horizon_persistence refused on one call and answered on the other"),
    }
}

