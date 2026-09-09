// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much power does the whole spacecraft need?
///
/// `P_dem = (P_prop + P_pay + P_av + P_com + P_th)*(1 + L_h)`
///
/// Source: `larson_wertz`
///
/// Written as one function with named arguments rather than a chain of
/// additions, so a forgotten term is a missing argument the compiler names.
pub const NODE_ID: &str = "pwr_demand";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc6b354710ef121b4;

pub fn evaluate(pp: Power, pay: Power, av: Power, com: Power, th: Power, lh: Ratio) -> Result<Power, Fault> {
    // ---- HOLE 1 : sum the named loads and inflate them by the harness loss -> Power
    let p: Power = power::power_demand(pp, pay, av, com, th, lh);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = p;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "P_dem", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_dem", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "demand cannot be negative" });
    }
    if answer.get() > 50000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_dem", value: answer.get(), bound: 50000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 50 kW nothing in this mass class supplies it" });
    }
    Ok(answer)
}
