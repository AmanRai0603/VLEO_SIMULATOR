// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long does data wait on board before it can come down?
///
/// `t = 86400/(2*N_pass)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "mis_time_to_downlink";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xdb9bf4704a146539;

pub fn evaluate(n: Ratio) -> Result<Time, Fault> {
    // ---- HOLE 1 : take half the mean interval between passes -> Time
    let t: Time = mission::mean_time_to_downlink(n.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "t_dl", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_dl", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Time::UNIT, reason: "a wait cannot be negative" });
    }
    if answer.get() > 86400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_dl", value: answer.get(), bound: 86400.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above a day the mission is not operable" });
    }
    Ok(answer)
}
