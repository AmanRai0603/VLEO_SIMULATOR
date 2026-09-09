// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How hard is the atmosphere pulling the satellite backwards?
///
/// `D = 0.5*rho*V^2*Cd*A`
///
/// Source: `vallado2013`
pub const NODE_ID: &str = "aero_drag_force";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x7eec597d6fa80708;

pub fn evaluate(q: Pressure, cd: Ratio, a: Area) -> Result<Force, Fault> {
    // ---- HOLE 1 : multiply dynamic pressure by the drag coefficient and the frontal area -> Force
    let d: Force = Force::new(q.get() * cd.get() * a.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Force = d;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "D", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Force::UNIT, reason: "drag cannot be negative" });
    }
    if answer.get() > 10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D", value: answer.get(), bound: 10.0, edge: Edge::Upper, unit: Force::UNIT, reason: "above 10 N the vehicle is decelerating at a rate no electric propulsion system can answer" });
    }
    Ok(answer)
}
