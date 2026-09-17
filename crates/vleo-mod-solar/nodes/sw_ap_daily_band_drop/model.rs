// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far below its own rotation does a single day of Ap fall, at the declared confidence, at the level that rotation sits at?
///
/// `dAp_day_low(L) = -pctl(Ap - movmean(Ap, 27 d), 0.05) over days whose rotation sits within 15% of L`
///
/// Source: `noaa_swpc`
///
/// The cold tail of sw_ap_daily_band_spread's sample, and a separate row
/// because Ap's two tails are nothing like each other at any level. At a
/// rotation of Ap 11 they are 14.67 up and 7.56 down; at Ap 26, 63.85 and
/// 21.02 — the high tail is three times the low one. The geomagnetic index
/// is the most skewed quantity in this subsystem, because it floors hard at
/// zero and has no ceiling at all. The cold day is the low-drag case, and on
/// the Ap side it is also the quiet-sky case a magnetometer, a magnetorquer
/// sizing and an aerodynamic control authority are bounded by from below.
///
/// # Assumptions
///
/// * The result stays above zero, and nothing in the arithmetic ensures it — fails when the sustained quiet level is small. With this table the crossing is at a sustained Ap near 15.6, and the declared window's 17.50 clears it by under two units of level — where before, with a single 10.59, the crossing was at 15.2. Conditioning on level did not move that crossing much because the drop falls with the level too, but the margin on the ANSWER narrowed from seven to five. sw_ap_cold_short's guard refuses rather than publishing a negative index, and it is the row most likely in this subsystem to fire
/// * Ap's departures are strongly asymmetric at every level, and this row exists because of it — fails when it is treated as the negation of sw_ap_daily_band_spread. The high tail is twice the low one at Ap 4 and three times at Ap 26. Anything that mirrors one onto the other is wrong by a factor of two to three, and in the direction that puts the cold day below zero
/// * A percentile of departures does not know that Ap floors at zero — fails when the rotation mean it is subtracted from is smaller than the drop. The table is measured from a sample that is itself truncated, so the low tail already reflects the floor implicitly — but the interpolation between knots does not, and neither does the subtraction in sw_ap_cold_short. The guard there is what stops it, not the statistic here
/// * A 15 per cent bin half-width, which trades resolution against sample size — fails when either matters more than the other. The declared window sits between the Ap 15 knot, holding 1558 days, and the Ap 20 knot, holding 803. Both are well populated, so this row's own answer is on firmer ground than its high-tail twin's, which is reading the table's top knot at 300 days
/// * The 27-day moving mean is the rotation — fails when Ap's recurrence is driven by coronal holes and high-speed streams whose period is nearer 27.0 days than the 27.27 of the equatorial photosphere. A 27-day window is the conventional round number rather than a measured period
/// * It is the record's own daily scatter, not a forecast error — fails when this is read as an uncertainty. It says how far below its rotation the geomagnetic field has gone, measured on days that already happened
pub const NODE_ID: &str = "sw_ap_daily_band_drop";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xce5c88ee53de7733;

pub fn evaluate(level: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : read the 5th-percentile within-rotation Ap departure off the measured table at this rotation level, as a magnitude -> Ratio
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
    const LO: Table1 = Table1 {
        // rotation level, Ap
        x: &[4.0, 6.0, 8.0, 11.0, 15.0, 20.0, 24.0, 26.0],
        // 5th-percentile departure below that rotation, as a MAGNITUDE. The sign
        // belongs to sw_ap_cold_short, which subtracts it.
        y: &[2.9704, 4.1685, 5.5926, 7.5556, 10.8370, 14.2963, 17.6926, 21.0185],
    };
    let band: Ratio = Ratio::new(LO.at(level.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = band;
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
