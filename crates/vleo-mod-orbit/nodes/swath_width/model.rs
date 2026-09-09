// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How wide a strip of ground does one pass cover?
///
/// `W = 2*lambda*Re`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "orbit_swath_width";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x3d5237ca30a079ef;

pub fn evaluate(lam: Angle) -> Result<Length, Fault> {
    // ---- HOLE 1 : convert the Earth-central half-angle into a great-circle width -> Length
    let w: Length = orbit::swath_width(lam);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = w;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "W", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "W", value: answer.get(), bound: 1000.0, edge: Edge::Lower, unit: Length::UNIT, reason: "a swath below one kilometre would not be a swath" });
    }
    if answer.get() > 40000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "W", value: answer.get(), bound: 40000000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "a swath cannot exceed the Earth's circumference" });
    }
    Ok(answer)
}
