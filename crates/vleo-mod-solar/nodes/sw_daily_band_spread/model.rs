// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far above its own rotation does a single day of F10.7 reach, at the declared confidence?
///
/// `dF107_day = pctl(F107 - movmean(F107, 27 d), 0.95) over the 2001 days ending at the window = 34.2315 sfu`
///
/// Source: `noaa_swpc`
///
/// The mean band says what the mission SUSTAINS. This says what one day
/// inside it does. They are different numbers and a design uses them for
/// different things: an array is sized on the sustained level, a thermal case
/// and a drag transient on the day. A tool that publishes only the sustained
/// number has decided for the reader which of the two their problem is.
///
/// # Assumptions
///
/// * One number holds for the whole window, and the window is a year — fails when the spread is not constant across a cycle — it widens near maximum as active regions grow — so a single percentile measured on the 2001 days before the window is too wide for a quiet stretch inside it and too narrow for an active one. The MATLAB source says the same thing about its own sigma in as many words: 'sigma is NOT flat across the cycle; a design that uses one number is too tight somewhere and too loose somewhere else'
/// * The 27-day moving mean is the rotation — fails when the solar rotation is 27.27 days at the equator and slower at the poles, and the active longitudes that drive F10.7 are not at one latitude. A 27-day window is the conventional round number rather than a measured period, and the departures it leaves carry whatever the mismatch contributes
/// * It is the record's own daily scatter, not a forecast error — fails when this is read as an uncertainty. It is not: it says how variable the sun is within a rotation, measured on days that already happened. What a forecast of a future day would get wrong is sw_uncertainty_growth's question and a larger number
pub const NODE_ID: &str = "sw_daily_band_spread";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x934b723e5d29cc7b;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(34.2314814815, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "dF107_day", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "dF107_day", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dF107_day", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a spread of zero would mean every day equals its rotation mean, which the record contradicts on every day it holds; below zero is not a spread" });
    }
    if answer.get() > 120.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dF107_day", value: answer.get(), bound: 120.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 120 sfu the departure exceeds the largest single-day excursion in the record, so a value there is an arithmetic error rather than an active sun" });
    }
    Ok(answer)
}
