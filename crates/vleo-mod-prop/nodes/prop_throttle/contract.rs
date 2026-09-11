// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `k_thr` (Propulsion throttle), in `-`.
pub const NODE_ID: &str = "prop_throttle";
pub const SHEET_HASH: u64 = 0xc16699a360a6c290;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "pwr_available",
    "prop_bus_power",
    "pay_power",
    "pwr_avionics_load",
    "com_tx_power",
    "pwr_thermal_load",
    "pwr_harness_loss",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["prop_throttle"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 7 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let av: Power = Power::new(inputs[0]);
    let pp: Power = Power::new(inputs[1]);
    let pay: Power = Power::new(inputs[2]);
    let ax: Power = Power::new(inputs[3]);
    let cm: Power = Power::new(inputs[4]);
    let th: Power = Power::new(inputs[5]);
    let lh: Ratio = Ratio::new(inputs[6]);
    let answer = super::model::evaluate(av, pp, pay, ax, cm, th, lh)?;
    outputs[0] = answer.get();
    Ok(())
}
