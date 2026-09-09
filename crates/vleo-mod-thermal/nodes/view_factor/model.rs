// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much of the sky, from a nadir-facing surface, is filled by the Earth?
///
/// `F = (Re/r)^2`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "thm_view_factor";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x524902c1f5f56891;

pub fn evaluate(r: Length) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : apply the flat-plate to sphere view factor at the orbital radius -> Ratio
    let f: Ratio = thermal::earth_view_factor(r);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = f;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F_E", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F_E", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a view factor cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F_E", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "a view factor cannot exceed one" });
    }
    Ok(answer)
}
