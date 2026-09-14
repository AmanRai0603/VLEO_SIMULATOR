// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What daily planetary Ap recurs once per mission lifetime?
///
/// `Ap(T) = 92.5155 + 40.9265*ln(T / 1 yr)`
///
/// Source: `noaa_swpc`
///
/// Geomagnetic activity has no usable long-term forecast, so a design does
/// not predict the storm — it sizes for the worst one the mission is likely
/// to meet. That makes the mission length the input: a five-year mission and
/// a fifteen-year one are owed different skies, and this row moves when
/// orbit_mission_duration moves.
///
/// # Assumptions
///
/// * The tail is log-linear in the return period, with the two coefficients fitted on this record — fails when the form is a choice, not the source's. Fitted over ranks 2 to 56 of the record, which is return periods 0.5035 to 14.0986 years; it carries a residual rms of 4.95 Ap against the empirical curve, worst +8.6 and -12.9. A power law on the same points is more than twice as bad (rms 9.10, worst -47.7), which is why this form and not that one. Anyone who needs the empirical step rather than a smooth curve should read the record, not this row.
/// * 28.2 years of record support the whole curve, and its top end rests on two observations — fails when the empirical method cannot see past its own record length. At T = 15 years the answer is fitted through the second-largest daily Ap in 28.2 years, and at T = 10 years the third; the record's largest value, Ap 273, has an apparent return period of exactly 28.2 years for no reason other than that it is the largest thing in 28.2 years. The fitted domain is 0.5035 to 14.0986 years, and the declared input range 0.5 to 15 years reaches past BOTH ends of it: the curve is extrapolated by 0.285 Ap at a half-year mission and by 2.54 Ap at a fifteen-year one, because there is no rank between 1 and 2 and rank 1 is the record length itself. Small, but it is an extrapolation and an earlier version of this sheet claimed it was never one. A mission at the 15-year bound is being sized on a curve whose top is two data points. A design that needs the once-per-century storm needs a longer record or a fitted extreme-value model, not this row.
/// * Every day in the record is treated as an independent draw — fails when storms cluster — a coronal hole returns once per solar rotation and a single event runs for more than one day — so the record holds fewer independent storms than it holds storm days. Clustering does not bias the exceedance level itself, which is a quantile of the marginal distribution, but it does mean the effective sample behind the tail is smaller than N and the uncertainty on the answer is wider than the residual above suggests. sw_event_duration and sw_recurrence_lag measure the clustering; neither is written yet.
/// * The record this is fitted on is missing 273 days, and the missing window contains storms — fails when 2017-01-01 to 2017-09-30 has no rows in the daily table — nine months absent from a 29-year span, and the node's answer is a tail quantile of what remains. The hole is not empty of weather: the same bundle's alerts.csv records four days inside it with an OBSERVED K of 7 or more, including K = 8 three times on 7-8 September 2017. Bounding those days from the alerts (their other three-hourly intervals held at the record median) puts 2017-09-08 at a daily Ap of at least 104 and 2017-09-07 at 48, which would rank about 24th and 43rd in the record. Restoring them moves the five-year level by -0.0 Ap and the fifteen-year by -0.7: the tail is set by events far larger, so this particular gap does not bias this particular answer. A different gap, or one holding a top-ten storm, would.
/// * ap_planetary is the SWPC estimated planetary A, not the GFZ definitive value — fails when anyone reconciling these return levels against GFZ's definitive Ap series will find differences, and they are not transcription errors — they are two different published quantities. The bundle's INDEX.md says so; it is repeated here because a node author reads the sheet and not necessarily the bundle.
pub const NODE_ID: &str = "sw_storm_return_level";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x96809314a0af8c3b;

pub fn evaluate(life: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : take the mission length in years and read the fitted exceedance curve at that return period -> Ratio
    // The fit, stated where it can be read rather than buried. Both numbers are
    // the sheet's, fitted on bundles/solar-weather@2026.09.14 over ranks 2 to 56
    // — return periods 0.5035 to 14.0986 years — and they belong to the record,
    // not to this code. They are pinned to that bundle version; the [data]
    // declaration can only pin the name, and the sheet says so.
    const A: f64 = 92.515531;
    const B: f64 = 40.926516;
    // 365.25 days, the same Julian year prf_design divides by, so the two
    // definitions of "a year" cannot drift apart between the record and the fit.
    let years: f64 = life.days() / 365.25;
    // ln needs a positive argument, and a zero or negative mission length is a
    // caller's error rather than a mission — the declared lower bound on the
    // answer catches those, because ln of something tiny lands far below it.
    //
    // NaN is NOT one of those cases and must not be smuggled into them.
    // `f64::max` returns the OTHER operand when one side is NaN, so the obvious
    // `years.max(1.0e-9)` turns a NaN mission length into 1e-9, and the node then
    // refuses with "the fit reached somewhere it wasn't meant to go" and a
    // concrete-looking Ap of -755.6. That sends whoever is debugging it after the
    // fit instead of after their NaN. Let a NaN stay a NaN and the generated
    // finite check below names it for what it is.
    let safe: f64 = if years.is_nan() { f64::NAN } else { years.max(1.0e-9) };
    let level: Ratio = Ratio::new(A + B * pmath::ln(safe));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = level;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_T", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 20.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_T", value: answer.get(), bound: 20.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "ap is an equivalent amplitude in nanotesla. Below 20 the answer is not a storm at all — the record's median day is 7 — so a return level under it means the input or the fit reached somewhere neither was meant to go" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_T", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the last point of the published ap/Kp table, above which Kp is no longer distinguished, and the largest daily Ap in 28.2 years of record is 273. A return level above 400 is the fit extrapolating past everything that supports it" });
    }
    Ok(answer)
}
