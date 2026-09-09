// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much frontal area do the array, antennas and radiators add beyond the body?
///
/// `A_app = 0.05`
///
/// Source: `orbitt_case_c1`
///
/// Separated from the body area because it is the term a power or thermal
/// decision moves, and the tree has to show that coupling.
pub const NODE_ID: &str = "aero_appendage_area";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x097b27e81f620582;

pub fn evaluate() -> Result<Area, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Area = match Area::from_unit(0.05, Unit::SquareMetre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "A_app", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Area = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "A_app", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_app", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Area::UNIT, reason: "an area cannot be negative" });
    }
    if answer.get() > 5.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_app", value: answer.get(), bound: 5.0, edge: Edge::Upper, unit: Area::UNIT, reason: "above 5 m2 of appendage the design is an array with a satellite attached, and the drag closure is decided by the array alone" });
    }
    Ok(answer)
}
