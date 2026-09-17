// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long must one satellite operate?
///
/// `T_mis = T_mis_req`
///
/// Source: `orbitt_case_c1`
pub const NODE_ID: &str = "orbit_mission_duration";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xa4de90cb1b5e53e8;

pub fn evaluate(required: Time) -> Result<Time, Fault> {
    // ---- HOLE 1 : carry the declared mission duration through unchanged -> Time
    let duration: Time = required;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = duration;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_mis", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 15778800.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_mis", value: answer.get(), bound: 15778800.0, edge: Edge::Lower, unit: Time::UNIT, reason: "below six months the programme cannot amortise a satellite, so it is not the mission being designed" });
    }
    if answer.get() > 473364000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_mis", value: answer.get(), bound: 473364000.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above 15 years the cost model, the degradation model and the battery cycle model are all extrapolated well past their fits" });
    }
    Ok(answer)
}
