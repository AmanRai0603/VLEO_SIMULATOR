// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `F_AO` (Atomic oxygen fluence), in `-`.
pub const NODE_ID: &str = "aero_ao_fluence";
pub const SHEET_HASH: u64 = 0x5bf5b881c612f42f;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "env_atomic_oxygen_density",
    "orbit_velocity",
    "orbit_mission_duration",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["aero_ao_fluence"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let n_o: NumberDensity = NumberDensity::new(inputs[0]);
    let v: Velocity = Velocity::new(inputs[1]);
    let t: Time = Time::new(inputs[2]);
    let answer = super::model::evaluate(n_o, v, t)?;
    outputs[0] = answer.get();
    Ok(())
}
