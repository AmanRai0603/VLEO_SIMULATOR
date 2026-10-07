// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much does the spacecraft antenna concentrate the transmitted power?
///
/// `G = eta*(pi*D/lambda)^2`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_antenna_gain";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x3f2a126ab96a304e;

pub fn evaluate(d: Length, f: Frequency, e: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::com_antenna_gain. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::com_antenna_gain::evaluate(d.get(), f.get(), e.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "G_t")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "G_t", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "G_t", value: answer.get(), bound: -10.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "an aperture antenna does not have less than -10 dBi of gain" });
    }
    if answer.get() > 80.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "G_t", value: answer.get(), bound: 80.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 80 dBi is a deep-space aperture, not a spacecraft antenna" });
    }
    Ok(answer)
}
