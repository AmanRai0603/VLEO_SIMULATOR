// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much output does the array lose per kelvin above its reference temperature?
///
/// `k_T = -0.0025`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pwr_cell_temperature_coefficient";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x936f8965ecd5ca42;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(-0.0025, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "k_T", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "k_T", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -0.01 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "k_T", value: answer.get(), bound: -0.01, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below -1% per kelvin no cell technology degrades that fast" });
    }
    if answer.get() > 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "k_T", value: answer.get(), bound: 0.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "a positive coefficient would mean a cell that improves when heated, which does not happen" });
    }
    Ok(answer)
}
