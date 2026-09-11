// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `N_e` (Signal electrons per sample), in `-`.
pub const NODE_ID: &str = "pay_signal_electrons";
pub const SHEET_HASH: u64 = 0x121be8c520a01b03;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "pay_radiance",
    "pay_aperture",
    "pay_focal_length",
    "pay_pixel_pitch",
    "pay_transmission",
    "pay_quantum_efficiency",
    "pay_dwell_time",
    "pay_wavelength",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["pay_signal_electrons"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 8 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let l: Ratio = Ratio::new(inputs[0]);
    let d: Length = Length::new(inputs[1]);
    let f: Length = Length::new(inputs[2]);
    let p: Length = Length::new(inputs[3]);
    let tau: Ratio = Ratio::new(inputs[4]);
    let qe: Ratio = Ratio::new(inputs[5]);
    let t: Time = Time::new(inputs[6]);
    let lam: Length = Length::new(inputs[7]);
    let answer = super::model::evaluate(l, d, f, p, tau, qe, t, lam)?;
    outputs[0] = answer.get();
    Ok(())
}
