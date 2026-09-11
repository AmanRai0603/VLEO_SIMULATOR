// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `EbN0` (Achieved energy per bit over noise density), in `dB`.
pub const NODE_ID: &str = "com_ebn0";
pub const SHEET_HASH: u64 = 0x02f5e9962027c2e5;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "com_cn0",
    "com_data_rate",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["com_ebn0"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let c: Ratio = Ratio::new(inputs[0]);
    let r: DataRate = DataRate::new(inputs[1]);
    let answer = super::model::evaluate(c, r)?;
    outputs[0] = answer.get();
    Ok(())
}
