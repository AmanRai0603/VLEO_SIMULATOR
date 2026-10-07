// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much stored propellant must be carried for the disposal manoeuvre?
///
/// `m = m_dry*(exp(dv/v_e) - 1)`
///
/// Source: `iso24113`
///
/// An air-breathing design does not reach zero here. Debris mitigation is a
/// requirement, not a courtesy.
pub const NODE_ID: &str = "mass_disposal_propellant";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x755420b80af8a721;

pub fn evaluate(m: Mass, dv: Velocity) -> Result<Mass, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::mass_disposal_propellant. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Mass = match methods::mass_disposal_propellant::evaluate(m.get(), dv.get()) {
        Ok(v) => Mass::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "m_dis")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Mass = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "m_dis", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_dis", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Mass::UNIT, reason: "a propellant mass cannot be negative" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_dis", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: Mass::UNIT, reason: "above a tonne the disposal budget is larger than the spacecraft" });
    }
    Ok(answer)
}
