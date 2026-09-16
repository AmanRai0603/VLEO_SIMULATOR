// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long must the mission operate?
///
/// `T_mis_req = 5`
///
/// Source: `orbitt_case_c1`
///
/// # Assumptions
///
/// * One duration governs the whole programme — every subsystem sizes against the same mission length — fails when a constellation replenishes on a schedule shorter than a satellite's life, so the satellite duration and the service duration are different numbers and this row is the second of them
pub const NODE_ID: &str = "sys_mission_requirements_mission_duration";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xa3694836d5b8ac1d;

pub fn evaluate() -> Result<Time, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Time = match Time::from_unit(5.0, Unit::Year) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "T_mis_req", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_mis_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 15778800.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_mis_req", value: answer.get(), bound: 15778800.0, edge: Edge::Lower, unit: Time::UNIT, reason: "below six months the programme cannot amortise a satellite, so it is not the mission being designed" });
    }
    if answer.get() > 473364000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_mis_req", value: answer.get(), bound: 473364000.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above 15 years the cost model, the degradation model and the battery cycle model are all extrapolated well past their fits" });
    }
    Ok(answer)
}
