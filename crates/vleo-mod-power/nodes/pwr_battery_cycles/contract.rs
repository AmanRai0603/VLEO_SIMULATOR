// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `N_cyc` (Battery charge cycles), in `-`.
pub const NODE_ID: &str = "pwr_battery_cycles";
pub const SHEET_HASH: u64 = 0x65d0729cb5ce91c2;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "orbit_mission_duration",
    "orbit_period",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["pwr_battery_cycles"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let tm: Time = Time::new(inputs[0]);
    let to: Time = Time::new(inputs[1]);
    let answer = super::model::evaluate(tm, to)?;
    outputs[0] = answer.get();
    Ok(())
}
