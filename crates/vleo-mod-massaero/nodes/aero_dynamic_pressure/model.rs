// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What pressure does the flow exert per unit of drag coefficient and area?
///
/// `q = 0.5*rho*V^2`
///
/// Source: `vallado2013`
pub const NODE_ID: &str = "aero_dynamic_pressure";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x0a23af54ba5a3f8b;

pub fn evaluate(rho: MassDensity, v: Velocity) -> Result<Pressure, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::aero_dynamic_pressure. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Pressure = match methods::aero_dynamic_pressure::evaluate(rho.get(), v.get()) {
        Ok(v) => Pressure::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "q")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Pressure = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "q", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "q", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Pressure::UNIT, reason: "dynamic pressure cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "q", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Pressure::UNIT, reason: "above 1 Pa the vehicle is re-entering, not orbiting" });
    }
    Ok(answer)
}
