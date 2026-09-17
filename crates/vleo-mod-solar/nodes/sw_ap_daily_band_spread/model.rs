// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far above its own rotation does a single day of Ap reach, at the declared confidence, at the level that rotation sits at?
///
/// `dAp_day(L) = pctl(Ap - movmean(Ap, 27 d), 0.95) over days whose rotation sits within 15% of L`
///
/// Source: `noaa_swpc`
///
/// The geomagnetic index is the most level-dependent quantity in this
/// subsystem. At a rotation level of Ap 4 a one-in-twenty day is 6.0 above
/// it; at Ap 26 it is 63.9 — a factor of eleven, and most of it in the top
/// third of the range, because high-Ap rotations are driven by recurrent
/// streams and storms and their individual days swing enormously. A single
/// number for that is not a band, it is an average of bands, and it is
/// correct only near the mean level of whatever sample produced it.
///
/// # Assumptions
///
/// * THE DECLARED WINDOW IS ABOVE THE TOP KNOT, so its answer is the clamp and not an interpolation — fails when the sustained Ap exceeds 26, as it does here at 26.70. The table holds its top value rather than extrapolating, and the segment below that top knot is the steepest in the table — 40.61 at Ap 24 to 63.85 at 26. Extrapolating that slope to 26.70 would give about 72 rather than 63.85, so the clamp is the CONSERVATIVE-DOWNWARD choice here and the published single-day Ap may be low. The record cannot settle it: Ap 28 holds 170 days and Ap 30 holds 105
/// * Conditioning on level more than doubled this row's consumer, and the previous value was not a margin but an error — fails when this is read as a refinement. sw_ap_design_short was 41.70 and is 90.55. A design sized on 41.70 for its own hot scenario was under-designed by a factor of two, because the band it used was measured over a sample whose rotations averaged Ap 11.64 and applied at 26.70
/// * The top of the table rests on very little record — fails when a design sits near it. The Ap 26 knot holds 300 days, and the whole record has only 27 rotation-sized blocks above Ap 22 in 28.2 years. A 95th percentile of 300 days is the fifteenth-largest of them, so the top of this table moves if one storm period is added to or removed from the record. The bottom knots hold thousands of days and are not like this, and the table does not say which regime a caller is in at the point of use
/// * A disturbed rotation is not a quiet rotation scaled up — fails when somebody fits a smooth law through these knots. The rise is a factor of eleven and most of it is in the last third, because high-Ap rotations contain storms and a storm day's departure has little to do with its rotation's own level. A power law or a constant ratio through this table would be wrong at both ends
/// * The 27-day moving mean is the rotation — fails when Ap's recurrence is driven by coronal holes and high-speed streams whose period is nearer 27.0 days than the 27.27 of the equatorial photosphere, and they persist for many rotations. A 27-day window is the conventional round number rather than a measured period
/// * It is the record's own daily scatter, not a forecast error — fails when this is read as an uncertainty. It says how variable the geomagnetic field is within a rotation, measured on days that already happened
pub const NODE_ID: &str = "sw_ap_daily_band_spread";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x8a5d9ce151df35d4;

pub fn evaluate(level: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : read the 95th-percentile within-rotation Ap departure off the measured table at this rotation level -> Ratio
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
        // rotation level, Ap
        x: &[4.0, 6.0, 8.0, 11.0, 15.0, 20.0, 24.0, 26.0],
        // 95th-percentile departure above that rotation. The last segment is the
        // steepest in either table — 40.61 at Ap 24 to 63.85 at 26 — and the
        // declared window reads the clamp just past its end.
        y: &[6.0167, 8.1481, 11.2222, 14.6667, 23.7852, 28.6759, 40.6148, 63.8519],
    };
    let band: Ratio = Ratio::new(HI.at(level.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = band;
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
