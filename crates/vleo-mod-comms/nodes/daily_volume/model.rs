// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much data can the system bring down in a day?
///
/// `V = V_pass*N_pass*A`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_daily_volume";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x47dae619cfcce246;

pub fn evaluate(v: DataVolume, n: Ratio) -> Result<DataVolume, Fault> {
    // ---- HOLE 1 : multiply by the usable passes per day at 95% station availability -> DataVolume
    let vd: DataVolume = comms::daily_downlink_volume(v, n.get(), Ratio::new(0.95));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: DataVolume = vd;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "V_day", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V_day", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: DataVolume::UNIT, reason: "a data volume cannot be negative" });
    }
    if answer.get() > 1e16 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "V_day", value: answer.get(), bound: 1e16, edge: Edge::Upper, unit: DataVolume::UNIT, reason: "above 10 petabits a day no ground segment in this design receives it" });
    }
    Ok(answer)
}
