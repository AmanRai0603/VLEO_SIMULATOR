// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far above its own rotation does a single day of F10.7 reach, at the declared confidence, at the level that rotation sits at?
///
/// `dF107_day(L) = pctl(F107 - movmean(F107, 27 d), 0.95) over days whose rotation sits within 15% of L`
///
/// Source: `noaa_swpc`
///
/// The mean band says what the mission SUSTAINS. This says what one day
/// inside it does. They are different numbers and a design uses them for
/// different things: an array is sized on the sustained level, a thermal case
/// and a drag transient on the day. AND IT DEPENDS ON THE LEVEL, which is the
/// whole content of this row. A quiet sun is quiet in both senses — its
/// rotations sit low AND its days stay near their rotation. At a rotation
/// level of 70 sfu a one-in-twenty day is 4.6 above it; at 210 it is 46.9.
/// Any construction that applies one number at both is wrong at one end, and
/// which end depends on where its sample happened to sit.
///
/// # Assumptions
///
/// * The departure scales with the level, and a single number for it is wrong away from its own sample's mean — fails when this is read as a refinement. It is a correction. Over the whole record the 95th-percentile departure runs 4.63 at a rotation level of 70 to 46.93 at 210 — a factor of ten — and the previous version of this row published one number, 34.23, measured on a 2001-day window whose rotations averaged 136.63. Applied at 69.63 it took the cold chain below the floor of 60 and sw_f107_cold_short refused. The refusal was the construction failing, not the guard being tight
/// * The table is clamped beyond its ends rather than extrapolated — fails when a design level falls outside 70 to 210 sfu. Below 70 the answer is held at 4.63 and above 210 at 46.93. The record's own rotation means run from about 65 to 253, so the top of that range is reachable by a mission at cycle maximum, and a level above 210 gets a band measured at 210 — narrower than the record supports. The clamp is deliberate and it is the same choice sw_kp_from_ap makes at the ends of the published Kp scale
/// * A 15 per cent bin half-width, which trades resolution against sample size — fails when either matters more than the other. Narrower bins resolve the level dependence better and hold fewer days, so the percentile gets noisier; wider bins are the defect this row exists to fix, in miniature. 15 per cent keeps every bin above 2000 days except the top one, at 850, and still separates the ends by a factor of ten
/// * The knots rest on the whole record, and the record is 28.2 years — fails when the top knot is leaned on. 210 sfu holds 850 days against 3061 at 70, and those days are concentrated in two cycle maxima. A design at a high sustained level is reading a percentile with far less behind it than one at a low level, and the table does not say so at the point of use
/// * The 27-day moving mean is the rotation — fails when the solar rotation is 27.27 days at the equator and slower at the poles, and the active longitudes that drive F10.7 are not at one latitude. A 27-day window is the conventional round number rather than a measured period, and the departures it leaves carry whatever the mismatch contributes
/// * It is the record's own daily scatter, not a forecast error — fails when this is read as an uncertainty. It is not: it says how variable the sun is within a rotation, measured on days that already happened. What a forecast of a future day would get wrong is sw_uncertainty_growth's question and a larger number
pub const NODE_ID: &str = "sw_daily_band_spread";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xd70a3fff1375d700;

pub fn evaluate(level: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::sw_daily_band_spread. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::sw_daily_band_spread::evaluate(level.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "dF107_day")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
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
