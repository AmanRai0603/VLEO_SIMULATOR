// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `P_eol` (Array power, end of life), in `W`.
pub const NODE_ID: &str = "pwr_array_power_eol";
pub const SHEET_HASH: u64 = 0xe22a74ca6e74fb63;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "pwr_array_power_bol",
    "pwr_degradation",
    "pwr_cell_derating",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["pwr_array_power_eol"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Power::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let p: Power = Power::new(inputs[0]);
    let f: Ratio = Ratio::new(inputs[1]);
    let ft: Ratio = Ratio::new(inputs[2]);
    let answer = super::model::evaluate(p, f, ft)?;
    outputs[0] = answer.get();
    Ok(())
}
