// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `sig_pt` (Pointing error, three sigma), in `arcsec`.
pub const NODE_ID: &str = "gnc_pointing_error";
pub const SHEET_HASH: u64 = 0xb9d0fc83eea02e71;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "gnc_sensor_noise",
    "gnc_alignment_error",
    "gnc_control_error",
    "gnc_thermal_distortion",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["gnc_pointing_error"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Angle::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 4 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let sen: Angle = Angle::new(inputs[0]);
    let ali: Angle = Angle::new(inputs[1]);
    let ctl: Angle = Angle::new(inputs[2]);
    let thm: Angle = Angle::new(inputs[3]);
    let answer = super::model::evaluate(sen, ali, ctl, thm)?;
    outputs[0] = answer.get();
    Ok(())
}
