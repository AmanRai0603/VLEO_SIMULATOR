// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `CN0` (Carrier to noise density ratio), in `dB`.
pub const NODE_ID: &str = "com_cn0";
pub const SHEET_HASH: u64 = 0x542e132e5634ca84;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "com_eirp",
    "com_path_loss",
    "com_atmospheric_loss",
    "com_g_over_t",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["com_cn0"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 4 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let e: Ratio = Ratio::new(inputs[0]);
    let lf: Ratio = Ratio::new(inputs[1]);
    let la: Ratio = Ratio::new(inputs[2]);
    let gt: Ratio = Ratio::new(inputs[3]);
    let answer = super::model::evaluate(e, lf, la, gt)?;
    outputs[0] = answer.get();
    Ok(())
}
