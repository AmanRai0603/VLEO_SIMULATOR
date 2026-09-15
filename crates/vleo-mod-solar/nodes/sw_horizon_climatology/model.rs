// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// If we forecast the record's mean F10.7 and nothing else, how wrong are we L days later?
///
/// `C(L) = RMS[F107(t+L) - 114.8437], from the record`
///
/// Source: `noaa_swpc`
///
/// The baseline that makes sw_horizon_persistence mean something. On its own
/// this number says little; against persistence it says which of the two
/// cheapest forecasts is worth using, and the lead where the answer changes
/// is the horizon prf_horizon exists to find.
///
/// # Assumptions
///
/// * It is nearly flat, and it is the record's own standard deviation — fails when the climatology forecast ignores the lead entirely, so its error is the spread of the target days about the mean and nothing more: 44.39 sfu at a one-day lead, 45.32 at two years. The whole variation across eighteen leads is under 1 sfu, and it comes from the set of target days shifting rather than from anything getting harder. Anyone expecting a baseline that degrades with lead is expecting the wrong baseline — that is the point of using it as one.
/// * The crossover is the design answer, and this row does not state it — fails when read against sw_horizon_persistence, persistence is the better forecast out to about 547 days and is beaten by 730 — so today's flux is worth something for roughly a year and a half, which is longer than the 27-day outlook horizon would suggest. That crossover is the number a designer wants and neither row publishes it: it is a property of the pair, and the tree carries one answer per row. A third row could state it; none does yet.
/// * The mean is the record's unconditional mean, so this baseline knows nothing about the cycle — fails when a climatology that knew the solar cycle phase would be a much better baseline than 114.84 sfu everywhere, and would beat persistence sooner. That needs a date, and the date now exists: sys_mission_requirements_mission_epoch is published and sw_mean_cycle_level already turns it into a phase-conditioned level. So a fair baseline is buildable and is not built here — this row deliberately keeps the deaf baseline, because changing it would change what the horizon below means without changing its name. So the horizon this pair implies is the horizon against a DEAF baseline, and a fair baseline would shorten it.
/// * Only pairs of REAL observations exactly L days apart, which is again not what the MATLAB does — fails when prf_design and prf_horizon both build a full daily grid and fill the record's 273 absent days by linear interpolation. Interpolated days have no variability, so every error statistic spanning them is understated. This row pairs only days that were both observed, as sw_uncertainty_growth does, and for the same reason: a number that is partly invented is not a measurement of the record. It is therefore expected to disagree slightly with prf_horizon, which is why it carries no parity grid.
/// * Pinned to solar-weather@2026.09.14, and the [data] declaration can only pin the name — fails when crates/vleo-modules compares bundle NAMES, so a run with a later solar-weather satisfies the precondition and still uses this table. Every entry must be re-measured if the bundle version changes. The same obligation sits on every measured row in this group.
pub const NODE_ID: &str = "sw_horizon_climatology";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb1a9573a430c48a6;

pub fn evaluate(lead: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : take the lead in days and read the measured climatology RMS at that lead -> Ratio
    // The same eighteen leads, and the RMS departure of the target day from the
    // record mean of 114.8437 sfu, from solar-weather@2026.09.14.
    //
    // These values barely move — 44.39 to 45.32 across three orders of magnitude in
    // lead — because a climatology forecast ignores the lead. That flatness is the
    // point of using it as a baseline, not a defect in the measurement.
    use vleo_core::math::Table1;
    const C: Table1 = Table1 {
        x: &[
            1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 14.0, 20.0, 27.0, 40.0, 60.0, 90.0, 135.0, 180.0, 270.0,
            365.0, 547.0, 730.0,
        ],
        y: &[
            44.3929, 44.3953, 44.3968, 44.3993, 44.4012, 44.4036, 44.4069, 44.4071, 44.4147, 44.4242,
            44.4344, 44.4458, 44.4635, 44.4714, 44.5344, 44.6734, 45.0333, 45.3201,
        ],
    };
    let err: Ratio = Ratio::new(C.at(lead.days()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = err;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "D_clim", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D_clim", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "an RMS cannot be negative, and this one cannot approach zero: it is the spread of the record about its own mean, which is 44 sfu" });
    }
    if answer.get() > 50.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D_clim", value: answer.get(), bound: 50.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the measured entries run 44.39 to 45.32 sfu and the relation is a table that clamps, so no input can produce more. 50 is above both and tight enough to catch a broken table, which a bound of 65 would not" });
    }
    Ok(answer)
}
