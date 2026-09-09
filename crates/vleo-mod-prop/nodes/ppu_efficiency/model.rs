// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How efficiently does the power processing unit convert bus power into thruster power?
///
/// `eta_ppu = 0.9`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "prop_ppu_efficiency";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x870cd3387928b676;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.9, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "eta_ppu", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "eta_ppu", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "eta_ppu", value: answer.get(), bound: 0.5, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 50% the power processing unit dissipates more than the thruster uses and the thermal design does not close" });
    }
    if answer.get() > 0.98 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "eta_ppu", value: answer.get(), bound: 0.98, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 98% no flown power processing unit has been demonstrated" });
    }
    Ok(answer)
}
