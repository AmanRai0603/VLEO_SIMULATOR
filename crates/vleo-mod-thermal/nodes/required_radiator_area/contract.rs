// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `A_req` (Radiator area required), in `m^2`.
pub const NODE_ID: &str = "thm_required_radiator_area";
pub const SHEET_HASH: u64 = 0x79de83d4426eea8c;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "thm_dissipation",
    "thm_emissivity",
    "thm_temperature_limit",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["thm_required_radiator_area"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Area::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let q: Power = Power::new(inputs[0]);
    let e: Ratio = Ratio::new(inputs[1]);
    let lim: Temperature = Temperature::new(inputs[2]);
    let answer = super::model::evaluate(q, e, lim)?;
    outputs[0] = answer.get();
    Ok(())
}
