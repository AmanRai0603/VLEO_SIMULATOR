// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does developing the design cost, once?
///
/// `C = a*m_dry^b, escalated`
///
/// Source: `nasa_cer`
pub const NODE_ID: &str = "cost_non_recurring";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x60c7ebf109488d93;

pub fn evaluate(m: Mass, i: Ratio) -> Result<Money, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::cost_non_recurring. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Money = match methods::cost_non_recurring::evaluate(m.get(), i.get()) {
        Ok(v) => Money::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "C_nre")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Money = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "C_nre", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_nre", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Money::UNIT, reason: "a cost cannot be negative" });
    }
    if answer.get() > 100000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_nre", value: answer.get(), bound: 100000000000.0, edge: Edge::Upper, unit: Money::UNIT, reason: "above 100 billion the relation is being fed a mass it was not fitted on" });
    }
    Ok(answer)
}
