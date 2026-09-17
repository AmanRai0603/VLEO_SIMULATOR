// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How cold can a single day of F10.7 be inside the mission window?
///
/// `F107_cold_short = F107_cold_long - dF107_day_low`
///
/// Source: `noaa_swpc`
///
/// The coldest of the five scenarios, and the one a drag-based capability is
/// weakest at. Its hot twin, sw_f107_design_short, is the worst single day
/// upward; this is the worst single day downward, and the two are NOT mirror
/// images even though the rows look identical bar a sign. They are not
/// mirrors because the two terms behave differently. The rotation band is
/// symmetric — both edges are 1.28 standard deviations from the centre. The
/// daily term is not, at any level: at a rotation of 70 sfu the tails are
/// 4.63 up and 4.39 down, at 145 they are 34.61 and 32.39, at 210 they are
/// 46.93 and 40.78. And the two days are not even read off the same point of
/// the table, because the hot day rides on the hot mean and the cold day on
/// the cold mean. At this window that is 20.07 added at 104.07 against 4.39
/// taken off at 69.63.
///
/// # Assumptions
///
/// * The two spreads stack rather than combine — fails when a reader takes the result as a 95 per cent day. It is not: stacking a 10th-percentile rotation level with a 5th-percentile day inside it is nearer a 1-in-100 day than a 1-in-20 one, assuming the two are independent — and they are not independent either, because a quiet rotation is made of quiet days. The study does this and the port reproduces it; the number is conservative and its label is wrong
/// * The cold day is not the hot day mirrored — fails when somebody builds it by negating sw_f107_design_short's daily term. Two things make that wrong and only one is the tail asymmetry. The tails differ by six to fifteen per cent depending on level; and the two rows read the table at DIFFERENT LEVELS, 69.63 here against 104.07 there, so the terms are 4.39 and 20.07 — a factor of four and a half. Mirroring would put this scenario nearly sixteen sfu below what the record supports
/// * The daily drop measured before the window applies inside it — fails when the LEVEL is now carried but the PHASE is not. sw_daily_band_drop conditions on the rotation level, which removes the error that made this row refuse; it does not condition on where in the cycle that level occurs. A rotation at 100 sfu on a rising cycle and one at 100 sfu on a declining cycle get the same drop, and the record does not say they should
/// * A day at this level is a day the rest of the tool can model — fails when the value approaches the guard. Below 60 sfu every relation reading F10.7 in this repository is extrapolating past its own support, so this row refuses rather than handing a number downstream that looks like flux and is not
/// * THIS ROW REFUSED UNTIL THE DAILY DROP WAS CONDITIONED ON LEVEL, and the history is worth keeping — fails when a reader assumes the construction was always sound. It was not. With a single un-conditioned drop of 31.2722 this relation gave 38.3559 sfu at the declared window and the guard refused it, because 38 sfu has never been observed and the quiet sun's floor is near 64. A drop measured over rotations averaging 136.63 sfu does not apply at a rotation of 69.63; conditioned on level it is 4.3870 and the answer is 65.2411. The guard was what caught it, so a design that quietly widened a floor to get a number would have shipped a sun that cannot exist
pub const NODE_ID: &str = "sw_f107_cold_short";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x0158aca5b8a86e32;

pub fn evaluate(sustained: Ratio, daily: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : subtract the within-rotation daily drop from the sustained cold level -> Ratio
    // The daily term is a MAGNITUDE, so the sign lives here in the relation
    // rather than in sw_daily_band_drop's value. And it is the low tail's own
    // number, 31.27, not the high tail's 34.23 negated: the two differ by nine
    // per cent because the departures are skewed.
    let single_day: Ratio = sustained - daily;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = single_day;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_cold_short", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_cold_short", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 60 sfu has never been observed and every relation reading F10.7 has no support there. This is the row most likely to reach it — a symmetric rotation band and a daily drop stacked on a low centre — and it DID reach it, until sw_daily_band_drop was conditioned on the rotation level. The guard is here rather than downstream because this is where the impossible number is formed" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_cold_short", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value. On the COLDEST of the five scenarios a value there means a sign is wrong somewhere in the chain above it" });
    }
    Ok(answer)
}
