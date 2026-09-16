// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `Ap_short` (Single-day Ap to design to), in `-`.
pub const NODE_ID: &str = "sw_ap_design_short";
pub const SHEET_HASH: u64 = 0xab449467ed97381b;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "sw_ap_design_long",
    "sw_ap_daily_band_spread",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["sw_ap_design_short"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let sustained: Ratio = Ratio::new(inputs[0]);
    let daily: Ratio = Ratio::new(inputs[1]);
    let answer = super::model::evaluate(sustained, daily)?;
    outputs[0] = answer.get();
    Ok(())
}
