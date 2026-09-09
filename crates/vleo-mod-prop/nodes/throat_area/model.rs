// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How large is the opening from the collection chamber into the thruster?
///
/// `A_out = 0.01`
///
/// Source: `romano2021`
pub const NODE_ID: &str = "prop_throat_area";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xf743d1b68b0e6f8f;

pub fn evaluate() -> Result<Area, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Area = match Area::from_unit(0.01, Unit::SquareMetre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "A_out", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Area = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "A_out", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0001 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_out", value: answer.get(), bound: 0.0001, edge: Edge::Lower, unit: Area::UNIT, reason: "below 1 cm2 the throat chokes the flow to the thruster whatever the mouth collects" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_out", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Area::UNIT, reason: "the throat cannot be larger than the mouth in any useful intake" });
    }
    Ok(answer)
}
