// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `com_antenna_gain`.
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

/// 0.3 m at 8.2 GHz, 60% efficient
///
/// Provenance: `independent-derivation`, source `larson_wertz`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Length::new(0.3), Frequency::new(8200000000.0), Ratio::new(0.6)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 26.00679804);
    assert!(err <= 1e-8, "0.3 m at 8.2 GHz, 60% efficient: got {} want 26.00679804, relative error {} exceeds the declared tolerance 1e-8. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

