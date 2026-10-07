// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the end-of-life disposal manoeuvre cost, as required by the debris mitigation standard?
///
/// `dv = |V_circ - V_transfer(r, a_t)|`
///
/// Source: `iso24113`
///
/// Carried as a design variable rather than discovered at the end, because
/// ISO 24113 makes it a requirement and not a courtesy.
pub const NODE_ID: &str = "orbit_deorbit_delta_v";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x4f950770ccebee9f;

pub fn evaluate(h: Length) -> Result<Velocity, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::orbit_deorbit_delta_v. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Velocity = match methods::orbit_deorbit_delta_v::evaluate(h.get()) {
        Ok(v) => Velocity::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "dv_dis")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Velocity = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "dv_dis", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dv_dis", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Velocity::UNIT, reason: "a disposal manoeuvre cannot cost negative delta-v" });
    }
    if answer.get() > 500.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dv_dis", value: answer.get(), bound: 500.0, edge: Edge::Upper, unit: Velocity::UNIT, reason: "above 500 m/s the disposal budget dominates the design and the altitude choice should be revisited" });
    }
    Ok(answer)
}
