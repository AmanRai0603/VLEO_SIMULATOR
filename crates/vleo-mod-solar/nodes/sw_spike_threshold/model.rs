// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far above its own 81-day mean must F10.7 rise to count as a spike?
///
/// `spike when F107 / F107A >= mean + 2.5*sd = 1.3049`
///
/// Source: `noaa_swpc`
///
/// A spike is a day the smooth cycle does not explain. F10.7 is measured
/// against its own 81-day centred mean rather than against a fixed level,
/// because 150 sfu is an ordinary day at solar maximum and an extraordinary
/// one at minimum. The threshold is the record's own scatter, not a number
/// picked to look round.
///
/// # Assumptions
///
/// * Two and a half standard deviations, and that multiplier is the whole definition — fails when somebody wants a spike count rather than a threshold. Over 10284 days the ratio F10.7 to its 81-day centred mean has mean 0.999340 and standard deviation 0.122210, so 2.5 sd lands at 1.304866 and catches 202 days — 1.96 per cent of the record, 6.97 a year. Moving the multiplier to 2.0 would catch about 6 per cent and to 3.0 about 1 per cent. Nothing in the physics picks 2.5; it is chosen because it sits near the 98th percentile, which is where a day stops being the tail of ordinary variation and starts being an event, and because it is a round number of standard deviations rather than a round number of per cent
/// * The ratio is very nearly centred on one, which is what makes a symmetric threshold legitimate — fails when the 81-day window is not centred, or is too short to be a baseline. Measured, the mean ratio is 0.999340 — six parts in ten thousand below one — so the centred mean is an unbiased baseline for the day at its centre. A trailing 81-day mean would sit below the current day during a rise and the same multiplier would then catch rises and miss falls. This row's baseline is CENTRED, which means it cannot be computed in real time: it needs 40 days of future. It is a descriptive threshold for a record, not an operational trigger
/// * It says nothing about the dwell, only the level — fails when a design needs to know how long the sky stays there. A threshold is an instant test and 202 days of the record pass it; they arrive in 61 separate events with a mean length of 3.31 days, which is sw_event_duration's business. The two rows are a pair and neither is usable alone: a threshold with no duration sizes nothing, and a duration with no threshold is not defined
/// * The 273 absent days cannot spike, and nine months is long enough to matter — fails when the count is read as complete. observed_daily.csv is missing 2017-01-01 to 2017-09-30 entirely, so any spike in those nine months is absent from both the 202 and the 61. 2017 sat on cycle 24's decline where F10.7 was low and the ratio's own scatter was small, so the loss is probably small — but it is a loss, and the per-year figures are over a nominal 29 years that is really 28.25
pub const NODE_ID: &str = "sw_spike_threshold";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xcc5b0f192e0cd31c;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(1.3048655732, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "S_thr", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "S_thr", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1.1 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "S_thr", value: answer.get(), bound: 1.1, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 1.1 the threshold would sit inside one standard deviation of the ratio and catch roughly a fifth of all days. A fifth of the record is not a set of events, it is the weather, and every relation downstream that treats a spike as exceptional would be wrong" });
    }
    if answer.get() > 1.6 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "S_thr", value: answer.get(), bound: 1.6, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 1.6 nothing is caught: the largest ratio in 29 years is 2.109781 and only 30 days exceed 1.5. A threshold there would make the spike rows describe a handful of days and the event duration would be a statistic of three or four bursts" });
    }
    Ok(answer)
}
