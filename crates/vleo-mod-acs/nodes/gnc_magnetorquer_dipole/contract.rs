// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `m_req` (Magnetorquer dipole required), in `A.m^2`.
pub const NODE_ID: &str = "gnc_magnetorquer_dipole";
pub const SHEET_HASH: u64 = 0xf66190f61f3da001;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "gnc_momentum_storage",
    "env_magnetic_field",
    "gnc_dump_time",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["gnc_magnetorquer_dipole"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = DipoleMoment::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[DipoleMoment::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 3 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let h: AngularMomentum = AngularMomentum::new(inputs[0]);
    let b: MagneticFluxDensity = MagneticFluxDensity::new(inputs[1]);
    let td: Time = Time::new(inputs[2]);
    let answer = super::model::evaluate(h, b, td)?;
    outputs[0] = answer.get();
    Ok(())
}
