// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Are emitters located accurately enough?
///
/// `M = (achieved - required)/required, in the sense the requirement is stated`
///
/// Source: `orbitt_case_c2`
pub const NODE_ID: &str = "kpi_geolocation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe93bafc60d9d339f;

pub fn evaluate(req: Length, ach: Length) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::kpi_geolocation. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::kpi_geolocation::evaluate(req.get(), ach.get()) {
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
