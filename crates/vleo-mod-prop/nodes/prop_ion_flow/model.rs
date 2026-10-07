// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much of the collected mass is actually accelerable?
///
/// `mdot_i = eta_u*mdot`
///
/// Source: `romano2021`
pub const NODE_ID: &str = "prop_ion_flow";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x3558bc55488a25fe;

pub fn evaluate(md: MassFlow, eta_u: Ratio) -> Result<MassFlow, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::prop_ion_flow. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: MassFlow = match methods::prop_ion_flow::evaluate(md.get(), eta_u.get()) {
        Ok(v) => MassFlow::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "mdot_i")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: MassFlow = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "mdot_i", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "mdot_i", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: MassFlow::UNIT, reason: "an ion mass flow cannot be negative" });
    }
    if answer.get() > 9.999999999999999e-6 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "mdot_i", value: answer.get(), bound: 9.999999999999999e-6, edge: Edge::Upper, unit: MassFlow::UNIT, reason: "cannot exceed the collected flow it comes from" });
    }
    Ok(answer)
}
