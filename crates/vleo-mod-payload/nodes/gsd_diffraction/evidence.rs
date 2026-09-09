// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! Evidence for `pay_gsd_diffraction`.
//!
//! Every expected value below names a source outside this code. A number
//! produced by the thing being tested proves nothing, so the schema
//! refuses a fixture whose provenance is the implementation.

use super::model;
use vleo_core::units::*;

fn relative_error(got: f64, expected: f64) -> f64 {
    if expected == 0.0 { pmath::abs(got) } else { pmath::abs((got - expected) / expected) }
}

/// 250 km, 550 nm, 0.3 m aperture
///
/// Provenance: `independent-derivation`, source `larson_wertz`.
#[test]
fn fixture_0() {
    let got = model::evaluate(Length::new(250000.0), Length::new(5.5e-7), Length::new(0.3)).expect("the fixture case must not be refused");
    let err = relative_error(got.get(), 0.55917);
    assert!(err <= 0.0001, "250 km, 550 nm, 0.3 m aperture: got {} want 0.55917, relative error {} exceeds the declared tolerance 0.0001. This is a physics disagreement, not a build failure — take it to the node owner. Do not widen the tolerance.", got.get(), err);
}

