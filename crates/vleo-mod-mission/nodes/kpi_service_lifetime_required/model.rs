// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the customer require for service lifetime?
///
/// `R_req = 5`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "kpi_service_lifetime_required";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x7be477dba099f4bf;

pub fn evaluate() -> Result<Time, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Time = match Time::from_unit(5.0, Unit::Year) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "R_req", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "R_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 3155760.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_req", value: answer.get(), bound: 3155760.0, edge: Edge::Lower, unit: Time::UNIT, reason: "below five weeks the satellite is not a mission" });
    }
    if answer.get() > 788940000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_req", value: answer.get(), bound: 788940000.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above 25 years the debris mitigation standard is violated" });
    }
    Ok(answer)
}
