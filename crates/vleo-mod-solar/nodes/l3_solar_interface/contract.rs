// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `F107_hotmean` (Solar weather — subsystem interface), in `-`.
pub const NODE_ID: &str = "l3_solar_interface";
pub const SHEET_HASH: u64 = 0x7bfa34ea86713460;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "sw_central_expectation",
    "sw_f107_design_long",
    "sw_f107_cold_long",
    "sw_f107_design_short",
    "sw_f107_cold_short",
    "sw_ap_central_expectation",
    "sw_ap_design_long",
    "sw_ap_cold_long",
    "sw_ap_design_short",
    "sw_ap_cold_short",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["l3_solar_interface", "l3_solar_interface.f107_nominal", "l3_solar_interface.f107_coldmean", "l3_solar_interface.f107_hotday", "l3_solar_interface.f107_coldday", "l3_solar_interface.f107bar_nominal", "l3_solar_interface.f107bar_hotmean", "l3_solar_interface.f107bar_coldmean", "l3_solar_interface.f107bar_hotday", "l3_solar_interface.f107bar_coldday", "l3_solar_interface.ap_nominal", "l3_solar_interface.ap_hotmean", "l3_solar_interface.ap_coldmean", "l3_solar_interface.ap_hotday", "l3_solar_interface.ap_coldday"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Ratio::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT, Ratio::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 10 || outputs.len() < 15 {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let f107_centre: Ratio = Ratio::new(inputs[0]);
    let f107_hot_long: Ratio = Ratio::new(inputs[1]);
    let f107_cold_long: Ratio = Ratio::new(inputs[2]);
    let f107_hot_day: Ratio = Ratio::new(inputs[3]);
    let f107_cold_day: Ratio = Ratio::new(inputs[4]);
    let ap_centre: Ratio = Ratio::new(inputs[5]);
    let ap_hot_long: Ratio = Ratio::new(inputs[6]);
    let ap_cold_long: Ratio = Ratio::new(inputs[7]);
    let ap_hot_day: Ratio = Ratio::new(inputs[8]);
    let ap_cold_day: Ratio = Ratio::new(inputs[9]);
    let answer = super::model::evaluate(f107_centre, f107_hot_long, f107_cold_long, f107_hot_day, f107_cold_day, ap_centre, ap_hot_long, ap_cold_long, ap_hot_day, ap_cold_day)?;
    outputs[0] = answer.F107_hotmean.get();
    outputs[1] = answer.F107_nominal.get();
    outputs[2] = answer.F107_coldmean.get();
    outputs[3] = answer.F107_hotday.get();
    outputs[4] = answer.F107_coldday.get();
    outputs[5] = answer.F107bar_nominal.get();
    outputs[6] = answer.F107bar_hotmean.get();
    outputs[7] = answer.F107bar_coldmean.get();
    outputs[8] = answer.F107bar_hotday.get();
    outputs[9] = answer.F107bar_coldday.get();
    outputs[10] = answer.Ap_nominal.get();
    outputs[11] = answer.Ap_hotmean.get();
    outputs[12] = answer.Ap_coldmean.get();
    outputs[13] = answer.Ap_hotday.get();
    outputs[14] = answer.Ap_coldday.get();
    Ok(())
}
