// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much is lost between the transmitter and the antenna, in decibels?
///
/// `L_line = 1.2`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_line_loss";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x565abed82cbc1bdc;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(1.2, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "L_line", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "L_line", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L_line", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a loss cannot be negative in decibels here" });
    }
    if answer.get() > 6.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L_line", value: answer.get(), bound: 6.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 6 dB most of the transmitter output is heating the harness" });
    }
    Ok(answer)
}
