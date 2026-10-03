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
pub const SHEET_HASH: u64 = 0x2cf8cd3bd55d07a1;

pub fn evaluate(lead: Time) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::sw_horizon_climatology. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::sw_horizon_climatology::evaluate(lead.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "D_clim")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
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
