// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `Isp` (Specific impulse), in `s`.
pub const NODE_ID: &str = "prop_specific_impulse";
pub const SHEET_HASH: u64 = 0x4166b581dfb0dbaa;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "prop_thrust",
    "prop_collected_flow",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["prop_specific_impulse"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Time::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let t: Force = Force::new(inputs[0]);
    let md: MassFlow = MassFlow::new(inputs[1]);
    let answer = super::model::evaluate(t, md)?;
    outputs[0] = answer.get();
    Ok(())
}
