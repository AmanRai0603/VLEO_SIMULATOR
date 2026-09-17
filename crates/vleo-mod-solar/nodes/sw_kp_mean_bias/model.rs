// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// By how much does the published table over-read the 24-hour MEAN Kp at this Ap?
///
/// `dKp_mean(Ap) = piecewise_linear(ap_bin_centres -> measured_medians, Ap)`
///
/// Source: `noaa_swpc`
///
/// The pair to sw_kp_slot_bias, and the half of the pair a thermospheric
/// model actually asks for more often. DTM2020_Oper takes Kp in two senses
/// — akp(1) a single three-hourly value, akp(3) the mean of the day's eight
/// — and the published scale answers neither, because it is defined for a
/// three-hourly ap while the design product carries a daily mean.
/// sw_kp_slot_bias measures the peak-slot gap and this measures the mean-slot
/// one. They have opposite signs, so applying the wrong one doubles the error
/// instead of removing it.
///
/// # Assumptions
///
/// * The correction is a median per bin, so it describes the typical day at that Ap and not the day in hand — fails when the offset is a distribution, not a number. Adding the median recovers the typical 24-hour mean and still misses any individual day, and the spread within a bin is not published by this row. A design that needs the worst case at a given Ap needs a percentile of the offset, not its median.
/// * The quietest bin measures the opposite sign to the one Jensen's inequality requires — fails when the Ap 0-to-5 bin measures +0.042 against an argument that says the value must be at or below zero. The two candidate causes are that ap_planetary is SWPC's ESTIMATED planetary amplitude rather than the exact mean of the day's eight ap — so the concavity argument is being applied to a number it does not quite describe — and that a twenty-fourth of a Kp unit is below the resolution of either published scale at the quiet end. Neither is settled here. It matters least where it occurs, because a design is not sized by the quietest 2529 days in the record, but a reader who assumes the published sign holds everywhere will be wrong in one bin of nine.
/// * The top two bins rest on 25 and 20 days — fails when prf_ap2kp's own minimum for using a bin at all is 20 days, and the 110-to-400 bin sits exactly on it. The correction a design reads above Ap 110 — the largest correction this row publishes, -0.487 — is supported by twenty days. That is stated rather than smoothed, and anyone sizing a design there should know the number is thin rather than discover it later.
/// * This is NOT the daily-peak slot — fails when the two slots have opposite signs. The peak-slot offset measured on the same 10,297 days runs from +0.833 to +1.747; this one runs from +0.042 to -0.487. Applying this row where sw_kp_slot_bias was wanted moves the answer the wrong way by roughly one and a half Kp, and the result is still a plausible Kp, so nothing downstream will refuse it.
pub const NODE_ID: &str = "sw_kp_mean_bias";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb74115f951260a2b;

pub fn evaluate(ap: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : read the measured mean-slot offset off the nine binned medians at this Ap -> Ratio
    // The nine bin medians, measured on solar-weather@2026.09.14. The x values are
    // prf_ap2kp's own bin CENTRES — the midpoints of its edges 0 5 10 15 20 30 45
    // 70 110 400 — and the y values are the medians in each bin. They belong to
    // the record, not to this code, and the sheet names the bundle version.
    //
    // THE TABLE LIVES IN vleo-core AS `env::kp_mean_slot_bias` RATHER THAN HERE, and
    // this hole calls it. Two callers read it now: this row, at one Ap, and
    // sw_kp_scenarios, at the five the driver set carries. A table copied into
    // both would drift from itself without anything noticing — which is not
    // hypothetical, it is what happened to the ap-to-Kp scale.
    //
    // It holds the end values instead of extrapolating, which is the same choice
    // prf_ap2kp makes explicitly ("hold the end bins, never extrapolate").
    let off: Ratio = Ratio::new(vleo_core::physics::env::kp_mean_slot_bias(ap.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = off;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "dKp_mean", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dKp_mean", value: answer.get(), bound: -1.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the deepest measured bin median is -0.487 and the relation is a table that clamps at its ends, so no input can produce less. -1.0 is therefore unreachable by this relation and exists to catch a broken table rather than an extreme sky" });
    }
    if answer.get() > 0.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dKp_mean", value: answer.get(), bound: 0.5, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Jensen's inequality puts this at or below zero wherever the published Ap is the mean of the day's eight ap, and the one bin that measures positive measures +0.042. +0.5 would say the table under-reads the 24-hour mean by half a Kp, which no bin shows and which is the signature of the peak-slot offset having been put in this row by mistake" });
    }
    Ok(answer)
}
