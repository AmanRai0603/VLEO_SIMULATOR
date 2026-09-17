// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `V_g` (Ground track speed), in `m/s`.
pub const NODE_ID: &str = "orbit_ground_track_speed";
pub const SHEET_HASH: u64 = 0xe15473059019bed1;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "orbit_velocity",
    "orbit_radius",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["orbit_ground_track_speed"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Velocity::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Velocity::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let v: Velocity = Velocity::new(inputs[0]);
    let r: Length = Length::new(inputs[1]);
    let answer = super::model::evaluate(v, r)?;
    outputs[0] = answer.get();
    Ok(())
}
