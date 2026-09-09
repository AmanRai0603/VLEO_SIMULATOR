// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does building the whole constellation cost?
///
/// `C = sum over n of T1*n^log2(b)`
///
/// Source: `nasa_cer`
pub const NODE_ID: &str = "cost_production";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe767cda4aeb4290c;

pub fn evaluate(cb: Money, cp: Money, n: Ratio, b: Ratio) -> Result<Money, Fault> {
    // ---- HOLE 1 : sum the learning curve over the production run of buses and payloads together -> Money
    let c: Money = cost::production_run_cost(Money::new(cb.get() + cp.get()), n.get() as u32, b.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Money = c;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "C_prod", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_prod", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Money::UNIT, reason: "a cost cannot be negative" });
    }
    if answer.get() > 1000000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_prod", value: answer.get(), bound: 1000000000000.0, edge: Edge::Upper, unit: Money::UNIT, reason: "above a trillion dollars the programme is not the one being designed" });
    }
    Ok(answer)
}
