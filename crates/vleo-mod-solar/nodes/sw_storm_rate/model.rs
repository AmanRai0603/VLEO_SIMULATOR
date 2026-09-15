// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How many days a year does the real sky exceed this Kp?
///
/// `rate(Kp) = days per year with Ap > ap(Kp), measured over the record`
///
/// Source: `noaa_swpc`
///
/// The row that tells a reader what a declared Kp costs. env_kp declares Kp =
/// 3 by hand, and this says the record exceeds it on 63.6 days a year —
/// more than two months of every year. It takes a Kp rather than an Ap
/// because the Kp is what the design declares and what the atmosphere model
/// reads.
///
/// # Assumptions
///
/// * Tabulated at the published scale's own 28 values, so the interpolation has almost nowhere to go — fails when the rate falls by four orders of magnitude across the scale — 364 days a year above Kp 0, 0.04 above Kp 8 — so interpolating it linearly between INTEGER Kp would overstate the rate badly in the middle of each interval. Measuring at all 28 published values instead leaves at most one third of a Kp unit between points, which is the granularity the scale itself has. Within that the answer is still a straight line and still this node's choice, not the record's.
/// * The top of the scale reads zero, and zero here means NOT OBSERVED rather than impossible — fails when the record holds no day above Ap 300, so the rate at Kp 8.67 and Kp 9 is measured as 0.0 days a year. That is 28.2 years of evidence, not a statement about the Sun: a Kp 9 day is a real and documented kind of event and this record simply does not contain one. Reading 0.0 as 'cannot happen' would be the worst possible misuse of this row, and a design sized on it would carry no allowance for the largest storms at all. Kp 8 already rests on a single day.
/// * It counts DAYS, not storms, and a storm lasts more than a day — fails when consecutive disturbed days are counted separately, so 63.6 days a year above Kp 3 is not 63.6 storms a year — it is fewer, longer events. sw_event_duration supplies the mean length needed to convert one into the other — 3.31 days above the Kp 3 threshold — so 63.6 days a year is closer to 19 events a year than to 63. Anyone reading this as an event count will overestimate how often the sky is disturbed AND underestimate how long it stays that way.
pub const NODE_ID: &str = "sw_storm_rate";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x277914b0f3490e21;

pub fn evaluate(kp: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : read the measured exceedance rate at this Kp, tabulated at the published scale's own values -> Ratio
    // The exceedance rate at each of the 28 PUBLISHED Kp values, counted on
    // solar-weather@2026.09.14: days whose daily Ap exceeds that Kp's tabulated ap,
    // per year of record. Tabulating at the scale's own resolution leaves at most a
    // third of a Kp unit for the interpolation to be wrong in, which matters because
    // the rate falls four orders of magnitude from end to end.
    //
    // The last two entries are 0.0, and the sheet is emphatic that this means the
    // record contains no such day rather than that no such day occurs.
    use vleo_core::math::Table1;
    const RATE: Table1 = Table1 {
        x: &[
            0.0,
            0.333333,
            0.666667,
            1.0,
            1.333333,
            1.666667,
            2.0,
            2.333333,
            2.666667,
            3.0,
            3.333333,
            3.666667,
            4.0,
            4.333333,
            4.666667,
            5.0,
            5.333333,
            5.666667,
            6.0,
            6.333333,
            6.666667,
            7.0,
            7.333333,
            7.666667,
            8.0,
            8.333333,
            8.666667,
            9.0,
        ],
        y: &[
            364.1506,
            343.8648,
            316.2024,
            275.5245,
            239.1733,
            207.2552,
            180.4794,
            135.8649,
            90.3993,
            63.588,
            45.572,
            30.7478,
            18.6898,
            12.5899,
            7.6958,
            4.3621,
            3.1209,
            1.7378,
            1.2413,
            1.0285,
            0.6738,
            0.2837,
            0.2483,
            0.1419,
            0.0355,
            0.0355,
            0.0,
            0.0,
        ],
    };
    let r: Ratio = Ratio::new(RATE.at(kp.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = r;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "rate", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "rate", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a count of days cannot be negative. Zero is reachable and is the measured value at the top of the scale, which the assumptions say means not observed in 28.2 years rather than impossible" });
    }
    if answer.get() > 366.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "rate", value: answer.get(), bound: 366.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "there are at most 366 days in a year, so a rate above that is a counting error rather than a sky. The largest measured entry is 364.15 days a year above Kp 0, which is every day the record has an Ap at all" });
    }
    Ok(answer)
}
