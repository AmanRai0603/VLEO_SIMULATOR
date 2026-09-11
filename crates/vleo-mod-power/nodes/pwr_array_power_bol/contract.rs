// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `P_bol` (Array power, beginning of life), in `W`.
pub const NODE_ID: &str = "pwr_array_power_bol";
pub const SHEET_HASH: u64 = 0x2bed5eea0d8280a8;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "pwr_array_area",
    "pwr_cell_efficiency",
    "pwr_packing",
    "pwr_array_incidence",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["pwr_array_power_bol"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Power::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 4 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let a: Area = Area::new(inputs[0]);
    let e: Ratio = Ratio::new(inputs[1]);
    let f: Ratio = Ratio::new(inputs[2]);
    let th: Angle = Angle::new(inputs[3]);
    let answer = super::model::evaluate(a, e, f, th)?;
    outputs[0] = answer.get();
    Ok(())
}
