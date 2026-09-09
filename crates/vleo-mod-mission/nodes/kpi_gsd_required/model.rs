// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the customer require for ground sample distance?
///
/// `R_req = 1`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "kpi_gsd_required";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb97e8ba172eeb9dd;

pub fn evaluate() -> Result<Length, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Length = match Length::from_unit(1.0, Unit::Metre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "R_req", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "R_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.05 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_req", value: answer.get(), bound: 0.05, edge: Edge::Lower, unit: Length::UNIT, reason: "below 5 cm no optical payload in this mass class achieves it" });
    }
    if answer.get() > 100.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_req", value: answer.get(), bound: 100.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 100 m the product is not the one being sold" });
    }
    Ok(answer)
}
