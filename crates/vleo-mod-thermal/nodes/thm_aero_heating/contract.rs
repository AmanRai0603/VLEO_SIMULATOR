// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `Q_aero` (Free-molecular aerodynamic heating), in `W`.
pub const NODE_ID: &str = "thm_aero_heating";
pub const SHEET_HASH: u64 = 0xaa3d9a875436e9ee;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "env_mass_density",
    "orbit_velocity",
    "aero_frontal_area",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["thm_aero_heating"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Power::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let rho: MassDensity = MassDensity::new(inputs[0]);
    let v: Velocity = Velocity::new(inputs[1]);
    let a: Area = Area::new(inputs[2]);
    let answer = super::model::evaluate(rho, v, a)?;
    outputs[0] = answer.get();
    Ok(())
}
