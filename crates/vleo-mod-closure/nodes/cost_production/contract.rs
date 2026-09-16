// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `C_prod` (Production run cost), in `MUSD`.
pub const NODE_ID: &str = "cost_production";
pub const SHEET_HASH: u64 = 0xe767cda4aeb4290c;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "cost_bus_recurring",
    "cost_payload_recurring",
    "mis_satellites",
    "cost_learning_slope",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["cost_production"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Money::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Money::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 4 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let cb: Money = Money::new(inputs[0]);
    let cp: Money = Money::new(inputs[1]);
    let n: Ratio = Ratio::new(inputs[2]);
    let b: Ratio = Ratio::new(inputs[3]);
    let answer = super::model::evaluate(cb, cp, n, b)?;
    outputs[0] = answer.get();
    Ok(())
}
