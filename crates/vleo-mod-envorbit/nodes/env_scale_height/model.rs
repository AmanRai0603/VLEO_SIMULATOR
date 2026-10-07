// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Over what altitude change does the density fall by a factor of e?
///
/// `H = R*T/(M*g)`
///
/// Source: `vallado2013`
pub const NODE_ID: &str = "env_scale_height";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb1ee379c59b52b32;

pub fn evaluate(h: Length, t_inf: Temperature) -> Result<Length, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::env_scale_height. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Length = match methods::env_scale_height::evaluate(h.get(), t_inf.get()) {
        Ok(v) => Length::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "H")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Length = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "H", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 5000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "H", value: answer.get(), bound: 5000.0, edge: Edge::Lower, unit: Length::UNIT, reason: "a scale height below 5 km does not occur above the model base" });
    }
    if answer.get() > 200000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "H", value: answer.get(), bound: 200000.0, edge: Edge::Upper, unit: Length::UNIT, reason: "above 200 km the atmosphere is effectively isothermal and the concept stops being useful" });
    }
    Ok(answer)
}
