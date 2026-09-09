// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much thrust is produced once the power available is taken into account?
///
/// `T = k*T`
///
/// Source: `orbitt_case_c1`
///
/// # Assumptions
///
/// * Thrust scales linearly with delivered power at fixed specific impulse — fails when a real thruster's efficiency falls away from its design point, so a deeply throttled system delivers less than this; the relation is optimistic below about half throttle
pub const NODE_ID: &str = "prop_delivered_thrust";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x11173687f5907800;

pub fn evaluate(t: Force, k: Ratio) -> Result<Force, Fault> {
    // ---- HOLE 1 : scale thrust with the throttle at fixed specific impulse -> Force
    let d: Force = t * k.get();
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Force = d;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_del", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_del", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Force::UNIT, reason: "thrust cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_del", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Force::UNIT, reason: "above 1 N no air-breathing thruster in this band produces thrust" });
    }
    Ok(answer)
}
