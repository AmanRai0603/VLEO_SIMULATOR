// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `M_f107_long` (Closure — F10.7, sustained), in `-`.
pub const NODE_ID: &str = "l3_solar_ach_01";
pub const SHEET_HASH: u64 = 0x906ef3769bef9709;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "sw_f107_design_long",
    "l3_solar_req_01",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["l3_solar_ach_01"];
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
    let ach: Ratio = Ratio::new(inputs[0]);
    let req: Ratio = Ratio::new(inputs[1]);
    let answer = super::model::evaluate(ach, req)?;
    outputs[0] = answer.get();
    Ok(())
}
