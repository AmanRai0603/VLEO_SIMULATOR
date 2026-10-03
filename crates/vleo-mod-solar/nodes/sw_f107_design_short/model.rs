// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What F10.7 must the design survive on a single day inside the mission window?
///
/// `F107_short = F107_long + dF107_day`
///
/// Source: `noaa_swpc`
///
/// The SHORT TERM of the pair. sw_f107_design_long says what the mission sits
/// at for months; this says what one day in twenty reaches while sitting
/// there. A thermal case and a drag transient are sized on this one, an array
/// and a propellant budget on its long sibling, and giving a design only one
/// of the two decides for the reader which problem they have.
///
/// # Assumptions
///
/// * The two spreads stack, and stacking two percentiles is not a percentile — fails when the published number is read as the 95th percentile of a day. It is not: it is the 90th percentile of the rotation level plus the 95th percentile of the daily departure, which for independent normals lands near the 99th. The bound is conservative, the label is not, and the honest statistic would be the percentile of the daily value itself rather than a sum of two
/// * The rotation error and the daily departure are independent — fails when they are not. Both widen with activity, so a window the pattern gets wrong on the high side is also a window whose days scatter most, and the true joint tail is fatter than the sum of two marginals suggests in one direction and thinner in the other
/// * One day in twenty is the day worth designing to — fails when the mission is long. Over a 365-day window a one-in-twenty day happens about eighteen times, so this is not a rare event but a routine one; the rare day a design might actually care about is further out and this row does not publish it
pub const NODE_ID: &str = "sw_f107_design_short";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x6eb82822f28ace12;

pub fn evaluate(sustained: Ratio, daily: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::sw_f107_design_short. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::sw_f107_design_short::evaluate(sustained.get(), daily.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "F107_short")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_short", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_short", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 60 sfu has never been observed and every relation reading F10.7 has no support there; a single-day design level below it means a spread has been subtracted rather than added" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_short", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value — and this row, being the sustained level plus a daily excursion, is the one most likely to reach it, which is exactly why the guard is here" });
    }
    Ok(answer)
}
