// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long does the product take to reach the customer once processed?
///
/// `t_del = 3`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "mis_delivery_time";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xd832403405804f07;

pub fn evaluate() -> Result<Time, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Time = match Time::from_unit(3.0, Unit::Minute) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "t_del", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "t_del", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 6.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_del", value: answer.get(), bound: 6.0, edge: Edge::Lower, unit: Time::UNIT, reason: "below six seconds no delivery path in this design runs" });
    }
    if answer.get() > 86400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_del", value: answer.get(), bound: 86400.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above a day the product is not the near-real-time one being sold" });
    }
    Ok(answer)
}
