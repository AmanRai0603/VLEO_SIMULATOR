// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `GSD_d` (Diffraction-limited ground sample), in `m`.
pub const NODE_ID: &str = "pay_gsd_diffraction";
pub const SHEET_HASH: u64 = 0x61dfbce5e713c72c;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "orbit_altitude",
    "pay_wavelength",
    "pay_aperture",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["pay_gsd_diffraction"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Length::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let h: Length = Length::new(inputs[0]);
    let lam: Length = Length::new(inputs[1]);
    let d: Length = Length::new(inputs[2]);
    let answer = super::model::evaluate(h, lam, d)?;
    outputs[0] = answer.get();
    Ok(())
}
