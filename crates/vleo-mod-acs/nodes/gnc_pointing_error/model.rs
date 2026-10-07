// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How accurately does the payload point?
///
/// `sigma = sqrt(sum of squares of the independent contributors)`
///
/// Source: `larson_wertz`
///
/// Root-sum-squared, unlike the disturbance torques above. These terms are
/// genuinely independent, and summing them linearly would give a budget
/// nothing could ever meet.
pub const NODE_ID: &str = "gnc_pointing_error";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xf01bfee955a3dee5;

pub fn evaluate(sen: Angle, ali: Angle, ctl: Angle, thm: Angle) -> Result<Angle, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::gnc_pointing_error. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Angle = match methods::gnc_pointing_error::evaluate(sen.get(), ali.get(), ctl.get(), thm.get()) {
        Ok(v) => Angle::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "sig_pt")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Angle = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "sig_pt", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sig_pt", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Angle::UNIT, reason: "a pointing error cannot be negative" });
    }
    if answer.get() > 0.05235987755982988 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sig_pt", value: answer.get(), bound: 0.05235987755982988, edge: Edge::Upper, unit: Angle::UNIT, reason: "above three degrees no payload in this design produces a usable product" });
    }
    Ok(answer)
}
