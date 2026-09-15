// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// By how much can F10.7 rise over a lead this long, at 95% confidence?
///
/// `dF107_p95(L) = piecewise_linear(measured_leads -> measured_p95, L)`
///
/// Source: `noaa_swpc`
///
/// Half of the F10.7 design value. sw_central_expectation says where F10.7 is
/// heading; this says how wrong that can be by the time the mission is there,
/// so the band widens with lead exactly as knowledge fades. The percentile
/// does the safety: the design value is the central expectation plus this.
///
/// # Assumptions
///
/// * A table over measured leads, not a fitted curve, because the shape is the solar cycle and a smooth fit would erase it — fails when the measured p95 growth is NOT monotone in lead: it rises to +114 sfu at four years, falls to +66 at ten to eleven, and rises again to +115 at fifteen. That is the eleven-year cycle showing through — a lead of half a cycle can put you furthest from where you started, while a lead of a full cycle returns you to a similar phase. Any monotone form, log-linear included, would report roughly +90 at eleven years where the record says +66, overstating the band by a third at exactly the lead a long mission cares about. The table is faithful and the interpolation between its points is this node's choice.
/// * Only pairs of REAL observations exactly L days apart — which is not what prf_design does — fails when prf_design.m:29 builds a full daily grid from the first date to the last and fills it by linear interpolation, so the record's 273 absent days become 273 straight-line days with no variability at all, and every L-day difference spanning them is understated. Measured both ways on this record, the MATLAB's grid gives a p95 between 1.0 and 2.3 sfu LOWER across leads from 7 to 1826 days — a design band narrower than the record supports, in the unsafe direction. This node pairs only days that were both observed, which costs sample size and buys a number that is not partly invented. It is therefore expected to DISAGREE with prf_design by about that much, and a parity grid would have recorded the disagreement rather than a match.
/// * The 95th percentile, and the sample thins as the lead grows — fails when prf_design computes p50, p90, p95 and p99 and lets the caller pick the confidence the mission needs; this row publishes p95 only, because a node answers one question. At a one-year lead the four are +1, +53, +68 and +100 sfu — so a mission that needs p99 is reading a number 32 sfu too small here. The pair count also falls from 9,947 at a half-year lead to 4,838 at fifteen years, and twenty-eight years of record hold barely two and a half solar cycles, so the longest leads are sampled by very few independent cycle phases.
pub const NODE_ID: &str = "sw_uncertainty_growth";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x4b1bd73cba36bfab;

pub fn evaluate(lead: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : take the lead in days and read the measured 95th-percentile F10.7 growth at that lead -> Ratio
    // Seventeen leads, seventeen measured percentiles, from
    // solar-weather@2026.09.14. x is the lead in DAYS, y the 95th percentile of
    // F107(t+L) - F107(t) over pairs where both days were observed.
    //
    // The y values fall and rise again — +114 at four years, +66 at ten, +115 at
    // fifteen — and that is the eleven-year cycle, not noise. The sheet says why a
    // table and not a fitted curve, and why these numbers are expected to sit
    // 1 to 2.3 sfu ABOVE prf_design's, which interpolates the record's absent days.
    //
    // Table1::at clamps at both ends rather than extrapolating: past fifteen years
    // the record has too few pairs to say anything new, and below half a year the
    // declared input range does not reach.
    use vleo_core::math::Table1;
    const GROWTH: Table1 = Table1 {
        x: &[
            183.0, 365.0, 548.0, 730.0, 1096.0, 1461.0, 1826.0, 2191.0, 2557.0, 2922.0, 3287.0,
            3653.0, 4018.0, 4383.0, 4748.0, 5113.0, 5478.0,
        ],
        y: &[
            57.0, 68.3, 77.0, 91.0, 107.0, 114.0, 113.3, 104.0, 92.0, 88.0, 78.0, 66.0, 66.0, 76.0,
            91.0, 106.0, 115.0,
        ],
    };
    let growth: Ratio = Ratio::new(GROWTH.at(lead.days()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = growth;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "dF107_p95", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dF107_p95", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the 95th percentile of the change in F10.7 over a lead of at least half a year is positive everywhere in this record — the smallest measured is +57 sfu. A negative value means the difference has been taken the wrong way round, which would turn a safety margin into a reduction" });
    }
    if answer.get() > 120.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dF107_p95", value: answer.get(), bound: 120.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the largest measured percentile is +115.0 sfu at a fifteen-year lead and the relation is a table that clamps at its ends, so no input can produce more. 120 is unreachable by this relation and exists to catch a broken table rather than an extreme sky" });
    }
    Ok(answer)
}
