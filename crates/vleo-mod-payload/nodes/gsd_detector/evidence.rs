// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `pay_gsd_detector`.
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

/// 250 km, 5.5 um pixel, 2.4 m focal length
///
/// Provenance: `independent-derivation`, source `larson_wertz`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Length::new(250000.0), Length::new(5.5e-6), Length::new(2.4)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.572917);
    assert!(err <= 1e-5, "250 km, 5.5 um pixel, 2.4 m focal length: got {} want 0.572917, relative error {} exceeds the declared tolerance 1e-5. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

