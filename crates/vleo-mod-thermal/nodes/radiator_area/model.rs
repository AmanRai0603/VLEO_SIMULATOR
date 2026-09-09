// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much area is available to reject heat?
///
/// `A_rad = 2.5`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "thm_radiator_area";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x3fa141d26b63da0f;

pub fn evaluate() -> Result<Area, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Area = match Area::from_unit(2.5, Unit::SquareMetre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "A_rad", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Area = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "A_rad", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.1 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_rad", value: answer.get(), bound: 0.1, edge: Edge::Lower, unit: Area::UNIT, reason: "below 0.1 m2 no useful heat is rejected" });
    }
    if answer.get() > 30.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_rad", value: answer.get(), bound: 30.0, edge: Edge::Upper, unit: Area::UNIT, reason: "above 30 m2 the radiator is the spacecraft" });
    }
    Ok(answer)
}
