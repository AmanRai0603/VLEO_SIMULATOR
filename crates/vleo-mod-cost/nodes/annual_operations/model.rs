// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does running the constellation cost each year?
///
/// `C_ops = 4.5`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "cost_annual_operations";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x03acfa9df83196ec;

pub fn evaluate() -> Result<Money, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Money = match Money::from_unit(4.5, Unit::MillionUsDollar) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "C_ops", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Money = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "C_ops", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_ops", value: answer.get(), bound: 100000.0, edge: Edge::Lower, unit: Money::UNIT, reason: "below 100 thousand a year no ground segment and team exist" });
    }
    if answer.get() > 200000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_ops", value: answer.get(), bound: 200000000.0, edge: Edge::Upper, unit: Money::UNIT, reason: "above 200 million a year the programme is not the one costed here" });
    }
    Ok(answer)
}
