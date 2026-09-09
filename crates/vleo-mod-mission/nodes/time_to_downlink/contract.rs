// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `t_dl` (Mean time to next downlink), in `min`.
pub const NODE_ID: &str = "mis_time_to_downlink";
pub const SHEET_HASH: u64 = 0xdb9bf4704a146539;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "com_passes_per_day",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["mis_time_to_downlink"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Time::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 1 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let n: Ratio = Ratio::new(inputs[0]);
    let answer = super::model::evaluate(n)?;
    outputs[0] = answer.get();
    Ok(())
}
