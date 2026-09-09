// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much does the array produce at the end of the mission?
///
/// `P_eol = P_bol*f_deg*f_T`
///
/// Source: `larson_wertz`
///
/// Every power budget in this tree closes against this number, not against
/// the beginning-of-life one.
pub const NODE_ID: &str = "pwr_array_power_eol";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe22a74ca6e74fb63;

pub fn evaluate(p: Power, f: Ratio, ft: Ratio) -> Result<Power, Fault> {
    // ---- HOLE 1 : apply both the ageing degradation and the temperature derating to the beginning-of-life output -> Power
    let pe: Power = power::array_power_eol(p, f) * ft.get();
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = pe;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "P_eol", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_eol", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "array output cannot be negative" });
    }
    if answer.get() > 50000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_eol", value: answer.get(), bound: 50000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "cannot exceed the beginning-of-life output" });
    }
    Ok(answer)
}
