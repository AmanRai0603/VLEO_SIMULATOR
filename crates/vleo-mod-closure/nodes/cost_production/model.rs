// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does building the whole constellation cost?
///
/// `C = sum over n of T1*n^log2(b)`
///
/// Source: `nasa_cer`
pub const NODE_ID: &str = "cost_production";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x123e5bb3608c52fe;

pub fn evaluate(cb: Money, cp: Money, n: Ratio, b: Ratio) -> Result<Money, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::cost_production. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Money = match methods::cost_production::evaluate(cb.get(), cp.get(), n.get(), b.get()) {
        Ok(v) => Money::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "C_prod")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Money = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "C_prod", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_prod", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Money::UNIT, reason: "a cost cannot be negative" });
    }
    if answer.get() > 1000000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_prod", value: answer.get(), bound: 1000000000000.0, edge: Edge::Upper, unit: Money::UNIT, reason: "above a trillion dollars the programme is not the one being designed" });
    }
    Ok(answer)
}
