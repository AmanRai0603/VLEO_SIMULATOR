// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How clean is one sample?
///
/// `SNR = N_e/sqrt(N_e + N_dark + n_read^2)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pay_snr";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xbcbf4c0bde35a569;

pub fn evaluate(n: Ratio, nr: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::pay_snr. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::pay_snr::evaluate(n.get(), nr.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "SNR_o")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "SNR_o", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "SNR_o", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a signal to noise ratio cannot be negative" });
    }
    if answer.get() > 10000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "SNR_o", value: answer.get(), bound: 10000.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 10000 the detector is far outside its linear range" });
    }
    Ok(answer)
}
