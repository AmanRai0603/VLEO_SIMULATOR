// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How many usable ground station contacts are there each day?
///
/// `N_pass = 9`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "com_passes_per_day";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x8350254bd75ce1a2;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(9.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "N_pass", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "N_pass", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "N_pass", value: answer.get(), bound: 0.5, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below half a pass a day the mission cannot be operated" });
    }
    if answer.get() > 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "N_pass", value: answer.get(), bound: 60.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 60 passes a day exceeds the number of revolutions in a day" });
    }
    Ok(answer)
}
