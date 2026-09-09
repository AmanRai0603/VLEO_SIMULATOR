// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `A_acc` (Instantaneous access area), in `m^2`.
pub const NODE_ID: &str = "mis_access_area";
pub const SHEET_HASH: u64 = 0xb3349c53a804b26c;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "orbit_earth_central_angle",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["mis_access_area"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Area::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.is_empty() || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let lam: Angle = Angle::new(inputs[0]);
    let answer = super::model::evaluate(lam)?;
    outputs[0] = answer.get();
    Ok(())
}
