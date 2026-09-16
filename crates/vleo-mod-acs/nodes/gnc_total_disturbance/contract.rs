// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `T_dis` (Total disturbance torque), in `N.m`.
pub const NODE_ID: &str = "gnc_total_disturbance";
pub const SHEET_HASH: u64 = 0xe43e6a3d4530dadb;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "aero_torque",
    "gnc_gravity_gradient_torque",
    "gnc_solar_torque",
    "gnc_magnetic_torque",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["gnc_total_disturbance"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Torque::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Torque::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 4 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let ta: Torque = Torque::new(inputs[0]);
    let tg: Torque = Torque::new(inputs[1]);
    let ts: Torque = Torque::new(inputs[2]);
    let tm: Torque = Torque::new(inputs[3]);
    let answer = super::model::evaluate(ta, tg, ts, tm)?;
    outputs[0] = answer.get();
    Ok(())
}
