// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far above its own rotation does a single day of Ap reach, at the declared confidence?
///
/// `dAp_day = pctl(Ap - movmean(Ap, 27 d), 0.95) over the 2001 days ending at the window = 15.0019`
///
/// Source: `noaa_swpc`
///
/// The Ap twin of sw_daily_band_spread. A storm is a day, not a rotation, and
/// this is the row that says how much of one a single day can be.
///
/// # Assumptions
///
/// * One number holds for the whole window — fails when the spread is not constant across a cycle and is widest in the declining phase when high-speed streams recur. A single percentile is too wide for a quiet stretch and too narrow for an active one
/// * A percentile of a burst process is a useful design number — fails when it is read as a worst case. The 95th percentile of the daily departure is exceeded one day in twenty — about eighteen times in a 365-day window — and the storms a design actually fears sit far out in a tail this row says nothing about
/// * The 27-day moving mean is the rotation — fails when recurrent high-speed streams have their own 27-day periodicity, so part of what this calls a daily departure is itself rotation-locked and is being removed by the very mean it is measured against
pub const NODE_ID: &str = "sw_ap_daily_band_spread";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x646173cad08672fc;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(15.0018518519, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "dAp_day", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "dAp_day", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dAp_day", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a value below zero is not a spread, and Ap itself floors at zero — a quiet day really is Ap 0" });
    }
    if answer.get() > 150.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dAp_day", value: answer.get(), bound: 150.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 150 the value exceeds anything the record supports for this quantity, so it is an arithmetic error rather than an active sun" });
    }
    Ok(answer)
}
