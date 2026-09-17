// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `C_tot` (Total programme cost), in `MUSD`.
pub const NODE_ID: &str = "cost_programme";
pub const SHEET_HASH: u64 = 0xcf81c6a1039de7b8;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "cost_non_recurring",
    "cost_production",
    "cost_launch_per_satellite",
    "mis_satellites",
    "cost_annual_operations",
    "orbit_mission_duration",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["cost_programme"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Money::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Money::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 6 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let nre: Money = Money::new(inputs[0]);
    let pr: Money = Money::new(inputs[1]);
    let lc: Money = Money::new(inputs[2]);
    let n: Ratio = Ratio::new(inputs[3]);
    let op: Money = Money::new(inputs[4]);
    let y: Time = Time::new(inputs[5]);
    let answer = super::model::evaluate(nre, pr, lc, n, op, y)?;
    outputs[0] = answer.get();
    Ok(())
}
