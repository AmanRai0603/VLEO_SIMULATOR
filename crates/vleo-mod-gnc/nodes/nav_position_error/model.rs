// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How well does the spacecraft know where it is?
///
/// `e = URE*GDOP`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "gnc_nav_position_error";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x354d798dbde16bf1;

pub fn evaluate(u: Length, g: Ratio) -> Result<Length, Fault> {
    // ---- HOLE 1 : multiply the user range error by the geometric dilution -> Length
    let e: Length = gnc::navigation_position_error(u, g.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = e;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "e_nav", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e_nav", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Length::UNIT, reason: "a position error cannot be negative" });
    }
    if answer.get() > 10000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "e_nav", value: answer.get(), bound: 10000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 10 km the navigation solution is not usable" });
    }
    Ok(answer)
}
