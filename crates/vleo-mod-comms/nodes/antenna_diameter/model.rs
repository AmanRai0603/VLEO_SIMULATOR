// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How large is the spacecraft downlink antenna?
///
/// `D_a = 0.3`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "com_antenna_diameter";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xa302b2344c597abd;

pub fn evaluate() -> Result<Length, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Length = match Length::from_unit(0.3, Unit::Metre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "D_a", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "D_a", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.02 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D_a", value: answer.get(), bound: 0.02, edge: Edge::Lower, unit: Length::UNIT, reason: "below 2 cm the aperture gain relation is not valid at this frequency" });
    }
    if answer.get() > 2.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D_a", value: answer.get(), bound: 2.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 2 m the antenna is a drag surface larger than the vehicle" });
    }
    Ok(answer)
}
