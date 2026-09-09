// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does it cost to put one satellite in orbit?
///
/// `C_lch = 1.8`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "cost_launch_per_satellite";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x0b99ec0b912e4de9;

pub fn evaluate() -> Result<Money, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Money = match Money::from_unit(1.8, Unit::MillionUsDollar) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "C_lch", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Money = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "C_lch", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 50000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_lch", value: answer.get(), bound: 50000.0, edge: Edge::Lower, unit: Money::UNIT, reason: "below 50 thousand dollars no launch service exists" });
    }
    if answer.get() > 100000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_lch", value: answer.get(), bound: 100000000.0, edge: Edge::Upper, unit: Money::UNIT, reason: "above 100 million per satellite the constellation is not the one costed here" });
    }
    Ok(answer)
}
