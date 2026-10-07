// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far is the spacecraft from the centre of the Earth?
///
/// `r = R_earth + h`
///
/// Source: `wgs84`
pub const NODE_ID: &str = "orbit_radius";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x4f66737f39031c3b;

pub fn evaluate(h: Length) -> Result<Length, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::orbit_radius. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Length = match methods::orbit_radius::evaluate(h.get()) {
        Ok(v) => Length::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "r")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "r", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 6400000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "r", value: answer.get(), bound: 6400000.0, edge: Edge::Lower, unit: Length::UNIT, reason: "below the Earth's mean radius is not an orbit" });
    }
    if answer.get() > 7000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "r", value: answer.get(), bound: 7000000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 7000 km is outside the band this tool is scoped to" });
    }
    Ok(answer)
}
