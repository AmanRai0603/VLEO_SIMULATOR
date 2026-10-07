// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

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
pub const SHEET_HASH: u64 = 0x5acac2813aa9de6a;

pub fn evaluate(d: Force, m: Mass) -> Result<Acceleration, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::aero_drag_acceleration. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Acceleration = match methods::aero_drag_acceleration::evaluate(d.get(), m.get()) {
        Ok(v) => Acceleration::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "a_D")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Acceleration = method_answer;
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
