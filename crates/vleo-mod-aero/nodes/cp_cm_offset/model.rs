// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far behind the centre of mass does the aerodynamic force act?
///
/// `x_cp = 0.15`
///
/// Source: `orbitt_case_c1`
///
/// Positive means aft, which is the stable configuration. Arranging that is
/// most of what passive aerostability in VLEO means.
pub const NODE_ID: &str = "aero_cp_cm_offset";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x8294eafc26c0f3c8;

pub fn evaluate() -> Result<Length, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Length = match Length::from_unit(0.15, Unit::Metre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "x_cp", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "x_cp", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "x_cp", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Length::UNIT, reason: "a zero offset is neutrally stable, which is the limit and not a design" });
    }
    if answer.get() > 2.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "x_cp", value: answer.get(), bound: 2.0, edge: Edge::Upper, unit: Length::UNIT, reason: "the offset cannot exceed the body length" });
    }
    Ok(answer)
}
