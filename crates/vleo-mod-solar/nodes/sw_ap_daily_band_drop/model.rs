// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far below its own rotation does a single day of Ap fall, at the declared confidence?
///
/// `dAp_day_low = -pctl(Ap - movmean(Ap, 27 d), 0.05) over the 2001 days ending at the window = 10.5889`
///
/// Source: `noaa_swpc`
///
/// The cold tail of sw_ap_daily_band_spread's sample, and a separate row
/// because Ap's two tails are nothing like each other: 15.00 up against 10.59
/// down. The geomagnetic index is the most skewed quantity in this subsystem,
/// because it floors hard at zero and has no ceiling at all. The cold day is
/// the low-drag case, and on the Ap side it is also the quiet-sky case a
/// magnetometer, a magnetorquer sizing and an aerodynamic control authority
/// are bounded by from below.
///
/// # Assumptions
///
/// * Ap's departures are strongly asymmetric, and this row exists because of it — fails when it is treated as the negation of sw_ap_daily_band_spread. It is forty-two per cent smaller, against nine per cent on the F10.7 side. Anything that mirrors one tail onto the other puts the cold Ap day two and a half units below what the record supports
/// * A percentile of departures does not know that Ap floors at zero — fails when the rotation mean it is subtracted from is smaller than the drop. Below a mean of about 11 this row carries Ap negative, which is arithmetic rather than sky. The declared window's 17.50 is clear of it, and the consumer guards its own output, but the statistic itself has no such knowledge
/// * One number holds for the whole window, and the window is a year — fails when the spread is not constant across a cycle — it widens near maximum — so a single percentile measured on the 2001 days before the window is too wide for a quiet stretch inside it and too narrow for an active one
/// * The 27-day moving mean is the rotation — fails when Ap's recurrence is driven by coronal holes and high-speed streams whose period is nearer 27.0 days than the 27.27 of the equatorial photosphere, and they persist for many rotations. A 27-day window is the conventional round number rather than a measured period, and the departures it leaves carry whatever the mismatch contributes
/// * It is the record's own daily scatter, not a forecast error — fails when this is read as an uncertainty. It is not: it says how far below its rotation the geomagnetic field has gone, measured on days that already happened
pub const NODE_ID: &str = "sw_ap_daily_band_drop";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe69a98e7820a0d01;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(10.5888888889, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "dAp_day_low", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "dAp_day_low", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dAp_day_low", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a drop of zero would mean no day ever falls below its rotation mean, which the record contradicts on about half the days it holds; below zero is not a distance, and a negative value here means the tail has been read from the wrong end" });
    }
    if answer.get() > 150.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dAp_day_low", value: answer.get(), bound: 150.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 150 the value exceeds anything the record supports for this quantity, so it is an arithmetic error rather than a quiet sky" });
    }
    Ok(answer)
}
