// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the whole power subsystem weigh?
///
/// `m = m_arr + m_batt + m_pcdu`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "pwr_subsystem_mass";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc3a7f44f93fc55c9;

pub fn evaluate(ma: Mass, mb: Mass) -> Result<Mass, Fault> {
    // ---- HOLE 1 : sum the array and battery masses and add 25% for the distribution and control unit -> Mass
    let m: Mass = Mass::new((ma.get() + mb.get()) * 1.25);
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Mass = m;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "m_pwr", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_pwr", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Mass::UNIT, reason: "a mass cannot be negative" });
    }
    if answer.get() > 2000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_pwr", value: answer.get(), bound: 2000.0, edge: Edge::Upper, unit: Mass::UNIT, reason: "above two tonnes the power system is not part of this spacecraft" });
    }
    Ok(answer)
}
