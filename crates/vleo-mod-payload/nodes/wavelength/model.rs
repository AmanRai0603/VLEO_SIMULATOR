// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// At what wavelength does the imager work?
///
/// `lam_o = 550`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "pay_wavelength";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc213f32bfbe47f72;

pub fn evaluate() -> Result<Length, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Length = match Length::from_unit(550.0, Unit::Nanometre) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "lam_o", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "lam_o", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 3.5000000000000004e-7 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "lam_o", value: answer.get(), bound: 3.5000000000000004e-7, edge: Edge::Lower, unit: Length::UNIT, reason: "below 350 nm the atmosphere absorbs most of the signal" });
    }
    if answer.get() > 2.5e-6 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "lam_o", value: answer.get(), bound: 2.5e-6, edge: Edge::Upper, unit: Length::UNIT, reason: "above 2500 nm the detector is a cooled infrared array, which is not the payload described here" });
    }
    Ok(answer)
}
