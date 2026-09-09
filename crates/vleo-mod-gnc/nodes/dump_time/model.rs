// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long may the wheels accumulate momentum before it is dumped?
///
/// `t_dump = 45`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_dump_time";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xeeb200bfbbefb3ce;

pub fn evaluate() -> Result<Time, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Time = match Time::from_unit(45.0, Unit::Minute) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "t_dump", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "t_dump", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_dump", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Time::UNIT, reason: "below a minute the magnetorquers are being commanded faster than the field geometry changes" });
    }
    if answer.get() > 86400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_dump", value: answer.get(), bound: 86400.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above a day the wheels saturate long before the dump" });
    }
    Ok(answer)
}
