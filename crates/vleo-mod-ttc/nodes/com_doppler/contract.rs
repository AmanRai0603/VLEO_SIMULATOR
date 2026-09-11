// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `df` (Doppler shift at closest approach), in `MHz`.
pub const NODE_ID: &str = "com_doppler";
pub const SHEET_HASH: u64 = 0x59e27edeff3abd6a;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "com_frequency",
    "orbit_ground_track_speed",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["com_doppler"];
/// The SI unit every value crossing this boundary is expressed in.
pub const OUTPUT_UNIT: Unit = Frequency::UNIT;

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let f: Frequency = Frequency::new(inputs[0]);
    let v: Velocity = Velocity::new(inputs[1]);
    let answer = super::model::evaluate(f, v)?;
    outputs[0] = answer.get();
    Ok(())
}
