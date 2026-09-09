// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What deceleration does the drag produce on this mass?
///
/// `a_D = D/m`
///
/// Source: `vallado2013`
pub const NODE_ID: &str = "aero_drag_acceleration";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb93db6e6e32b7af8;

pub fn evaluate(d: Force, m: Mass) -> Result<Acceleration, Fault> {
    // ---- HOLE 1 : divide the drag force by the wet mass -> Acceleration
    let a: Acceleration = aero::drag_acceleration(d, m);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Acceleration = a;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "a_D", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "a_D", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Acceleration::UNIT, reason: "drag deceleration cannot be negative" });
    }
    if answer.get() > 0.1 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "a_D", value: answer.get(), bound: 0.1, edge: Edge::Upper, unit: Acceleration::UNIT, reason: "above 0.1 m/s2 the orbit decays within hours" });
    }
    Ok(answer)
}
