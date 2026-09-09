// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the customer require for daily downlink volume?
///
/// `R_req = 500`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "kpi_data_volume_required";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xee7a20539b6cc895;

pub fn evaluate() -> Result<DataVolume, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: DataVolume = match DataVolume::from_unit(500.0, Unit::Gigabit) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "R_req", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: DataVolume = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "R_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_req", value: answer.get(), bound: 1000000000.0, edge: Edge::Lower, unit: DataVolume::UNIT, reason: "below one gigabit a day the mission returns nothing useful" });
    }
    if answer.get() > 1e16 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_req", value: answer.get(), bound: 1e16, edge: Edge::Upper, unit: DataVolume::UNIT, reason: "above 10 petabits a day the ground segment is not the one costed" });
    }
    Ok(answer)
}
