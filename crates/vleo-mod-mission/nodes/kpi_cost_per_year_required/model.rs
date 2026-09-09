// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the customer require for cost per operational year?
///
/// `R_req = 45`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "kpi_cost_per_year_required";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x277e202208eaf87c;

pub fn evaluate() -> Result<Money, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Money = match Money::from_unit(45.0, Unit::MillionUsDollar) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "R_req", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Money = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "R_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_req", value: answer.get(), bound: 100000.0, edge: Edge::Lower, unit: Money::UNIT, reason: "below 100 thousand a year no programme of this size runs" });
    }
    if answer.get() > 10000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_req", value: answer.get(), bound: 10000000000.0, edge: Edge::Upper, unit: Money::UNIT, reason: "above 10 billion a year the programme is not the one being designed" });
    }
    Ok(answer)
}
