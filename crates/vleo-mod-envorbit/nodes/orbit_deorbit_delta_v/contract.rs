// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `dv_dis` (Disposal delta-v), in `m/s`.
pub const NODE_ID: &str = "orbit_deorbit_delta_v";
pub const SHEET_HASH: u64 = 0x8ef8576fb8b9f5a1;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "orbit_altitude",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["orbit_deorbit_delta_v"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Velocity::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Velocity::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.is_empty() || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let h: Length = Length::new(inputs[0]);
    let answer = super::model::evaluate(h)?;
    outputs[0] = answer.get();
    Ok(())
}
