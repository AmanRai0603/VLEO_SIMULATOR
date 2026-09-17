// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `sig_tau` (Time of arrival uncertainty), in `s`.
pub const NODE_ID: &str = "pay_toa_uncertainty";
pub const SHEET_HASH: u64 = 0x1b8751e76d2e38da;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "pay_rf_bandwidth",
    "pay_rf_snr",
    "pay_rf_integration",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["pay_toa_uncertainty"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Time::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Time::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let b: Frequency = Frequency::new(inputs[0]);
    let s: Ratio = Ratio::new(inputs[1]);
    let t: Time = Time::new(inputs[2]);
    let answer = super::model::evaluate(b, s, t)?;
    outputs[0] = answer.get();
    Ok(())
}
