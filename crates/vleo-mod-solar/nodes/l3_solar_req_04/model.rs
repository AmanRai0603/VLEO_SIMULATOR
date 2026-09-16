// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What sustained Ap must this design operate in, for as long as the mission lasts?
///
/// `Ap_req_long = ap(G1) = 48`
///
/// Source: `orbitt_case_c1`
///
/// Not survive — OPERATE IN. A sustained level is the sky the mission sits
/// in for months, so the commitment is about continuous service rather than
/// about coming through intact. That is a stronger claim than its single-day
/// sibling makes and is why its number is lower. The checking partner is
/// l3_solar_ach_04, which reports the sustained Ap the record says the window
/// will present.
///
/// # Assumptions
///
/// * It is a ceiling on the driver and says nothing about what the driver does to the vehicle — fails when a reader takes a passing closure as evidence the design is adequate. Ap is an index; what a spacecraft feels is the heating, the density and the torque it produces, through models this subsystem does not own. A design that meets Ap <= 48 and is sized with the wrong drag coefficient has met this requirement and will still deorbit early
/// * The G1 threshold is a reasonable place for a CONTINUOUS-operation commitment, and nothing here establishes that — fails when the vehicle's real limit is elsewhere. 48 is a published threshold rather than an arbitrary round number, which makes it checkable, but checkable is not the same as correct: the level at which a particular design must stop operating comes from its thermal, its torque authority and its propellant, none of which this subsystem sees. What this row guarantees is that a reader can see what the commitment IS
/// * A sustained level and a single day are different commitments, and this is the sustained one — fails when somebody compares the achieved single-day Ap against this row. That is l3_solar_req_05's closure and its ceiling is 80. Reading the wrong one of the two reports a failure where there is none, or a pass where there is not
pub const NODE_ID: &str = "l3_solar_req_04";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x9f7df30a6d11877e;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(48.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_req_long", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_req_long", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_req_long", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero, and a sustained requirement of zero would commit the design to operating only in a perfectly quiet field, which no mission window presents" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_req_long", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the top of the published ap table, the value at Kp 9. A requirement above it is off the scale the G levels are defined on and could not be expressed as a G level at all" });
    }
    Ok(answer)
}
