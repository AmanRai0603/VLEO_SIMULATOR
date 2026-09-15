// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// If all we know is today's F10.7, how wrong are we L days later?
///
/// `D(L) = RMS[F107(t+L) - F107(t)], from the record`
///
/// Source: `noaa_swpc`
///
/// prf_horizon's structure function D(L) = RMS[F107(t+L) - F107(t)], its
/// first method. Paired with sw_horizon_climatology it answers the design
/// question directly: which of the two cheapest forecasts is worth using at
/// this lead, and what either costs.
///
/// # Assumptions
///
/// * The error DIPS at 27 days, and that is the Sun's rotation rather than noise — fails when persistence error rises from 7.1 sfu at one day to 29.1 at fourteen, then FALLS to 22.4 at twenty-seven before rising again. Twenty-seven days is the synodic solar rotation: the same active region comes back round, so today's flux is a better guide to the flux one rotation from now than to the flux a fortnight from now. Any monotone model of predictability erases that, and it is the one feature of this curve a forecaster would actually use. It is also why the row is a table and not a fit.
/// * The declared input cannot reach the interesting part of the curve — fails when orbit_mission_duration is declared 0.5 to 15 years, so in a real run this row is only ever asked about leads of 183 days and up — past the rotation dip, past the rise, in the slow tail. The table carries the short-lead structure because the question is about predictability and the structure is the answer, but nothing in this tree currently asks for it. A 27-day outlook wants a lead this tree does not publish.
/// * Persistence has not saturated even at two years, because F10.7 is cyclic and not a random walk — fails when for a process with no memory the structure function saturates at sqrt(2) times the standard deviation, which is 62.77 sfu here. The measured value at a two-year lead is 49.63 and still climbing. So the textbook saturation is not reached inside this record's measurable range, and reading the tail as if it had converged would understate how much worse a longer lead still gets.
/// * Only pairs of REAL observations exactly L days apart, which is again not what the MATLAB does — fails when prf_design and prf_horizon both build a full daily grid and fill the record's 273 absent days by linear interpolation. Interpolated days have no variability, so every error statistic spanning them is understated. This row pairs only days that were both observed, as sw_uncertainty_growth does, and for the same reason: a number that is partly invented is not a measurement of the record. It is therefore expected to disagree slightly with prf_horizon, which is why it carries no parity grid.
/// * Pinned to solar-weather@2026.09.14, and the [data] declaration can only pin the name — fails when crates/vleo-modules compares bundle NAMES, so a run with a later solar-weather satisfies the precondition and still uses this table. Every entry must be re-measured if the bundle version changes. The same obligation sits on every measured row in this group.
pub const NODE_ID: &str = "sw_horizon_persistence";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x340e2b7af3e268da;

pub fn evaluate(lead: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : take the lead in days and read the measured persistence RMS at that lead -> Ratio
    // Eighteen leads, eighteen measured RMS values, from
    // solar-weather@2026.09.14. x is the lead in DAYS, y the RMS of
    // F107(t+L) - F107(t) over pairs where both days were observed.
    //
    // The dip at x = 27.0 is deliberate and is not a transcription error: the
    // synodic solar rotation brings the same active region back round, so
    // persistence is BETTER at one rotation than at a fortnight. Any smoothing of
    // this table removes the one feature of it a forecaster would use.
    use vleo_core::math::Table1;
    const D: Table1 = Table1 {
        x: &[
            1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 14.0, 20.0, 27.0, 40.0, 60.0, 90.0, 135.0, 180.0, 270.0,
            365.0, 547.0, 730.0,
        ],
        y: &[
            7.0784, 10.3796, 13.6671, 19.3021, 23.6294, 27.6511, 29.1328, 25.4695, 22.4321, 29.602,
            27.9575, 30.1549, 31.0394, 32.9634, 35.7538, 38.8277, 43.7693, 49.6285,
        ],
    };
    let err: Ratio = Ratio::new(D.at(lead.days()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = err;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "D_pers", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D_pers", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "an RMS cannot be negative. Zero would mean persistence is exact, which is true only at a lead of zero and is not a lead this row is asked about" });
    }
    if answer.get() > 65.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D_pers", value: answer.get(), bound: 65.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "for a memoryless process the structure function saturates at sqrt(2)*sigma, which is 62.77 sfu on this record, and the largest measured entry is 49.63 at a two-year lead. 65 is above both, so it is unreachable by this relation and exists to catch a broken table rather than an extreme sky" });
    }
    Ok(answer)
}
