// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far below its own rotation does a single day of F10.7 fall, at the declared confidence, at the level that rotation sits at?
///
/// `dF107_day_low(L) = -pctl(F107 - movmean(F107, 27 d), 0.05) over days whose rotation sits within 15% of L`
///
/// Source: `noaa_swpc`
///
/// The other tail of sw_daily_band_spread's sample, and a separate row
/// because it is a separate number at every level: the departures are not
/// symmetric. The sun has a floor and no ceiling, so a day can rise further
/// above its rotation than it can fall below it, and a design that mirrors
/// one tail onto the other is assuming a shape the record does not have. It
/// is the cold side, which sounds like the side nobody sizes against. It is
/// not: a cold day is the low-drag case, and the low-drag case is what a
/// propellant budget's LOWER bound and an aerodynamic control authority's
/// worst case are made of. The hot day and the cold day both bound something.
///
/// # Assumptions
///
/// * The departure scales with the level, and the old single number was seven times too large at this window — fails when this is read as a refinement. 31.27 subtracted from 69.63 is 38.36 sfu, which has never been observed and which sw_f107_cold_short's guard refused. Conditioned on the level the drop is 4.39. A factor of seven is not a correction to a margin, it is a different answer
/// * The declared window sits at the clamp, so this row is reading the end of its own table — fails when the sustained cold level falls below 70 sfu, as it does here at 69.63. The answer is held at the lowest knot's 4.39 rather than extrapolated, which is the right choice — below 70 the record has few rotations and a line drawn onward would be invention — but it does mean this window's answer is the table's endpoint and not an interpolation. A colder window gets the same 4.39
/// * The departures are asymmetric, and this row exists because of it — fails when it is treated as the negation of sw_daily_band_spread. The gap runs from six per cent at a rotation of 70 to fifteen per cent at 210, because F10.7 has a floor near 65 and no ceiling, and the room below a rotation shrinks as the rotation approaches that floor. Mirroring the high tail onto the low one is asserting a symmetry the record refuses at every level
/// * A 15 per cent bin half-width, which trades resolution against sample size — fails when either matters more than the other. Narrower bins resolve the level dependence better and hold fewer days, so the percentile gets noisier; wider bins are the defect this row exists to fix, in miniature
/// * The 27-day moving mean is the rotation — fails when the solar rotation is 27.27 days at the equator and slower at the poles, and the active longitudes that drive F10.7 are not at one latitude. A 27-day window is the conventional round number rather than a measured period, and the departures it leaves carry whatever the mismatch contributes
/// * It is the record's own daily scatter, not a forecast error — fails when this is read as an uncertainty. It is not: it says how far below its rotation the sun has gone, measured on days that already happened. What a forecast of a future day would get wrong is sw_uncertainty_growth's question and a larger number
pub const NODE_ID: &str = "sw_daily_band_drop";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x2c68c4526ada033f;

pub fn evaluate(level: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::sw_daily_band_drop. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::sw_daily_band_drop::evaluate(level.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "dF107_day_low")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "dF107_day_low", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dF107_day_low", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a drop of zero would mean no day ever falls below its rotation mean, which the record contradicts on about half the days it holds; below zero is not a distance, and a negative value here means the tail has been read from the wrong end" });
    }
    if answer.get() > 120.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "dF107_day_low", value: answer.get(), bound: 120.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 120 sfu the departure exceeds the largest single-day excursion in the record in either direction, so a value there is an arithmetic error rather than a quiet sun" });
    }
    Ok(answer)
}
