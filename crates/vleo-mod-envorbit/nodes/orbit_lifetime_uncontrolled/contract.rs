// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `t_life` (Uncontrolled orbital lifetime), in `d`.
pub const NODE_ID: &str = "orbit_lifetime_uncontrolled";
pub const SHEET_HASH: u64 = 0xbd35717f3d5b3da4;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "orbit_altitude",
    "aero_ballistic_coefficient",
    "env_exospheric_temperature",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["orbit_lifetime_uncontrolled"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Time::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let h: Length = Length::new(inputs[0]);
    let bc: Ratio = Ratio::new(inputs[1]);
    let t_inf: Temperature = Temperature::new(inputs[2]);
    let answer = super::model::evaluate(h, bc, t_inf)?;
    outputs[0] = answer.get();
    Ok(())
}
