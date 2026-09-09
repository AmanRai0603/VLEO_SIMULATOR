// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `aero_drag_coefficient`.
//!
//! Every expected value below names a source outside this code. A number
//! produced by the thing being tested proves nothing, so the schema
//! refuses a fixture whose provenance is the implementation.

use super::model;
use vleo_core::units::*;

fn relative_error(got: f64, expected: f64) -> f64 {
    if expected == 0.0 { pmath::abs(got) } else { pmath::abs((got - expected) / expected) }
}

/// 250 km cylinder, moderate activity
///
/// Provenance: `independent-tool`, source `matlab_legacy`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Ratio::new(8.6435), Length::new(2.0), Length::new(0.6), Ratio::new(0.9918), Temperature::new(300.0), Velocity::new(7754.6), MolarMass::new(0.01872), Ratio::new(10000.0)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 2.85807);
    assert!(err <= 0.001, "250 km cylinder, moderate activity: got {} want 2.85807, relative error {} exceeds the declared tolerance 0.001. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

