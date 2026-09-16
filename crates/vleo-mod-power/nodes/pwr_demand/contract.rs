// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `P_dem` (Total bus power demand), in `W`.
pub const NODE_ID: &str = "pwr_demand";
pub const SHEET_HASH: u64 = 0xc6b354710ef121b4;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "prop_delivered_bus_power",
    "pay_power",
    "pwr_avionics_load",
    "com_tx_power",
    "pwr_thermal_load",
    "pwr_harness_loss",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["pwr_demand"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Power::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Power::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 6 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let pp: Power = Power::new(inputs[0]);
    let pay: Power = Power::new(inputs[1]);
    let av: Power = Power::new(inputs[2]);
    let com: Power = Power::new(inputs[3]);
    let th: Power = Power::new(inputs[4]);
    let lh: Ratio = Ratio::new(inputs[5]);
    let answer = super::model::evaluate(pp, pay, av, com, th, lh)?;
    outputs[0] = answer.get();
    Ok(())
}
