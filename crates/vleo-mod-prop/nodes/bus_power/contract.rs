// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `P_prop` (Propulsion demand on the bus), in `W`.
pub const NODE_ID: &str = "prop_bus_power";
pub const SHEET_HASH: u64 = 0x5793a8c1487725ae;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "prop_input_power",
    "prop_ppu_efficiency",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["prop_bus_power"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Power::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let pin: Power = Power::new(inputs[0]);
    let eta: Ratio = Ratio::new(inputs[1]);
    let answer = super::model::evaluate(pin, eta)?;
    outputs[0] = answer.get();
    Ok(())
}
