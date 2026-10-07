// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How many photoelectrons does one ground sample produce?
///
/// `N_e = L*Omega*A_px*B*tau*QE*t/(h*c/lambda)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pay_signal_electrons";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb446ee6f7016daa4;

pub fn evaluate(l: Ratio, d: Length, f: Length, p: Length, tau: Ratio, qe: Ratio, t: Time, lam: Length) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::pay_signal_electrons. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::pay_signal_electrons::evaluate(l.get(), d.get(), f.get(), p.get(), tau.get(), qe.get(), t.get(), lam.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "N_e")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "N_e", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "N_e", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "an electron count cannot be negative" });
    }
    if answer.get() > 1000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "N_e", value: answer.get(), bound: 1000000000.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above a billion electrons the well is saturated many times over" });
    }
    Ok(answer)
}
