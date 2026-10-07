// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much power does the spacecraft appear to radiate towards the ground station?
///
/// `EIRP = 10*log10(P_t) + G_t - L_line`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_eirp";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xeb1a369ea6533ec2;

pub fn evaluate(p: Power, g: Ratio, l: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::com_eirp. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::com_eirp::evaluate(p.get(), g.get(), l.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "EIRP")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "EIRP", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -20.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "EIRP", value: answer.get(), bound: -20.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below -20 dBW no link in this design closes" });
    }
    if answer.get() > 90.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "EIRP", value: answer.get(), bound: 90.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 90 dBW the transmitter is not the one described" });
    }
    Ok(answer)
}
