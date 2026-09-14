// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Quiet, active or storm — which regime is this daily Ap in?
///
/// `regime(Ap) = 1 if Ap <= 6, 2 if Ap <= 25, 3 otherwise`
///
/// Source: `noaa_swpc`
///
/// 1 is quiet, 2 is active, 3 is storm, in the order the study's own mixture
/// put them. A number rather than a name because the interface carries
/// numbers; the bands are integers and the boundaries are exact, so the
/// encoding loses nothing.
///
/// # Assumptions
///
/// * It departs from the published table on 171 of 10299 days, deliberately — fails when parity with the study is what is wanted. Every one of the 171 is a day the mixture labelled 'storm' with an Ap of 0 or 1 and a Kp_max of 0 or 1, in 2006 to 2011. This row calls them quiet. On the other 10128 days it reproduces the published label exactly — the disagreement count against the corrected labels is zero, not small. Anyone reconciling against daily_regime.csv will find exactly these days and no others, and the sign of the disagreement is always the same: the table says storm where this row says quiet, never the reverse
/// * The only Ap this tree publishes is a design storm, so the quiet band is unreachable today — fails when somebody expects to see all three answers in a run. The input is sw_storm_return_level, whose declared range is 20 to 230, so in any run this node can only return active or storm — and at the declared five-year mission it returns 3. The relation carries all three branches because the classifier has three, and the day an ordinary Ap exists in the tree this row already handles it. That no such row exists is a fact about the tree and is worth noticing rather than working around
/// * Three bands on one variable, which is all the study's mixture turned out to be — fails when a regime is expected to depend on anything but today's Ap. It does not: no history, no rate of change, no F10.7. A day at Ap 26 on the way up and a day at Ap 26 on the way down are the same regime here, and a storm's recovery day is labelled by its level rather than by what it is recovering from. That is the study's model and this row is a port of it, not an improvement on it
/// * The boundaries are exact integers and the published confidences are discarded — fails when a caller wants to know how sure the label is. daily_regime.csv carries a posterior per day — mean 0.550 for quiet, 0.580 for active, 0.705 for storm, with 21 per cent of quiet days below 0.5 — and this row publishes one number with no confidence beside it. The bands being exact makes the LABEL certain given the Ap; it does not make the label right. A day at Ap 26 is one unit from being called active and the study was only 55 to 70 per cent sure of its labels on average
pub const NODE_ID: &str = "sw_regime";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x40f82069e2f0c2af;

pub fn evaluate(ap: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : place the daily Ap in the mixture's three bands: quiet to 6, active to 25, storm above -> Ratio
    // The mixture's own decision boundaries, read off its published labels:
    // quiet holds Ap 0 to 6, active 7 to 25, storm 26 and above. Those three
    // bands reproduce daily_regime.csv with ZERO disagreements on 10128 of its
    // 10299 days; the 171 that differ are the low-tail artefact the sheet
    // declares, where the broad storm component took back the record's quietest
    // days. Integers, because the published boundaries fell on integers.
    let a: f64 = ap.get();
    let out: Ratio = Ratio::new(if a <= 6.0 {
        1.0
    } else if a <= 25.0 {
        2.0
    } else {
        3.0
    });
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = out;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "regime", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "regime", value: answer.get(), bound: 1.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "1 is quiet and is the lowest regime the mixture defines. Below it there is no label" });
    }
    if answer.get() > 3.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "regime", value: answer.get(), bound: 3.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "3 is storm and is the highest. A fourth regime would be a different classifier than the one this row ports" });
    }
    Ok(answer)
}
