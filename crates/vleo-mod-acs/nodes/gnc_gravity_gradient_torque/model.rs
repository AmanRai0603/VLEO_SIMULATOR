// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much torque does the gravity gradient apply?
///
/// `T = (3*mu/(2*r^3))*|Iz - Iy|*sin(2*theta)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_gravity_gradient_torque";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x26207e14190ca771;

pub fn evaluate(r: Length, imax: Ratio, imin: Ratio) -> Result<Torque, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::gnc_gravity_gradient_torque. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Torque = match methods::gnc_gravity_gradient_torque::evaluate(r.get(), imax.get(), imin.get()) {
        Ok(v) => Torque::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "T_gg")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Torque = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_gg", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_gg", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Torque::UNIT, reason: "a torque magnitude cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_gg", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Torque::UNIT, reason: "above 1 N.m no wheel that fits this vehicle can hold attitude" });
    }
    Ok(answer)
}
