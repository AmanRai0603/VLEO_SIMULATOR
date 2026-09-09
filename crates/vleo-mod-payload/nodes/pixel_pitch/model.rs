// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How large is one detector pixel?
///
/// `p_x = 5.5`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "pay_pixel_pitch";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x5db5836fc922de55;

pub fn evaluate() -> Result<Length, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Length = match Length::from_unit(5.5, Unit::Micrometre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "p_x", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "p_x", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1e-6 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "p_x", value: answer.get(), bound: 1e-6, edge: Edge::Lower, unit: Length::UNIT, reason: "below one micrometre no space-qualified detector has pixels that small" });
    }
    if answer.get() > 2.9999999999999997e-5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "p_x", value: answer.get(), bound: 2.9999999999999997e-5, edge: Edge::Upper, unit: Length::UNIT, reason: "above 30 micrometres the detector-limited sample dominates and the aperture is wasted" });
    }
    Ok(answer)
}
