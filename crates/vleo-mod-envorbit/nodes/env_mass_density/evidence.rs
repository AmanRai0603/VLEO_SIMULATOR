// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `env_mass_density`.
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

/// 250 km, moderate activity
///
/// Provenance: `independent-tool`, source `matlab_legacy`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Length::new(250000.0), Temperature::new(949.6)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 6.631e-11);
    assert!(err <= 0.005, "250 km, moderate activity: got {} want 6.631e-11, relative error {} exceeds the declared tolerance 0.005. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 400 km, moderate activity
///
/// Provenance: `independent-tool`, source `matlab_legacy`.
#[test]
fn fixture_1() {
    let got = model::evaluate(Length::new(400000.0), Temperature::new(949.6)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 2.907e-12);
    assert!(err <= 0.005, "400 km, moderate activity: got {} want 2.907e-12, relative error {} exceeds the declared tolerance 0.005. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

/// 200 km, solar minimum
///
/// Provenance: `independent-tool`, source `matlab_legacy`.
#[test]
fn fixture_2() {
    let got = model::evaluate(Length::new(200000.0), Temperature::new(633.9)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 1.679e-10);
    assert!(err <= 0.005, "200 km, solar minimum: got {} want 1.679e-10, relative error {} exceeds the declared tolerance 0.005. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

