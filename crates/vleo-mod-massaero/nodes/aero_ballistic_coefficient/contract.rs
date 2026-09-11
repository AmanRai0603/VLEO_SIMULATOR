// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `BC` (Ballistic coefficient), in `-`.
pub const NODE_ID: &str = "aero_ballistic_coefficient";
pub const SHEET_HASH: u64 = 0x7512f2f652dac3fa;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "mass_wet",
    "aero_drag_coefficient",
    "aero_frontal_area",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["aero_ballistic_coefficient"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let m: Mass = Mass::new(inputs[0]);
    let cd: Ratio = Ratio::new(inputs[1]);
    let a: Area = Area::new(inputs[2]);
    let answer = super::model::evaluate(m, cd, a)?;
    outputs[0] = answer.get();
    Ok(())
}
