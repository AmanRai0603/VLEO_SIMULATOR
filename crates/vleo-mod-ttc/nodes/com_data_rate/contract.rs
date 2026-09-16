// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `R_b` (Achievable downlink data rate), in `Mbit/s`.
pub const NODE_ID: &str = "com_data_rate";
pub const SHEET_HASH: u64 = 0x7cd4c4bc025cbc11;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "com_cn0",
    "com_required_ebn0",
    "com_implementation_loss",
    "com_link_margin_required",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["com_data_rate"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = DataRate::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[DataRate::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 4 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let c: Ratio = Ratio::new(inputs[0]);
    let er: Ratio = Ratio::new(inputs[1]);
    let li: Ratio = Ratio::new(inputs[2]);
    let m: Ratio = Ratio::new(inputs[3]);
    let answer = super::model::evaluate(c, er, li, m)?;
    outputs[0] = answer.get();
    Ok(())
}
