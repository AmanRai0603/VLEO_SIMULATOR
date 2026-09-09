// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `T` (Thrust), in `mN`.
pub const NODE_ID: &str = "prop_thrust";
pub const SHEET_HASH: u64 = 0x59520bfdfec2a689;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "prop_ion_flow",
    "prop_exhaust_velocity",
    "prop_div_efficiency",
    "prop_double_ion_factor",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["prop_thrust"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Force::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 4 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let mi: MassFlow = MassFlow::new(inputs[0]);
    let ve: Velocity = Velocity::new(inputs[1]);
    let ad: Ratio = Ratio::new(inputs[2]);
    let ap: Ratio = Ratio::new(inputs[3]);
    let answer = super::model::evaluate(mi, ve, ad, ap)?;
    outputs[0] = answer.get();
    Ok(())
}
