// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `lambda` (Mean free path), in `m`.
pub const NODE_ID: &str = "env_mean_free_path";
pub const SHEET_HASH: u64 = 0xb898a6fb2272a28b;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "env_number_density",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["env_mean_free_path"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Length::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 1 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let n: NumberDensity = NumberDensity::new(inputs[0]);
    let answer = super::model::evaluate(n)?;
    outputs[0] = answer.get();
    Ok(())
}
