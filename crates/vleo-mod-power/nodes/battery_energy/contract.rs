// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `E_batt` (Battery energy required), in `Wh`.
pub const NODE_ID: &str = "pwr_battery_energy";
pub const SHEET_HASH: u64 = 0xeae3279078fd1cbf;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "pwr_demand",
    "orbit_eclipse_fraction",
    "orbit_period",
    "pwr_dod",
    "pwr_discharge_efficiency",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["pwr_battery_energy"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Energy::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 5 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let dem: Power = Power::new(inputs[0]);
    let fe: Ratio = Ratio::new(inputs[1]);
    let t: Time = Time::new(inputs[2]);
    let dod: Ratio = Ratio::new(inputs[3]);
    let ed: Ratio = Ratio::new(inputs[4]);
    let answer = super::model::evaluate(dem, fe, t, dod, ed)?;
    outputs[0] = answer.get();
    Ok(())
}
