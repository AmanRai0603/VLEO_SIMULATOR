// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `Q_sol` (Absorbed direct solar), in `W`.
pub const NODE_ID: &str = "thm_absorbed_solar";
pub const SHEET_HASH: u64 = 0x4c52eac00fa26eab;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "aero_frontal_area",
    "thm_absorptivity",
    "pwr_array_incidence",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["thm_absorbed_solar"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Power::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Power::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let a: Area = Area::new(inputs[0]);
    let al: Ratio = Ratio::new(inputs[1]);
    let th: Angle = Angle::new(inputs[2]);
    let answer = super::model::evaluate(a, al, th)?;
    outputs[0] = answer.get();
    Ok(())
}
