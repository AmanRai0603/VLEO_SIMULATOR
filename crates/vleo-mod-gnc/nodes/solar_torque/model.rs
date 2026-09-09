// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much torque does solar radiation pressure apply?
///
/// `T = (S/c)*A*(1+q)*cos(theta)*x_cp`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_solar_torque";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x9f49b5cc78db9d4e;

pub fn evaluate(a: Area, x: Length, th: Angle) -> Result<Torque, Fault> {
    // ---- HOLE 1 : apply solar radiation pressure over the illuminated area with a reflectivity of 0.6 -> Torque
    let t: Torque = gnc::solar_pressure_torque(a, Ratio::new(0.6), x, th);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Torque = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_srp", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_srp", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Torque::UNIT, reason: "a torque magnitude cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_srp", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Torque::UNIT, reason: "above 1 N.m no wheel that fits this vehicle can hold attitude" });
    }
    Ok(answer)
}
