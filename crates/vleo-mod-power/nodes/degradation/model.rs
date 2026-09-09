// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much of the array's beginning-of-life output survives to the end of the mission?
///
/// `f_deg = (1 - d_yr)^t`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pwr_degradation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xfb83cb4102b23762;

pub fn evaluate(d: Ratio, t: Time) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : compound the annual rate over the mission duration in years -> Ratio
    let f: Ratio = power::degradation(d, t.get() / 31_557_600.0);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = f;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "f_deg", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "f_deg", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a degradation factor cannot be negative" });
    }
    if answer.get() > 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "f_deg", value: answer.get(), bound: 1.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "an array cannot end life producing more than it began with" });
    }
    Ok(answer)
}
