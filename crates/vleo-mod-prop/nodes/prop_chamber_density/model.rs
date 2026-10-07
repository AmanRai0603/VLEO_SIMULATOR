// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How dense is the gas the thruster is asked to ionise?
///
/// `n_c = CR*n`
///
/// Source: `romano2021`
///
/// This is the number that decides whether a plasma can be struck at all, and
/// it is why compression matters even though it does not appear in the
/// capture efficiency.
pub const NODE_ID: &str = "prop_chamber_density";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xf031d7e4b79095d5;

pub fn evaluate(n: NumberDensity, cr: Ratio) -> Result<NumberDensity, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::prop_chamber_density. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: NumberDensity = match methods::prop_chamber_density::evaluate(n.get(), cr.get()) {
        Ok(v) => NumberDensity::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "n_c")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: NumberDensity = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "n_c", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1000000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "n_c", value: answer.get(), bound: 1000000000000.0, edge: Edge::Lower, unit: NumberDensity::UNIT, reason: "below 1e12 per cubic metre no inductively coupled source will strike a discharge" });
    }
    if answer.get() > 1e22 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "n_c", value: answer.get(), bound: 1e22, edge: Edge::Upper, unit: NumberDensity::UNIT, reason: "above 1e22 the gas is at atmospheric pressure, which no intake in this band can produce" });
    }
    Ok(answer)
}
