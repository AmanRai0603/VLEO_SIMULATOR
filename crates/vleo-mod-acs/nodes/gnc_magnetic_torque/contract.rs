// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `T_mag` (Residual dipole torque), in `N.m`.
pub const NODE_ID: &str = "gnc_magnetic_torque";
pub const SHEET_HASH: u64 = 0x2239c901b93a477a;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "gnc_residual_dipole",
    "env_magnetic_field",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["gnc_magnetic_torque"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Torque::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Torque::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 2 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let m: DipoleMoment = DipoleMoment::new(inputs[0]);
    let b: MagneticFluxDensity = MagneticFluxDensity::new(inputs[1]);
    let answer = super::model::evaluate(m, b)?;
    outputs[0] = answer.get();
    Ok(())
}
