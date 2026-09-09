// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// By what factor does unit cost fall with each doubling of cumulative production?
///
/// `b_lc = 0.92`
///
/// Source: `nasa_cer`
///
/// The term that decides whether forty satellites cost forty times one, and
/// the term most concept estimates leave out.
pub const NODE_ID: &str = "cost_learning_slope";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x3edbba2aa3e00dce;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.92, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "b_lc", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "b_lc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.7 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "b_lc", value: answer.get(), bound: 0.7, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 0.70 no space production programme has learned that fast" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "b_lc", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "unity is no learning at all, the pessimistic limit" });
    }
    Ok(answer)
}
