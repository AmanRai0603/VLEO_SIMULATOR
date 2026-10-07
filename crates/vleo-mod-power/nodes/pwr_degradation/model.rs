// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much of the array's beginning-of-life output survives to the end of the mission?
///
/// `f_deg = (1 - d_yr)^t`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pwr_degradation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xaa2e813cf1e37ac6;

pub fn evaluate(d: Ratio, t: Time) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::pwr_degradation. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::pwr_degradation::evaluate(d.get(), t.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "f_deg")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "f_deg", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "f_deg", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a degradation factor cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "f_deg", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "an array cannot end life producing more than it began with" });
    }
    Ok(answer)
}
