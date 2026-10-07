// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much noise does the receiving system contribute?
///
/// `T_s = T_a/L + T0*(L-1)/L + T0*(F-1)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_system_noise_temperature";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x094809e74ff69d86;

pub fn evaluate(ta: Temperature, ll: Ratio) -> Result<Temperature, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::com_system_noise_temperature. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Temperature = match methods::com_system_noise_temperature::evaluate(ta.get(), ll.get()) {
        Ok(v) => Temperature::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "T_s")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Temperature = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_s", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 10.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_s", value: answer.get(), bound: 10.0, edge: Edge::Lower, unit: Temperature::UNIT, reason: "below 10 K requires cryogenic cooling not present in this ground segment" });
    }
    if answer.get() > 3000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_s", value: answer.get(), bound: 3000.0, edge: Edge::Upper, unit: Temperature::UNIT, reason: "above 3000 K the receiving system is not usable" });
    }
    Ok(answer)
}
