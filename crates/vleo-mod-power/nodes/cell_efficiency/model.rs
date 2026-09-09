// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How efficiently does one cell convert sunlight?
///
/// `eta_cell = 0.3`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pwr_cell_efficiency";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xbe6c0be8ded281f3;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.3, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "eta_cell", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "eta_cell", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.1 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "eta_cell", value: answer.get(), bound: 0.1, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 10% no cell technology considered for this programme performs that badly" });
    }
    if answer.get() > 0.4 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "eta_cell", value: answer.get(), bound: 0.4, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 40% is beyond any qualified space cell available at the time of this baseline" });
    }
    Ok(answer)
}
