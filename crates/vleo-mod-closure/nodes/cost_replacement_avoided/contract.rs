// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `C_avd` (Replacement cost avoided), in `MUSD`.
pub const NODE_ID: &str = "cost_replacement_avoided";
pub const SHEET_HASH: u64 = 0xd78b109d944002ff;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "cost_bus_recurring",
    "cost_payload_recurring",
    "cost_launch_per_satellite",
    "mis_satellites",
    "prop_thrust_to_drag",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["cost_replacement_avoided"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Money::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Money::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 5 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let cb: Money = Money::new(inputs[0]);
    let cp: Money = Money::new(inputs[1]);
    let lc: Money = Money::new(inputs[2]);
    let n: Ratio = Ratio::new(inputs[3]);
    let td: Ratio = Ratio::new(inputs[4]);
    let answer = super::model::evaluate(cb, cp, lc, n, td)?;
    outputs[0] = answer.get();
    Ok(())
}
