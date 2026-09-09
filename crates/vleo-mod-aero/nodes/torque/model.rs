// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much torque does the drag apply about the centre of mass?
///
/// `T = D*x_cp`
///
/// Source: `larson_wertz`
///
/// In VLEO this dominates gravity gradient and solar pressure by one to three
/// orders of magnitude, which is why a control design carried over from a
/// higher orbit is undersized here.
pub const NODE_ID: &str = "aero_torque";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x8449f27f0b1a807d;

pub fn evaluate(d: Force, x: Length) -> Result<Torque, Fault> {
    // ---- HOLE 1 : multiply the drag force by the centre-of-pressure offset -> Torque
    let t: Torque = aero::aerodynamic_torque(d, x);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Torque = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_aero", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_aero", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Torque::UNIT, reason: "a torque magnitude cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_aero", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Torque::UNIT, reason: "above 1 N.m no reaction wheel that fits this vehicle can hold attitude" });
    }
    Ok(answer)
}
