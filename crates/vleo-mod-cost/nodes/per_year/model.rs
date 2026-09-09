// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does a year of service cost?
///
/// `C = C_total/years`
///
/// Source: `nasa_cer`
pub const NODE_ID: &str = "cost_per_year";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xd82656d28ceb7657;

pub fn evaluate(c: Money, y: Time) -> Result<Money, Fault> {
    // ---- HOLE 1 : divide the programme cost by the mission duration in years -> Money
    let cy: Money = cost::cost_per_service_unit(c, y.get() / 31_557_600.0);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Money = cy;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "C_yr", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_yr", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Money::UNIT, reason: "a cost cannot be negative" });
    }
    if answer.get() > 1000000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_yr", value: answer.get(), bound: 1000000000000.0, edge: Edge::Upper, unit: Money::UNIT, reason: "above a trillion dollars a year the programme is not the one being designed" });
    }
    Ok(answer)
}
