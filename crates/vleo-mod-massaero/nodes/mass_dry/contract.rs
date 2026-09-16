// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_variables)]

use vleo_core::fault::Fault;
use vleo_core::units::*;

/// What this node publishes: `m_dry` (Dry mass), in `kg`.
pub const NODE_ID: &str = "mass_dry";
pub const SHEET_HASH: u64 = 0xbfe6d4527b361563;
/// The variables this node reads, in the order `call` expects them.
pub const INPUT_VARS: &[&str] = &[
    "mass_structure",
    "mass_propulsion_hw",
    "pwr_subsystem_mass",
    "mass_harness",
    "mass_avionics",
    "mass_comms_hw",
    "mass_gnc_hw",
    "mass_payload",
    "mass_system_margin",
];
/// The variables this node publishes.
pub const OUTPUT_VARS: &[&str] = &["mass_dry"];
/// The SI unit this node's own answer crosses the boundary in.
pub const OUTPUT_UNIT: Unit = Mass::UNIT;
/// The SI unit of each published variable, in `OUTPUT_VARS` order.
pub const OUTPUT_UNITS: &[Unit] = &[Mass::UNIT];

/// The untyped adapter. Values cross as SI `f64` and are re-typed here,
/// so the bus carries no quantity types and a face cannot pass arguments
/// in the wrong order.
pub fn call(inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
    if inputs.len() < 9 || outputs.is_empty() {
        return Err(Fault::Blocked { node: NODE_ID, missing: "an input the contract declares" });
    }
    let st: Mass = Mass::new(inputs[0]);
    let pr: Mass = Mass::new(inputs[1]);
    let pw: Mass = Mass::new(inputs[2]);
    let th: Mass = Mass::new(inputs[3]);
    let av: Mass = Mass::new(inputs[4]);
    let cm: Mass = Mass::new(inputs[5]);
    let gn: Mass = Mass::new(inputs[6]);
    let pa: Mass = Mass::new(inputs[7]);
    let mg: Ratio = Ratio::new(inputs[8]);
    let answer = super::model::evaluate(st, pr, pw, th, av, cm, gn, pa, mg)?;
    outputs[0] = answer.get();
    Ok(())
}
