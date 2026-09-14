// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Does the stated 95 per cent band actually contain the truth 95 per cent of the time?
///
/// `C_band = fraction of pairs with F107(t+L) - F107(t) <= dF107_p95(L), pooled over the seventeen leads`
///
/// Source: `noaa_swpc`
///
/// Counted over 128135 day pairs at the same seventeen leads
/// sw_uncertainty_growth is tabulated on. The answer is 0.9509 — the band
/// is honest and very slightly conservative.
///
/// # Assumptions
///
/// * It is an in-sample check and cannot be anything else on this record — fails when 0.9509 is read as out-of-sample validation. The percentiles in sw_uncertainty_growth were measured on these same pairs, so a coverage near 0.95 is close to arithmetic rather than evidence — it confirms the percentile was computed correctly, not that it will hold. What makes the number worth publishing is that it could have come back wrong: an off-by-one in the lead, a percentile taken on the absolute change rather than the signed one, or a table read at the wrong index would all show here. It is a check on the implementation, and it is honest about being only that
/// * The coverage is uniform across leads, which is the part that is evidence — fails when the pooled figure hides a bad lead. It does not: per-lead coverage runs from 0.949970 at 365 and 1826 days to 0.952425 at 2557 days, a spread of 0.0025 across seventeen leads spanning half a year to fifteen years. A table that was right at short leads and wrong at long ones would show as a drift and there is none. The pooled 0.9509 is therefore a fair summary rather than an average over disagreeing parts
/// * Signed, not absolute — this counts only the band being exceeded UPWARD — fails when a two-sided band is wanted. The growth percentile is the 95th of the SIGNED change, so this row asks how often F10.7 rose by more than the stated amount, and a large fall counts as inside the band. For a drag design that is the right test, because the unsafe direction is flux arriving higher than planned. A mission exposed to F10.7 being LOWER than planned — a power budget, for instance — is not checked by this row at all
/// * Pairs are massively overlapping, so the sample is far smaller than 128135 — fails when the count is read as independent evidence. Consecutive pairs at a given lead share all but one day, and at a lead of fifteen years two pairs a day apart are nearly the same measurement. The effective sample is closer to the number of independent intervals — the record divided by the lead, which at the longest leads is five or six — than to the 4838 to 9947 pairs each lead contributes. Nothing here is a confidence interval, and the third decimal place of 0.9509 means nothing
pub const NODE_ID: &str = "sw_band_coverage";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xba92c8446cc7ab59;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.9509267569, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "C_band", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "C_band", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.9 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_band", value: answer.get(), bound: 0.9, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 0.9 a band sold as 95 per cent would be missing the truth twice as often as it claims, and every margin built on sw_f107_design would be smaller than it reads. That is a defect in the percentile table, not a property of the sky, and it should stop a run" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "C_band", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "a coverage cannot exceed 1. A value at exactly 1 would mean the band was never exceeded in 29 years, which for a 95th percentile would mean the table is far too wide and the design is paying for margin it does not need" });
    }
    Ok(answer)
}
