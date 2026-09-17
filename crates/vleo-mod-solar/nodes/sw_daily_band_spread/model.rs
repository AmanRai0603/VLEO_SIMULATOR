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
pub const SHEET_HASH: u64 = 0x00ef56c98f2601bb;

pub fn evaluate(level: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : read the 95th-percentile within-rotation departure off the measured table at this rotation level -> Ratio
    // The table is MEASURED DATA, not a published relation, and it is declared
    // here rather than in the sheet because a sheet holds one number and this is
    // a curve. Every knot is a percentile of the record conditioned on rotation
    // level, derived by the method the theory tab sets out — bins of 15 per
    // cent, at least 200 days each, monotone in both tails, the ladder stopping
    // at the last knot that passes both rules.
    //
    // Table1::at CLAMPS at both ends rather than extrapolating, which is the
    // clamp the sheet declares rather than a convenience: past the ends the
    // record does not continue, and a straight line drawn onward would be this
    // node inventing sky the record never showed.
    use vleo_core::math::Table1;
    const HI: Table1 = Table1 {
        // rotation level, sfu
        x: &[70.0, 85.0, 100.0, 120.0, 145.0, 175.0, 210.0],
        // 95th-percentile departure above that rotation, sfu
        y: &[4.6259, 11.4741, 18.2741, 27.1074, 34.6148, 40.6926, 46.9259],
    };
    let band: Ratio = Ratio::new(HI.at(level.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = band;
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
