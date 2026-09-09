// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How large is the deployed solar array?
///
/// `A_arr = 6`
///
/// Source: `orbitt_case_c1`
///
/// A design variable rather than a computed one, deliberately: it is coupled
/// to drag through the appendage area, so a solver that sized it from the
/// power demand alone would hide the coupling the tree exists to show.
pub const NODE_ID: &str = "pwr_array_area";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe5afe6c209de4ad2;

pub fn evaluate() -> Result<Area, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Area = match Area::from_unit(6.0, Unit::SquareMetre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "A_arr", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Area = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "A_arr", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_arr", value: answer.get(), bound: 0.5, edge: Edge::Lower, unit: Area::UNIT, reason: "below 0.5 m2 no useful power is produced at any efficiency" });
    }
    if answer.get() > 40.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_arr", value: answer.get(), bound: 40.0, edge: Edge::Upper, unit: Area::UNIT, reason: "above 40 m2 the array's own drag exceeds the thrust the intake can produce at every altitude in the band" });
    }
    Ok(answer)
}
