// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `mdot` (Collected mass flow), in `mg/s`.
pub const NODE_ID: &str = "prop_collected_flow";
pub const SHEET_HASH: u64 = 0x80ca74de8b3c4880;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "env_mass_density",
    "orbit_velocity",
    "prop_intake_area",
    "prop_capture_efficiency",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["prop_collected_flow"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = MassFlow::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 4 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let rho: MassDensity = MassDensity::new(inputs[0]);
    let v: Velocity = Velocity::new(inputs[1]);
    let a_in: Area = Area::new(inputs[2]);
    let eta_c: Ratio = Ratio::new(inputs[3]);
    let answer = super::model::evaluate(rho, v, a_in, eta_c)?;
    outputs[0] = answer.get();
    Ok(())
}
