// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `F107_cyc` (Mean-cycle F10.7 at a phase), in `-`.
pub const NODE_ID: &str = "sw_mean_cycle_level";
pub const SHEET_HASH: u64 = 0x9c8ab20dbc4cf7b2;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "sw_cycle_phase",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["sw_mean_cycle_level"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.is_empty() || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let phase: Ratio = Ratio::new(inputs[0]);
    let answer = super::model::evaluate(phase)?;
    outputs[0] = answer.get();
    Ok(())
}
