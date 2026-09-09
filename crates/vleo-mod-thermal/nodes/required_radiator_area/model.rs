// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much radiator would be needed to hold the temperature limit?
///
/// `A = Q/(eps*sigma*(T^4 - T_sink^4))`
///
/// Source: `larson_wertz`
///
/// The inverse question, and the one a thermal design actually asks.
pub const NODE_ID: &str = "thm_required_radiator_area";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x79de83d4426eea8c;

pub fn evaluate(q: Power, e: Ratio, lim: Temperature) -> Result<Area, Fault> {
    // ---- HOLE 1 : solve the grey-body balance for area against a 250 K effective sink -> Area
    let a: Area = thermal::required_radiator_area(q, e, lim, Temperature::new(250.0));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Area = a;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "A_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_req", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Area::UNIT, reason: "a required area cannot be negative" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "A_req", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: Area::UNIT, reason: "above 1000 m2 the design does not close and the answer is that the dissipation is wrong" });
    }
    Ok(answer)
}
