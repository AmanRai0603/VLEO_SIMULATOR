// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `s` (Molecular speed ratio), in `-`.
pub const NODE_ID: &str = "aero_speed_ratio";
pub const SHEET_HASH: u64 = 0x9a432528dfbb8276;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "orbit_velocity",
    "env_local_temperature",
    "env_mean_molar_mass",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["aero_speed_ratio"];
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
    let v: Velocity = Velocity::new(inputs[0]);
    let t: Temperature = Temperature::new(inputs[1]);
    let m: MolarMass = MolarMass::new(inputs[2]);
    let answer = super::model::evaluate(v, t, m)?;
    outputs[0] = answer.get();
    Ok(())
}
