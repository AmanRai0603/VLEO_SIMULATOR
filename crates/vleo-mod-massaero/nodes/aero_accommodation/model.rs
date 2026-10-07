// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How completely does an incoming molecule thermalise with the surface before leaving?
///
/// `alpha = K*n_O*T_inf/(1 + K*n_O*T_inf),  K = 7.5e-17`
///
/// Source: `moe2005`
///
/// A design that assumes a fixed accommodation has a margin that moves when
/// the Sun does.
///
/// # Assumptions
///
/// * The surface is covered in adsorbed atomic oxygen and behaves as that coverage dictates — fails when a freshly cleaned or a fluorinated surface adsorbs far less, and its drag differs by tens of per cent from this
pub const NODE_ID: &str = "aero_accommodation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xccfd7b2bd3ee976b;

pub fn evaluate(n_o: NumberDensity, t_inf: Temperature) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::aero_accommodation. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::aero_accommodation::evaluate(n_o.get(), t_inf.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "alpha")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "alpha", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "alpha", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "accommodation is a fraction and cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "alpha", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "full accommodation is one; above it the reflected molecules would carry away energy the surface does not have" });
    }
    Ok(answer)
}
