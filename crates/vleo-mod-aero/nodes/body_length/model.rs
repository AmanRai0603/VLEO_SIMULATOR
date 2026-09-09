// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long is the vehicle along the flight direction?
///
/// `L = 2`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "aero_body_length";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb56db44fb7e6a428;

pub fn evaluate() -> Result<Length, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Length = match Length::from_unit(2.0, Unit::Metre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "L", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "L", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.3 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L", value: answer.get(), bound: 0.3, edge: Edge::Lower, unit: Length::UNIT, reason: "below 0.3 m the vehicle is smaller than a 3U cubesat and the multipayload mission does not fit in it" });
    }
    if answer.get() > 10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L", value: answer.get(), bound: 10.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 10 m the launch envelope and the free-molecular panel model both stop applying" });
    }
    Ok(answer)
}
