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
pub const SHEET_HASH: u64 = 0x121be8c520a01b03;

pub fn evaluate(l: Ratio, d: Length, f: Length, p: Length, tau: Ratio, qe: Ratio, t: Time, lam: Length) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : collect photons through the aperture solid angle over the dwell time and convert them at the detector's quantum efficiency, over a 100 nm band -> Ratio
    let n: Ratio = Ratio::new(payload::signal_electrons(l.get() * 1.0e6, d, f, p, tau, qe, t, lam, Length::new(1.0e-7)));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = n;
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
