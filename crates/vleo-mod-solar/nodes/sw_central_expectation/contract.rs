// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `F107_central` (F10.7 central expectation at a lead), in `-`.
pub const NODE_ID: &str = "sw_central_expectation";
pub const SHEET_HASH: u64 = 0xdfed80b967f888cd;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "sw_f107_observed",
    "orbit_mission_duration",
    "sys_mission_requirements_mission_epoch",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["sw_central_expectation"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Ratio::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let today: Ratio = Ratio::new(inputs[0]);
    let lead: Time = Time::new(inputs[1]);
    let epoch: Time = Time::new(inputs[2]);
    let answer = super::model::evaluate(today, lead, epoch)?;
    outputs[0] = answer.get();
    Ok(())
}
