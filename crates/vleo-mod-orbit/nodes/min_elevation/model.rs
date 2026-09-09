// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Below what elevation is a target or a ground station no longer usable?
///
/// `eps = 10`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "orbit_min_elevation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x3ce7ed985d677476;

pub fn evaluate() -> Result<Angle, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Angle = match Angle::from_unit(10.0, Unit::Degree) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "eps", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Angle = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "eps", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "eps", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Angle::UNIT, reason: "at the horizon the slant range is longest, the atmosphere is thickest and no useful link or image is obtained; zero is the limit, not a working value" });
    }
    if answer.get() > 1.5707963267948966 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "eps", value: answer.get(), bound: 1.5707963267948966, edge: Edge::Upper, unit: Angle::UNIT, reason: "90 degrees is the zenith, at which the access circle has no area" });
    }
    Ok(answer)
}
