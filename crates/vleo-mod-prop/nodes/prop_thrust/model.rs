// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much thrust does the thruster produce from the air it is given?
///
/// `T = mdot_i*v_e*alpha_div*alpha_pp`
///
/// Source: `romano2021`
pub const NODE_ID: &str = "prop_thrust";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe53eebcf85715c13;

pub fn evaluate(mi: MassFlow, ve: Velocity, ad: Ratio, ap: Ratio) -> Result<Force, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::prop_thrust. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Force = match methods::prop_thrust::evaluate(mi.get(), ve.get(), ad.get(), ap.get()) {
        Ok(v) => Force::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "T")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Force = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Force::UNIT, reason: "thrust cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Force::UNIT, reason: "above 1 N no air-breathing thruster in this band produces thrust, at any intake size" });
    }
    Ok(answer)
}
