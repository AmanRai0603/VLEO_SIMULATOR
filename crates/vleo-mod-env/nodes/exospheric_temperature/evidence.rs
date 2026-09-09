// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `env_exospheric_temperature`.
//!
//! Every expected value below names a source outside this code. A number
//! produced by the thing being tested proves nothing, so the schema
//! refuses a fixture whose provenance is the implementation.

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

