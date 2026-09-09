// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much of the Earth's surface is in view at the working elevation?
///
/// `lambda = acos((Re/r)*cos(eps)) - eps`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "orbit_earth_central_angle";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb2ac9702170c4196;

pub fn evaluate(r: Length, eps: Angle) -> Result<Angle, Fault> {
    // ---- HOLE 1 : apply the spherical Earth access geometry -> Angle
    let l: Angle = orbit::earth_central_angle(r, eps);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Angle = l;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "lambda", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "lambda", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Angle::UNIT, reason: "a negative half-angle means the target is below the horizon" });
    }
    if answer.get() > 1.5009831567151235 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "lambda", value: answer.get(), bound: 1.5009831567151235, edge: Edge::Upper, unit: Angle::UNIT, reason: "beyond 85 degrees the whole visible hemisphere is in view, which does not occur in this band" });
    }
    Ok(answer)
}
