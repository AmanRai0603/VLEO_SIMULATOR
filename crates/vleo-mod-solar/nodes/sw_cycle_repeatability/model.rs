// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much does one solar cycle repeat the shape of the last one?
///
/// `r_cyc = corr(F107 of cycle 23 by phase, F107 of cycle 24 by phase) = 0.7701`
///
/// Source: `noaa_swpc`
///
/// The correlation between cycles 23 and 24 after stacking both on phase. It
/// is the credibility of sw_mean_cycle_level: that row hands a design a
/// mean-cycle curve, and this row says how much a single cycle can be
/// expected to look like it. The answer is that the SHAPE repeats and the
/// AMPLITUDE does not, and a design that reads only the correlation will miss
/// the second half.
///
/// # Assumptions
///
/// * It is one pair of cycles, so there is no distribution behind this number — fails when 0.77 is read as an expected repeatability with an uncertainty. The record holds exactly two complete cycles, 23 and 24, and cycle 25 is still running, so this is a single observation of a correlation and not an estimate of one. Two samples cannot tell a repeatable shape from a coincidence between two particular cycles, and nothing in this bundle can fix that: it needs a longer record. Every consumer of sw_mean_cycle_level inherits this limit
/// * The shape repeats at 0.77 and the amplitude does not repeat at all, and these are separate findings — fails when the correlation is taken as the whole answer. A correlation is scale-free, so it is blind to exactly the thing a drag design cares about. Cycle 23's binned peak is 206.9 sfu and cycle 24's is 149.4, a ratio of 0.722 — the published smoothed peaks, 196.4 and 146.1, agree at 0.744 — and the rms difference between the two phase-stacked curves is 37.7 sfu against a pooled mean near 112. So the two cycles rise and fall alike and are not the same size. A mission sized on the mean of the two is sized for neither
/// * Two of the twenty phase bins are excluded, because 273 consecutive days are absent from the record — fails when the bin count is assumed complete. observed_daily.csv carries 10319 rows across a 10592-day span and the whole shortfall is one contiguous gap, 2017-01-01 to 2017-09-30. That gap sits at cycle 24 phase 0.735 to 0.803, which empties bin 15 entirely and leaves bin 14 with 141 days against the usual 201. The correlation is therefore taken over the 18 bins both cycles populate. The gap is on the declining side where F10.7 is low and slowly varying, so its effect on a shape correlation is small; it is excluded rather than interpolated because interpolating across nine months invents the data
/// * Twenty equal-width phase bins, and cycle length is taken from the published boundaries — fails when the boundaries move. Cycle 23 spans 4338 days and cycle 24 spans 4017, an 8 per cent difference, so an equal-PHASE bin is a different number of DAYS in each cycle — 217 against 201. Stacking on phase rather than on days since minimum is the choice that lets two unequal cycles be compared at all, and it means this row says nothing about whether the two cycles took the same TIME to do the same thing. They did not
pub const NODE_ID: &str = "sw_cycle_repeatability";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x637bf7d85cdbb0b9;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.7701397224, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "r_cyc", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "r_cyc", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.4 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "r_cyc", value: answer.get(), bound: 0.4, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 0.4 the two stacked cycles would share less than 16 per cent of their variance, which would say the solar cycle has no repeatable shape at all. The measured value is 0.770 for 59 per cent shared, and the asymmetric fast rise and slow decline is present in both cycles, so a value that low means the stacking or the boundaries are wrong rather than the Sun being irregular" });
    }
    if answer.get() > 0.95 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "r_cyc", value: answer.get(), bound: 0.95, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 0.95 the two cycles would be near-identical in shape, which the record contradicts: the rms difference between the two phase-stacked curves is 37.7 sfu on a pooled mean near 112, and their peaks differ by 28 per cent. An answer that high means the correlation was taken over too few bins or after a normalisation that removed the disagreement" });
    }
    Ok(answer)
}
