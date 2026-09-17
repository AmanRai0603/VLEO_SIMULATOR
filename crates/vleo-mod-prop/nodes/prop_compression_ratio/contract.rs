// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `CR` (Intake compression ratio), in `-`.
pub const NODE_ID: &str = "prop_compression_ratio";
pub const SHEET_HASH: u64 = 0x11c51ef5b8338a0c;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "prop_intake_area",
    "prop_throat_area",
    "prop_eta_geo",
    "prop_beta_backflow",
    "env_chamber_temperature",
    "env_mean_molar_mass",
    "env_number_density",
    "orbit_velocity",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["prop_compression_ratio"];
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
    let a_in: Area = Area::new(inputs[0]);
    let a_out: Area = Area::new(inputs[1]);
    let eta_geo: Ratio = Ratio::new(inputs[2]);
    let beta: Ratio = Ratio::new(inputs[3]);
    let t_c: Temperature = Temperature::new(inputs[4]);
    let m: MolarMass = MolarMass::new(inputs[5]);
    let n: NumberDensity = NumberDensity::new(inputs[6]);
    let v: Velocity = Velocity::new(inputs[7]);
    let answer = super::model::evaluate(a_in, a_out, eta_geo, beta, t_c, m, n, v)?;
    outputs[0] = answer.get();
    Ok(())
}
