// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the deployed array weigh?
///
/// `m = A*sigma_a`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pwr_array_mass";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xfbc1c4f746a49e94;

pub fn evaluate(a: Area, s: Ratio) -> Result<Mass, Fault> {
    // ---- HOLE 1 : multiply the deployed area by the areal density -> Mass
    let m: Mass = power::array_mass(a, s.get());
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Mass = m;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "m_arr", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_arr", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Mass::UNIT, reason: "a mass cannot be negative" });
    }
    if answer.get() > 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_arr", value: answer.get(), bound: 1000.0, edge: Edge::Upper, unit: Mass::UNIT, reason: "above a tonne the array is not part of this spacecraft" });
    }
    Ok(answer)
}
