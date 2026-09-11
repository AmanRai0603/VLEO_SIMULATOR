// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `aero_speed_ratio`.
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
/// Provenance: `independent-derivation`, source `sentman1961`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Velocity::new(7754.6), Temperature::new(905.8), MolarMass::new(0.01872)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 8.6435);
    assert!(err <= 0.005, "250 km, moderate activity: got {} want 8.6435, relative error {} exceeds the declared tolerance 0.005. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

