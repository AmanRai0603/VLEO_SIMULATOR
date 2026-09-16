// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What daily planetary Ap must the design survive on a single day inside the mission window?
///
/// `Ap_short = Ap_long + dAp_day`
///
/// Source: `noaa_swpc`
///
/// The one a drag transient and a single-orbit attitude case are sized on.
/// Its sustained sibling is what a propellant budget integrates.
pub const NODE_ID: &str = "sw_ap_design_short";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xab449467ed97381b;

pub fn evaluate(sustained: Ratio, daily: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : add the within-rotation daily departure to the sustained Ap level -> Ratio
    // The rotation error and the within-rotation departure are of different
    // things, so they add. For Ap the daily term is more than half the answer:
    // the level is quiet and the day is not.
    let single_day: Ratio = sustained + daily;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = single_day;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_short", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_short", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero, so a single-day design level below it means a spread has been subtracted rather than added" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_short", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the top of the Ap index itself; a value above it is not a geomagnetic index at all, and this row — a sustained level plus a daily excursion — is the one in the subsystem most likely to reach for it" });
    }
    Ok(answer)
}
