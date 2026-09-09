// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much ground does one satellite see at any instant?
///
/// `A = 2*pi*Re^2*(1 - cos(lambda))`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "mis_access_area";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb3349c53a804b26c;

pub fn evaluate(lam: Angle) -> Result<Area, Fault> {
    // ---- HOLE 1 : integrate the spherical cap subtended by the Earth-central half-angle -> Area
    let a: Area = mission::access_area(lam);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Area = a;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "A_acc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_acc", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Area::UNIT, reason: "an area cannot be negative" });
    }
    if answer.get() > 520000000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_acc", value: answer.get(), bound: 520000000000000.0, edge: Edge::Upper, unit: Area::UNIT, reason: "cannot exceed the surface area of the Earth" });
    }
    Ok(answer)
}
