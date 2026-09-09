// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much power does the propulsion system take from the bus?
///
/// `P_prop = P_in/eta_ppu`
///
/// Source: `larson_wertz`
///
/// The largest single load on the bus, and the reason the power chapter is
/// where an air-breathing design usually fails.
pub const NODE_ID: &str = "prop_bus_power";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x5793a8c1487725ae;

pub fn evaluate(pin: Power, eta: Ratio) -> Result<Power, Fault> {
    // ---- HOLE 1 : divide by the power processing unit efficiency -> Power
    let p: Power = prop::bus_power_demand(pin, eta);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Power = p;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "P_prop", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_prop", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Power::UNIT, reason: "bus demand cannot be negative" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "P_prop", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Power::UNIT, reason: "above 100 kW no bus in this mass class supplies it" });
    }
    Ok(answer)
}
