// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How precisely can the payload time an intercepted signal?
///
/// `sigma = 1/(2*pi*B*sqrt(2*SNR*B*T))`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pay_toa_uncertainty";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x1b8751e76d2e38da;

pub fn evaluate(b: Frequency, s: Ratio, t: Time) -> Result<Time, Fault> {
    // ---- HOLE 1 : apply the Cramer-Rao lower bound on time-of-arrival estimation -> Time
    let sg: Time = payload::toa_timing_uncertainty(b, s.get(), t);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = sg;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "sig_tau", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1e-15 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sig_tau", value: answer.get(), bound: 1e-15, edge: Edge::Lower, unit: Time::UNIT, reason: "below a femtosecond no receiver in this design times that precisely" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sig_tau", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above one second the measurement carries no geolocation information" });
    }
    Ok(answer)
}
