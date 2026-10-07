// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How wrong might the density be, and therefore how wrong is every margin built on it?
///
/// `sigma_rho = 0.15 + 0.03*Kp`
///
/// Source: `doornbos2011`
///
/// A node that consumes density and does not carry this forward is a node
/// whose margin is fictional.
pub const NODE_ID: &str = "env_density_uncertainty";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x27a8cbf447f7af93;

pub fn evaluate(kp: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::env_density_uncertainty. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::env_density_uncertainty::evaluate(kp.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "sigma_rho")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "sigma_rho", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.05 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sigma_rho", value: answer.get(), bound: 0.05, edge: Edge::Lower, unit: Ratio::UNIT, reason: "no published comparison of an empirical thermosphere model against measured drag does better than 5%" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sigma_rho", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "an uncertainty above 100% of the value is not an uncertainty, it is an absence of a model" });
    }
    Ok(answer)
}
