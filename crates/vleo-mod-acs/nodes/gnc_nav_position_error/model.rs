// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How well does the spacecraft know where it is?
///
/// `e = URE*GDOP`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_nav_position_error";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe4e3864c0bf88ba4;

pub fn evaluate(u: Length, g: Ratio) -> Result<Length, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::gnc_nav_position_error. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Length = match methods::gnc_nav_position_error::evaluate(u.get(), g.get()) {
        Ok(v) => Length::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "e_nav")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "e_nav", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e_nav", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Length::UNIT, reason: "a position error cannot be negative" });
    }
    if answer.get() > 10000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e_nav", value: answer.get(), bound: 10000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 10 km the navigation solution is not usable" });
    }
    Ok(answer)
}
