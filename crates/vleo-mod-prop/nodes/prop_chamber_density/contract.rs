// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `n_c` (Collection chamber number density), in `1/m^3`.
pub const NODE_ID: &str = "prop_chamber_density";
pub const SHEET_HASH: u64 = 0x5b3e5a6c2a6391cd;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "env_number_density",
    "prop_compression_ratio",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["prop_chamber_density"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = NumberDensity::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[NumberDensity::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let n: NumberDensity = NumberDensity::new(inputs[0]);
    let cr: Ratio = Ratio::new(inputs[1]);
    let answer = super::model::evaluate(n, cr)?;
    outputs[0] = answer.get();
    Ok(())
}
