// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What fraction of the time is the service available?
///
/// `A = P(at least ceil(0.8*N) of N satellites up)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "mis_availability";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xdbcde12e57985f9a;

pub fn evaluate(a: Ratio, n: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::mis_availability. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::mis_availability::evaluate(a.get(), n.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "A_svc")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "A_svc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_svc", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "an availability cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_svc", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "an availability cannot exceed one" });
    }
    Ok(answer)
}
