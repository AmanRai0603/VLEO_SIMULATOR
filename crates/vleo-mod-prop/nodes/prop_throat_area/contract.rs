// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `A_out` (Intake throat area), in `m^2`.
pub const NODE_ID: &str = "prop_throat_area";
pub const SHEET_HASH: u64 = 0xf743d1b68b0e6f8f;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["prop_throat_area"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Area::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Area::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let answer = super::model::evaluate()?;
    outputs[0] = answer.get();
    Ok(())
}
