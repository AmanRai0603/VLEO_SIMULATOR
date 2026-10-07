// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Does the power system supply what the spacecraft asks for?
///
/// `M = (P_avail - P_dem)/P_dem`
///
/// Source: `ecss_e_st_10_02`
///
/// A margin silently corrected to zero is a design that drifted without
/// anyone deciding to. This one is signed.
pub const NODE_ID: &str = "pwr_margin";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb800b4c477686956;

pub fn evaluate(av: Power, dem: Power) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::pwr_margin. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::pwr_margin::evaluate(av.get(), dem.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "M_pwr")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "M_pwr", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_pwr", value: answer.get(), bound: -1.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a margin below -100% would mean the demand is more than twice the supply and the design is not a design" });
    }
    if answer.get() > 100.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "M_pwr", value: answer.get(), bound: 100.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 10000% the array is absurdly oversized" });
    }
    Ok(answer)
}
