// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `T_eq` (Equilibrium temperature), in `K`.
pub const NODE_ID: &str = "thm_equilibrium_temperature";
pub const SHEET_HASH: u64 = 0x0b60557bf95b3e31;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "thm_absorbed_solar",
    "thm_absorbed_albedo",
    "thm_absorbed_ir",
    "thm_aero_heating",
    "thm_dissipation",
    "thm_radiator_area",
    "thm_emissivity",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["thm_equilibrium_temperature"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Temperature::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Temperature::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 7 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let qs: Power = Power::new(inputs[0]);
    let qa: Power = Power::new(inputs[1]);
    let qi: Power = Power::new(inputs[2]);
    let qh: Power = Power::new(inputs[3]);
    let qd: Power = Power::new(inputs[4]);
    let ar: Area = Area::new(inputs[5]);
    let e: Ratio = Ratio::new(inputs[6]);
    let answer = super::model::evaluate(qs, qa, qi, qh, qd, ar, e)?;
    outputs[0] = answer.get();
    Ok(())
}
