// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

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
pub const SHEET_HASH: u64 = 0x4e50bd785bf32275;

pub fn evaluate(n: Ratio, nr: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : combine shot noise, a nominal 5-electron dark contribution and read noise in quadrature -> Ratio
    let s: Ratio = Ratio::new(payload::optical_snr(n.get(), 5.0, nr.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = s;
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
