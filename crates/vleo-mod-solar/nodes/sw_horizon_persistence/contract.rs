// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `D_pers` (F10.7 error from persistence at a lead), in `-`.
pub const NODE_ID: &str = "sw_horizon_persistence";
pub const SHEET_HASH: u64 = 0x340e2b7af3e268da;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "orbit_mission_duration",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["sw_horizon_persistence"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.is_empty() || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let lead: Time = Time::new(inputs[0]);
    let answer = super::model::evaluate(lead)?;
    outputs[0] = answer.get();
    Ok(())
}
