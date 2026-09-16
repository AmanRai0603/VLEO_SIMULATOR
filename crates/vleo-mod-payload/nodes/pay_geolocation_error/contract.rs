// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `e_geo` (RF geolocation error), in `m`.
pub const NODE_ID: &str = "pay_geolocation_error";
pub const SHEET_HASH: u64 = 0x11554f19c2ed118f;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "pay_toa_uncertainty",
    "pay_rf_gdop",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["pay_geolocation_error"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Length::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Length::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let s: Time = Time::new(inputs[0]);
    let g: Ratio = Ratio::new(inputs[1]);
    let answer = super::model::evaluate(s, g)?;
    outputs[0] = answer.get();
    Ok(())
}
