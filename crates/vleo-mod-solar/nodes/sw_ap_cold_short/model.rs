// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How quiet can a single day of Ap be inside the mission window?
///
/// `Ap_cold_short = Ap_cold_long - dAp_day_low`
///
/// Source: `noaa_swpc`
///
/// The quietest of the five scenarios, and the one that comes nearest a
/// declared bound anywhere in this subsystem: at the declared window it lands
/// at Ap 6.9, seven units above a floor of zero. That closeness is the point
/// of reading this row rather than only its number. Ap is bounded below and
/// unbounded above, its mean band is symmetric anyway because a standard
/// deviation has no sides, and its daily tails are forty-two per cent apart
/// because percentiles of a truncated sample are. Three facts that disagree
/// about shape meet in one subtraction here.
///
/// # Assumptions
///
/// * The result stays above zero, and nothing in the arithmetic ensures it — fails when the centre is small. With this subsystem's own spread and drop the crossing is at a centre near 15.2, and Ap centres that low are ordinary at solar minimum. The declared window's 22.1 clears it by seven units — the narrowest margin to a declared bound anywhere in this subsystem. The guard refuses rather than publishing a negative index, which is right, but it means this row is the one most likely to refuse
/// * The two spreads stack rather than combine — fails when a reader takes the result as a 95 per cent day. Stacking a 10th-percentile rotation level with a 5th-percentile day inside it is nearer a 1-in-100 day than a 1-in-20 one, assuming independence — and a quiet rotation is made of quiet days, so they are not independent. The study does this and the port reproduces it; the number is conservative and its label is wrong
/// * The quiet day is not the disturbed day mirrored — fails when somebody builds it by negating sw_ap_design_short's daily term. The daily tails differ by forty-two per cent — 10.59 down against 15.00 up — so mirroring puts this scenario at Ap 2.5 instead of 6.9, a third of it, and the crossing to zero moves from a centre of 15.2 up to 19.6 — two and a half units below the declared window rather than seven
/// * The daily drop measured before the window applies inside it — fails when the window spans a different part of the cycle. Ap's within-rotation variability peaks in the DECLINING phase, when coronal holes are largest and high-speed streams recur, rather than at maximum. A drop measured on one phase is the wrong depth for another
pub const NODE_ID: &str = "sw_ap_cold_short";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xf41e49bc70785339;

pub fn evaluate(sustained: Ratio, daily: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : subtract the within-rotation daily Ap drop from the sustained cold level -> Ratio
    // The daily term is a MAGNITUDE, so the sign lives here rather than in
    // sw_ap_daily_band_drop's value. It is the low tail's own number, 10.59, not
    // the high tail's 15.00 negated — forty-two per cent apart, because Ap is
    // truncated at zero. The guard below is the one most likely in this
    // subsystem to fire on an ordinary input.
    let single_day: Ratio = sustained - daily;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = single_day;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_cold_short", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_cold_short", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero — a perfectly quiet day is Ap 0 and there is nothing below it. This is the row in the subsystem closest to its own floor, seven units clear at the declared window, and the only one where the guard is likely to fire on an ordinary input rather than on a mistake" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_cold_short", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the top of the Ap index itself; a value above it is not a geomagnetic index at all. On the QUIETEST of the five scenarios a value anywhere near it means a sign is wrong somewhere in the chain above" });
    }
    Ok(answer)
}
