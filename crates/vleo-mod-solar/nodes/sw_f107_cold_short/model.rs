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
/// daily term is not: 34.23 sfu up against 31.27 down. So the hot day sits
/// further above nominal than the cold day sits below it, by about three sfu,
/// and that asymmetry is the record's shape rather than a rounding artefact.
///
/// # Assumptions
///
/// * The two spreads stack rather than combine — fails when a reader takes the result as a 95 per cent day. It is not: stacking a 10th-percentile rotation level with a 5th-percentile day inside it is nearer a 1-in-100 day than a 1-in-20 one, assuming the two are independent — and they are not independent either, because a quiet rotation is made of quiet days. The study does this and the port reproduces it; the number is conservative and its label is wrong
/// * The cold day is not the hot day mirrored — fails when somebody builds it by negating sw_f107_design_short's daily term. The daily tails differ by nine per cent — 31.27 down against 34.23 up — so mirroring makes this scenario about three sfu colder than the record supports, and a drag-authority case sized on it is being asked to work in air that has never been observed
/// * The daily drop measured before the window applies inside it — fails when the window spans a different part of the cycle from the 2001 days the percentile was measured on. Within-rotation variability widens near maximum, so a drop measured on an active stretch is too deep for a quiet window and too shallow for an active one
/// * A day at this level is a day the rest of the tool can model — fails when the value approaches the guard. Below 60 sfu every relation reading F10.7 in this repository is extrapolating past its own support, so this row refuses rather than handing a number downstream that looks like flux and is not
/// * The construction is valid at the centre this repository actually gives, and AT THE DECLARED WINDOW IT IS NOT — fails when the centre is below 108.5 sfu, which sw_central_expectation's own 86.8497 at the declared epoch is. Stacked on that centre this relation yields 38.3559 sfu and the guard refuses it, because 38 sfu has never been observed and the quiet sun's floor is near 64. The refusal is correct and the arithmetic is what is wrong: a pooled sigma, a normal multiplier on skewed residuals, and a daily drop applied to a rotation already at the low edge of its own band each take too much off, and together they take more than half the sun away. Every one of the three is upstream of this row
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
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_cold_short", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 60 sfu has never been observed and every relation reading F10.7 has no support there. This is the row most likely to reach it — a symmetric rotation band and a daily drop stacked on a low centre — which is exactly why the guard is here rather than downstream" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_cold_short", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value. On the COLDEST of the five scenarios a value there means a sign is wrong somewhere in the chain above it" });
    }
    Ok(answer)
}
