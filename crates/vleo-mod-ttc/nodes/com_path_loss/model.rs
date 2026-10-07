// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much of the signal is lost simply by spreading out over the range?
///
/// `L = (4*pi*d/lambda)^2`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_path_loss";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x8452f968513d8a2c;

pub fn evaluate(d: Length, f: Frequency) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::com_path_loss. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::com_path_loss::evaluate(d.get(), f.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "L_fs")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "L_fs", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 100.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L_fs", value: answer.get(), bound: 100.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 100 dB the range is shorter than any orbit in this band" });
    }
    if answer.get() > 250.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L_fs", value: answer.get(), bound: 250.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 250 dB the range is not one this tool covers" });
    }
    Ok(answer)
}
