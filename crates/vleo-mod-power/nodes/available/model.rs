// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much power is actually available to the loads, averaged over an orbit?
///
/// `P_avail = P_eol*(1 - f_ecl)*eta_direct + P_eol*f_ecl*eta_batt`
///
/// Source: `larson_wertz`
///
/// # Assumptions
///
/// * The array is sized so that the sunlit period both runs the loads and recharges the battery — fails when if it is not, the battery state of charge walks down over successive orbits and this number is optimistic from the second day onwards
pub const NODE_ID: &str = "pwr_available";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xe92e10c367773f38;

pub fn evaluate(pe: Power, fe: Ratio, ed: Ratio) -> Result<Power, Fault> {
    // ---- HOLE 1 : average the direct and battery-borne paths over the sunlit and eclipsed parts of the orbit -> Power
    let p: Power = Power::new(pe.get() * ((1.0 - fe.get()) * 0.97 + fe.get() * ed.get() * 0.95));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = p;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "P_avail", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_avail", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "available power cannot be negative" });
    }
    if answer.get() > 50000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_avail", value: answer.get(), bound: 50000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 50 kW the array is not the one described by these inputs" });
    }
    Ok(answer)
}
