// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `h_req` (Momentum storage required), in `N.m.s`.
pub const NODE_ID: &str = "gnc_momentum_storage";
pub const SHEET_HASH: u64 = 0xbe5b3da4ce44bb9f;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "gnc_total_disturbance",
    "orbit_period",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["gnc_momentum_storage"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = AngularMomentum::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[AngularMomentum::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let t: Torque = Torque::new(inputs[0]);
    let p: Time = Time::new(inputs[1]);
    let answer = super::model::evaluate(t, p)?;
    outputs[0] = answer.get();
    Ok(())
}
