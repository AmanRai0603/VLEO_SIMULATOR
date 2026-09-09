// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much heat does the spacecraft have to get rid of?
///
/// `Q = P_dem - P_rf - P_beam`
///
/// Source: `larson_wertz`
///
/// # Assumptions
///
/// * All bus power that is not beam kinetic energy or radiated radio power becomes heat — fails when true to within the accuracy of this budget; it ignores the small fraction stored in the battery over a cycle, which averages to zero over an orbit
pub const NODE_ID: &str = "thm_dissipation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x36bc4652762c72ab;

pub fn evaluate(dem: Power, pj: Power, tx: Power) -> Result<Power, Fault> {
    // ---- HOLE 1 : everything drawn from the bus becomes heat except the beam kinetic power and the radiated radio-frequency power -> Power
    let q: Power = Power::new(pmath::max(0.0, dem.get() - pj.get() - tx.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = q;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Q", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Q", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "heat to reject cannot be negative" });
    }
    if answer.get() > 50000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Q", value: answer.get(), bound: 50000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 50 kW nothing in this mass class dissipates that much" });
    }
    Ok(answer)
}
