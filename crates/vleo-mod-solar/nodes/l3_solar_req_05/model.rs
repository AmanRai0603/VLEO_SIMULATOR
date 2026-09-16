// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What Ap must this design survive on a single day inside the mission window?
///
/// `Ap_req_short = ap(G2) = 80`
///
/// Source: `orbitt_case_c1`
///
/// Survive, not operate in. A day at this level may cost a safe mode, a
/// missed downlink and some propellant; what it may not cost is the vehicle.
/// That is a weaker claim than l3_solar_req_04 makes about continuous
/// operation, which is why its number is higher. Its checking partner is
/// l3_solar_ach_05. Above it sits l3_solar_req_03, which is a third question
/// again: the one storm in the whole mission, not the worst day of an
/// ordinary design band.
///
/// # Assumptions
///
/// * It is a ceiling on the driver and says nothing about what the driver does to the vehicle — fails when a reader takes a passing closure as evidence the design is adequate. Ap is an index; what a spacecraft feels is the heating, the density and the torque it produces, through models this subsystem does not own
/// * The worst day of a design band and the one storm of a mission are different questions — fails when somebody collapses this row into l3_solar_req_03. At the declared window the two achieved sides differ by a factor of nearly four, 41.70 against 158.38, because one is a percentile of ordinary variation and the other an extreme-value return level. A single ceiling covering both would have to be the storm one, and the design would then be claiming to operate through a G3 storm
/// * The G2 threshold is a reasonable place for a one-day survival commitment, and nothing here establishes that — fails when the vehicle's real limit is elsewhere. 80 is a published threshold rather than an arbitrary round number, which makes it checkable, but checkable is not the same as correct: the level at which a particular design must stop is a thermal, torque and propellant question this subsystem does not see
pub const NODE_ID: &str = "l3_solar_req_05";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x9250776791235b4d;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(80.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_req_short", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_req_short", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_req_short", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero, and a single-day requirement of zero would commit the design to surviving only a perfectly quiet day, which is not a survival requirement" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_req_short", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the top of the published ap table, the value at Kp 9. A requirement above it is off the scale the G levels are defined on and could not be expressed as a G level at all" });
    }
    Ok(answer)
}
