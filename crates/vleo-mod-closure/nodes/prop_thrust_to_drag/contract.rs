// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `T_D` (Thrust to drag ratio), in `-`.
pub const NODE_ID: &str = "prop_thrust_to_drag";
pub const SHEET_HASH: u64 = 0xb0b65a7d9b47f9b4;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "prop_delivered_thrust",
    "aero_drag_force",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["prop_thrust_to_drag"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Ratio::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let t: Force = Force::new(inputs[0]);
    let d: Force = Force::new(inputs[1]);
    let answer = super::model::evaluate(t, d)?;
    outputs[0] = answer.get();
    Ok(())
}
