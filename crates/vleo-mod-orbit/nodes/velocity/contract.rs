// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `V` (Circular orbital speed), in `m/s`.
pub const NODE_ID: &str = "orbit_velocity";
pub const SHEET_HASH: u64 = 0xf3181e3719f1fb4a;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "orbit_radius",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["orbit_velocity"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Velocity::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 1 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let r: Length = Length::new(inputs[0]);
    let answer = super::model::evaluate(r)?;
    outputs[0] = answer.get();
    Ok(())
}
