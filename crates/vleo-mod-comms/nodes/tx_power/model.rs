// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much radio-frequency power does the transmitter deliver?
///
/// `P_t = 8`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "com_tx_power";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb399956af42bbde5;

pub fn evaluate() -> Result<Power, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Power = match Power::from_unit(8.0, Unit::Watt) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "P_t", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "P_t", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.1 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_t", value: answer.get(), bound: 0.1, edge: Edge::Lower, unit: Power::UNIT, reason: "below 0.1 W no link closes from this altitude" });
    }
    if answer.get() > 200.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_t", value: answer.get(), bound: 200.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 200 W the transmitter is the largest load on the bus and the thermal design does not close" });
    }
    Ok(answer)
}
