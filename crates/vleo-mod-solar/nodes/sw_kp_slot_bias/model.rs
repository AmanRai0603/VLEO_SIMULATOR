// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// By how much does the published table under-read the daily PEAK Kp at this Ap?
///
/// `dKp_peak(Ap) = piecewise_linear(ap_bin_centres -> measured_medians, Ap)`
///
/// Source: `noaa_swpc`
///
/// sw_kp_from_ap applies the published scale as published, and the scale is
/// defined for the three-hourly ap while the design product carries a daily
/// mean. This row measures the resulting bias for the slot that sizes a drag
/// design — the daily peak — and publishes it as a correction to be
/// added. The 24-hour-mean slot has its own, smaller and oppositely-signed
/// bias, recorded in the assumptions rather than published, because a node
/// answers one question.
///
/// # Assumptions
///
/// * The correction is a median per bin, so it describes the typical day at that Ap and not the day in hand — fails when the offset is a distribution, not a number. Adding the median recovers the typical peak and still misses any individual day, and the spread within a bin is not published by this row. A design that needs the worst case at a given Ap needs a percentile of the offset, not its median.
/// * The nine bin medians are not monotone, and the top bin sits exactly at the record's support limit — fails when the offset rises with Ap as the concavity argument predicts — +1.000 at Ap 2.5 through +1.747 at Ap 90 — and then FALLS to +1.317 in the 110-to-400 bin. That bin holds 20 days, which is exactly prf_ap2kp's own minimum for using a bin at all, so the fall is as likely to be a small-sample artefact as a real saturation of the table near its top. It is carried through rather than smoothed away, because smoothing it would be this node inventing a shape the record does not show. Anyone designing at Ap above 110 is reading a correction supported by twenty days.
/// * The 24-hour-mean slot is NOT what this publishes — fails when the two slots have opposite signs, and using this row for the mean slot would double the error rather than remove it. Measured on the same 10,297 days, the mean-slot offset runs from +0.042 at Ap 2.5 down to -0.487 in the top bin — the table reads HIGH against the 24-hour mean and LOW against the peak. DTM2020_Oper wants both: akp(1) is a single three-hourly value, akp(3) the mean of the eight. The mean-slot offset is published by sw_kp_mean_bias; this row is the peak.
pub const NODE_ID: &str = "sw_kp_slot_bias";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xde65f5eeb49f7833;

pub fn evaluate(ap: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : read the measured peak-slot offset off the nine binned medians at this Ap -> Ratio
    // The nine bin medians, measured on solar-weather@2026.09.14. The x values are
    // prf_ap2kp's own bin CENTRES — the midpoints of its edges 0 5 10 15 20 30 45
    // 70 110 400 — and the y values are the medians in each bin. They belong to
    // the record, not to this code, and the sheet names the bundle version.
    //
    // THE TABLE LIVES IN vleo-core AS `env::kp_peak_slot_bias` RATHER THAN HERE, and
    // this hole calls it. Two callers read it now: this row, at one Ap, and
    // sw_kp_scenarios, at the five the driver set carries. A table copied into
    // both would drift from itself without anything noticing — which is not
    // hypothetical, it is what happened to the ap-to-Kp scale.
    //
    // It holds the end values instead of extrapolating, which is the same choice
    // prf_ap2kp makes explicitly ("hold the end bins, never extrapolate").
    let off: Ratio = Ratio::new(vleo_core::physics::env::kp_peak_slot_bias(ap.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = off;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "dKp_peak", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dKp_peak", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp(ap) is concave, so by Jensen's inequality the daily peak three-hourly Kp cannot sit below the table's reading of the daily mean. A negative correction contradicts the inequality the whole row rests on, and means the sign or the slot has been swapped. The smallest measured bin median is +0.833" });
    }
    if answer.get() > 2.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dKp_peak", value: answer.get(), bound: 2.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the largest measured bin median is +1.747 and the relation is a table that clamps at its ends, so no input can produce more than that. 2.0 is therefore unreachable by this relation and exists to catch a broken table rather than an extreme sky" });
    }
    Ok(answer)
}
