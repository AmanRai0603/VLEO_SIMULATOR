// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Over what bandwidth does the geolocation payload observe?
///
/// `B_rf = 20`
///
/// Source: `orbitt_case_c2`
pub const NODE_ID: &str = "pay_rf_bandwidth";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x544b7f6c96bd7fa5;

pub fn evaluate() -> Result<Frequency, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Frequency = match Frequency::from_unit(20.0, Unit::Megahertz) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "B_rf", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Frequency = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "B_rf", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "B_rf", value: answer.get(), bound: 100000.0, edge: Edge::Lower, unit: Frequency::UNIT, reason: "below 100 kHz the timing precision is too coarse for any geolocation requirement here" });
    }
    if answer.get() > 500000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "B_rf", value: answer.get(), bound: 500000000.0, edge: Edge::Upper, unit: Frequency::UNIT, reason: "above 500 MHz the digitiser and the downlink neither fit nor close" });
    }
    Ok(answer)
}
