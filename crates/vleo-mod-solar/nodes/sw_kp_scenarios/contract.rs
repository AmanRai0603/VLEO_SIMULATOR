// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `Kp_peak_hotday` (Kp in both slots, for all five scenarios), in `-`.
pub const NODE_ID: &str = "sw_kp_scenarios";
pub const SHEET_HASH: u64 = 0x50102f9afbc92977;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "sw_ap_central_expectation",
    "sw_ap_design_long",
    "sw_ap_cold_long",
    "sw_ap_design_short",
    "sw_ap_cold_short",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["sw_kp_scenarios", "sw_kp_scenarios.kp_mean_nominal", "sw_kp_scenarios.kp_mean_hotmean", "sw_kp_scenarios.kp_mean_coldmean", "sw_kp_scenarios.kp_mean_hotday", "sw_kp_scenarios.kp_mean_coldday", "sw_kp_scenarios.kp_peak_nominal", "sw_kp_scenarios.kp_peak_hotmean", "sw_kp_scenarios.kp_peak_coldmean", "sw_kp_scenarios.kp_peak_coldday"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 5 || outputs.len() < 10 {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let ap_nominal: Ratio = Ratio::new(inputs[0]);
    let ap_hotmean: Ratio = Ratio::new(inputs[1]);
    let ap_coldmean: Ratio = Ratio::new(inputs[2]);
    let ap_hotday: Ratio = Ratio::new(inputs[3]);
    let ap_coldday: Ratio = Ratio::new(inputs[4]);
    let answer = super::model::evaluate(ap_nominal, ap_hotmean, ap_coldmean, ap_hotday, ap_coldday)?;
    outputs[0] = answer.Kp_peak_hotday.get();
    outputs[1] = answer.Kp_mean_nominal.get();
    outputs[2] = answer.Kp_mean_hotmean.get();
    outputs[3] = answer.Kp_mean_coldmean.get();
    outputs[4] = answer.Kp_mean_hotday.get();
    outputs[5] = answer.Kp_mean_coldday.get();
    outputs[6] = answer.Kp_peak_nominal.get();
    outputs[7] = answer.Kp_peak_hotmean.get();
    outputs[8] = answer.Kp_peak_coldmean.get();
    outputs[9] = answer.Kp_peak_coldday.get();
    Ok(())
}
