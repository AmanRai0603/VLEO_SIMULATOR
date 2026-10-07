// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the worst-case disturbance torque the control system has to answer?
///
/// `T = |T_aero| + |T_gg| + |T_srp| + |T_mag|`
///
/// Source: `larson_wertz`
///
/// Summed, not root-sum-squared. These are biases that can and do align, and
/// a wheel sized on their RSS saturates the first time they do.
pub const NODE_ID: &str = "gnc_total_disturbance";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xdd2495b0f1723502;

pub fn evaluate(ta: Torque, tg: Torque, ts: Torque, tm: Torque) -> Result<Torque, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::gnc_total_disturbance. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Torque = match methods::gnc_total_disturbance::evaluate(ta.get(), tg.get(), ts.get(), tm.get()) {
        Ok(v) => Torque::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "T_dis")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Torque = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_dis", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_dis", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Torque::UNIT, reason: "a torque magnitude cannot be negative" });
    }
    if answer.get() > 5.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_dis", value: answer.get(), bound: 5.0, edge: Edge::Upper, unit: Torque::UNIT, reason: "above 5 N.m the vehicle is uncontrollable by any actuator that fits it" });
    }
    Ok(answer)
}
