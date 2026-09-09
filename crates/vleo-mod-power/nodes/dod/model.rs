// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How deeply is the battery discharged each eclipse?
///
/// `DoD = 0.2`
///
/// Source: `larson_wertz`
///
/// A lifetime decision rather than an energy one: 92-minute orbits over five
/// years is roughly 29000 cycles.
pub const NODE_ID: &str = "pwr_dod";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb6c5627de3d74caf;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.2, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "DoD", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "DoD", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.05 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "DoD", value: answer.get(), bound: 0.05, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 5% the battery is absurdly oversized" });
    }
    if answer.get() > 0.8 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "DoD", value: answer.get(), bound: 0.8, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 80% no lithium-ion cell survives 29000 cycles, which is what a five-year VLEO mission is" });
    }
    Ok(answer)
}
