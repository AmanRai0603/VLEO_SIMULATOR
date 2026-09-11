// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `q` (Dynamic pressure), in `Pa`.
pub const NODE_ID: &str = "aero_dynamic_pressure";
pub const SHEET_HASH: u64 = 0x93ebee77dae44a8b;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "env_mass_density",
    "orbit_velocity",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["aero_dynamic_pressure"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Pressure::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let rho: MassDensity = MassDensity::new(inputs[0]);
    let v: Velocity = Velocity::new(inputs[1]);
    let answer = super::model::evaluate(rho, v)?;
    outputs[0] = answer.get();
    Ok(())
}
