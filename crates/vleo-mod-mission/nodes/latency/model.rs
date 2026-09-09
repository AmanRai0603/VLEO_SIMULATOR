// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long from collection to the product reaching the customer?
///
/// `t = t_wait + t_downlink + t_process + t_deliver`
///
/// Source: `orbitt_case_c1`
///
/// The wait term dominates, and it is the one a constellation design can
/// actually move.
pub const NODE_ID: &str = "mis_latency";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb28f9549923d391d;

pub fn evaluate(tw: Time, td: Time, tp: Time, tdl: Time) -> Result<Time, Fault> {
    // ---- HOLE 1 : sum the wait, the downlink, the processing and the delivery -> Time
    let t: Time = mission::end_to_end_latency(tw, td, tp, tdl);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = t;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "t_lat", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_lat", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Time::UNIT, reason: "a latency cannot be negative" });
    }
    if answer.get() > 172800.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_lat", value: answer.get(), bound: 172800.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above two days the product is not the near-real-time one being sold" });
    }
    Ok(answer)
}
