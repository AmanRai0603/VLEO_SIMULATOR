// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far away is a target or ground station at the edge of the access circle?
///
/// `d = sqrt(Re^2 + r^2 - 2*Re*r*cos(lambda))`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "orbit_slant_range";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xbb906494a88be9a5;

pub fn evaluate(r: Length, eps: Angle) -> Result<Length, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::orbit_slant_range. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Length = match methods::orbit_slant_range::evaluate(r.get(), eps.get()) {
        Ok(v) => Length::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "d")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "d", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "d", value: answer.get(), bound: 100000.0, edge: Edge::Lower, unit: Length::UNIT, reason: "the slant range is at least the altitude" });
    }
    if answer.get() > 4000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "d", value: answer.get(), bound: 4000000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "beyond 4000 km the geometry is outside this tool's band" });
    }
    Ok(answer)
}
