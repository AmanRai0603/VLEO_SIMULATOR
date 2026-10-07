// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Does thrust exceed drag — does the design close at all?
///
/// `M = (achieved - required)/required, in the sense the requirement is stated`
///
/// Source: `romano2021`
///
/// The one closure the whole programme exists to reach. Everything else is a
/// detail until this is at or above zero margin.
pub const NODE_ID: &str = "kpi_thrust_margin";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x131a219ad8cbf604;

pub fn evaluate(req: Ratio, ach: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::kpi_thrust_margin. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::kpi_thrust_margin::evaluate(req.get(), ach.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "M")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "M", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -100.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M", value: answer.get(), bound: -100.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a margin below -10000% means the inputs are not the ones the requirement was written against" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "a margin above 100000% means the requirement is not the one that binds" });
    }
    Ok(answer)
}
