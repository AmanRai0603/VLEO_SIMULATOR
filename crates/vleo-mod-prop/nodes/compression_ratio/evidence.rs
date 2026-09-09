// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `prop_compression_ratio`.
//!
//! Every expected value below names a source outside this code. A number
//! produced by the thing being tested proves nothing, so the schema
//! refuses a fixture whose provenance is the implementation.

use super::model;
use vleo_core::units::*;

fn relative_error(got: f64, expected: f64) -> f64 {
    if expected == 0.0 { pmath::abs(got) } else { pmath::abs((got - expected) / expected) }
}

/// IRS-class baseline intake
///
/// Provenance: `independent-derivation`, source `romano2021`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Area::new(0.2), Area::new(0.01), Ratio::new(0.9), Ratio::new(0.06), Temperature::new(600.0), MolarMass::new(0.01872), NumberDensity::new(2133000000000000.0), Velocity::new(7754.6)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 308.09);
    assert!(err <= 0.001, "IRS-class baseline intake: got {} want 308.09, relative error {} exceeds the declared tolerance 0.001. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

