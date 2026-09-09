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
pub const SHEET_HASH: u64 = 0xb02fbde73f15f327;

pub fn evaluate(a: Ratio, n: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : evaluate the binomial probability that at least 80% of the constellation is operational -> Ratio
    let av: Ratio = mission::k_of_n_availability(a, n.get() as u32, ((n.get() * 0.8).ceil()) as u32);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = av;
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
