// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the first payload set cost to build?
///
/// `C = a*m_pay^b, escalated`
///
/// Source: `nasa_cer`
pub const NODE_ID: &str = "cost_payload_recurring";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x84f47dfb200c926d;

pub fn evaluate(m: Mass, i: Ratio) -> Result<Money, Fault> {
    // ---- HOLE 1 : evaluate the payload cost estimating relation on payload mass and escalate to 2026 -> Money
    let c: Money = cost::PAYLOAD_RECURRING_CER.evaluate_inflated(m.get(), 2026, i.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Money = c;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "C_pay", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_pay", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Money::UNIT, reason: "a cost cannot be negative" });
    }
    if answer.get() > 10000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_pay", value: answer.get(), bound: 10000000000.0, edge: Edge::Upper, unit: Money::UNIT, reason: "above 10 billion for one payload set the relation is being fed a mass it was not fitted on" });
    }
    Ok(answer)
}
