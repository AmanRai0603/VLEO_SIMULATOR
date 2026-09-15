// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far does a single day's F10.7 stray from its own 81-day centred mean?
///
/// `sd of F107 / F107A over the record = 0.122210`
///
/// Source: `noaa_swpc`
///
/// The scatter of the daily driver about the smoothed one, as a ratio so it
/// applies at any activity level. It is what sw_f107_81day cannot tell you
/// and what a density model driven by a smoothed flux alone will miss.
///
/// # Assumptions
///
/// * The ratio is centred on one to six parts in ten thousand, and that is a check rather than a coincidence — fails when the 81-day window were not centred, or the record were trending within it. Measured over 10284 days the mean ratio is 0.999340. A centred mean is an unbiased estimate of the day at its centre, so a mean ratio at one is what a correct computation must produce, and a departure would have meant the window was trailing or misaligned. The 0.00066 shortfall is the record's own asymmetry — flux spikes up and decays down — surviving the average
/// * It is a standard deviation of a distribution that is not normal — fails when 0.1222 is used to build a symmetric interval. The ratio runs to 2.11 at the top, which is nine standard deviations above the mean, and cannot go below zero at all: the distribution is bounded on one side and has a long tail on the other. Two standard deviations does not mean 95 per cent here. The measured percentiles are 1.2188 at the 95th and 1.3711 at the 99th, which are the numbers to use for a band
/// * Pooled across the whole cycle, and the scatter is not constant across it — fails when a design at a known cycle phase wants the scatter it will actually see. Active regions produce the departures, so the ratio's spread is larger near maximum than near minimum, and this row averages a quiet 2008 with a busy 2002 into one number. The mean-cycle level at the epoch's phase is 108.14 sfu, so a twelve per cent scatter there is about 13 sfu — but that is the pooled twelve per cent applied at one phase, not a phase-conditioned measurement
/// * The 273 absent days are not in it — fails when the count is read as the whole record. The ratio is defined on 10284 of the 10592 calendar days the record spans: 273 are the 2017 gap, and the rest are days too near an end of the record for a full 81-day centred window to exist. No value is interpolated across the gap — a day whose window overlaps it uses the days that are there, and a day with fewer than 57 of its 81 is left undefined rather than computed from a short window
pub const NODE_ID: &str = "sw_f107a_ratio";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x9e453c3bf010368c;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(0.1222100376, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "sd_ratio", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "sd_ratio", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.05 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sd_ratio", value: answer.get(), bound: 0.05, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 0.05 the daily flux would sit within five per cent of its 81-day mean on a typical day, which would make the daily driver and the smoothed one interchangeable. The record's 99th percentile ratio is 1.371, so it is not" });
    }
    if answer.get() > 0.25 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sd_ratio", value: answer.get(), bound: 0.25, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 0.25 the typical day would be a quarter away from its own baseline and the 81-day mean would not be describing the same quantity as the day. A value there means the window or the pairing is wrong rather than that the Sun is variable" });
    }
    Ok(answer)
}
