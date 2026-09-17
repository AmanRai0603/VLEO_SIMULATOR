// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `Cd` (Drag coefficient), in `-`.
pub const NODE_ID: &str = "aero_drag_coefficient";
pub const SHEET_HASH: u64 = 0x05d4235d29c026dd;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "aero_speed_ratio",
    "aero_body_length",
    "aero_body_diameter",
    "aero_accommodation",
    "env_wall_temperature",
    "orbit_velocity",
    "env_mean_molar_mass",
    "env_knudsen",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["aero_drag_coefficient"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Ratio::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 8 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let s: Ratio = Ratio::new(inputs[0]);
    let l: Length = Length::new(inputs[1]);
    let d: Length = Length::new(inputs[2]);
    let alpha: Ratio = Ratio::new(inputs[3]);
    let t_w: Temperature = Temperature::new(inputs[4]);
    let v: Velocity = Velocity::new(inputs[5]);
    let m: MolarMass = MolarMass::new(inputs[6]);
    let kn: Ratio = Ratio::new(inputs[7]);
    let answer = super::model::evaluate(s, l, d, alpha, t_w, v, m, kn)?;
    outputs[0] = answer.get();
    Ok(())
}
