// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `G_r` (Ground station antenna gain), in `dB`.
pub const NODE_ID: &str = "com_gs_gain";
pub const SHEET_HASH: u64 = 0xe2fb8c5b1bba699a;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "com_gs_antenna_diameter",
    "com_frequency",
    "com_antenna_efficiency",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["com_gs_gain"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let d: Length = Length::new(inputs[0]);
    let f: Frequency = Frequency::new(inputs[1]);
    let e: Ratio = Ratio::new(inputs[2]);
    let answer = super::model::evaluate(d, f, e)?;
    outputs[0] = answer.get();
    Ok(())
}
