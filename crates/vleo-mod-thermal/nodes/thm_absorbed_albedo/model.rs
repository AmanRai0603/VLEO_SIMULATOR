// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much reflected sunlight off the Earth does the spacecraft absorb?
///
/// `Q = S*A*alpha_s*a_E*F`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "thm_absorbed_albedo";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x666756364829fdd7;

pub fn evaluate(a: Area, al: Ratio, f: Ratio) -> Result<Power, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::thm_absorbed_albedo. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Power = match methods::thm_absorbed_albedo::evaluate(a.get(), al.get(), f.get()) {
        Ok(v) => Power::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "Q_alb")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Q_alb", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Q_alb", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "absorbed power cannot be negative" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Q_alb", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 100 kW the illuminated area is not this spacecraft" });
    }
    Ok(answer)
}
