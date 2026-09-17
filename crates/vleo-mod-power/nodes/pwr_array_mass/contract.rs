// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `m_arr` (Array mass), in `kg`.
pub const NODE_ID: &str = "pwr_array_mass";
pub const SHEET_HASH: u64 = 0xfbc1c4f746a49e94;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "pwr_array_area",
    "pwr_array_areal_density",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["pwr_array_mass"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Mass::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Mass::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let a: Area = Area::new(inputs[0]);
    let s: Ratio = Ratio::new(inputs[1]);
    let answer = super::model::evaluate(a, s)?;
    outputs[0] = answer.get();
    Ok(())
}
