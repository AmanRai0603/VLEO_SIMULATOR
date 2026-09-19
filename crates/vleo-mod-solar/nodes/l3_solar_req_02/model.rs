// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What F10.7 must this design survive on a single day inside the mission window?
///
/// `F107_req_day = 350 sfu, above the record's largest daily value of 343`
///
/// Source: `orbitt_case_c1`
///
/// Survive, not operate in. A day at this level may cost a safe mode, a
/// slewed array and some propellant; what it may not cost is the vehicle.
/// That is a weaker claim than l3_solar_req_01 makes about continuous
/// operation, which is why its number is higher — a transient is easier to
/// survive than a permanent condition, and a design whose two numbers were
/// equal would not have been thought about. The checking partner is
/// l3_solar_ach_02, which restates sw_f107_design_short: the sustained hot
/// level with the within-rotation daily excursion on top.
///
/// # Assumptions
///
/// * 350 is anchored in the record's largest observed day, and a longer record would move it — fails when a day above 343 sfu is observed. The record is 28.2 years and covers two and a half cycles; cycle 19 in the late 1950s ran higher than anything in it, with F10.7 reported above 350. So this requirement is anchored in the SATELLITE-ERA record rather than in the observed history of the sun, and a design meant to survive a cycle-19 maximum needs a larger number. The rounding to 350 buys seven sfu against that, which is not much
/// * A single day and a sustained level are different commitments, and this is the single day — fails when somebody compares the achieved sustained flux against this row, or the achieved day against l3_solar_req_01. The two closures are 138.30 against 350 and 104.07 against 250, and crossing them reports a margin that belongs to neither
/// * It is a ceiling on the DRIVER and says nothing about what the driver does to the vehicle — fails when the requirement is read as a survivability statement. F10.7 is an index of solar radio flux; what a spacecraft actually feels is the density that flux produces at its altitude, through a model this subsystem does not own. A design that meets F10.7 <= 350 and is sized on a density model with the wrong drag coefficient has met this requirement and will still deorbit early
/// * One number for the whole mission, with no epoch and no phase in it — fails when the mission slips. The requirement does not move with the epoch and is not meant to — it is the vehicle's capability, and the sky's variation belongs on the achieved side of the closure where it can be seen
pub const NODE_ID: &str = "l3_solar_req_02";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x8195b3d4c2f07658;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(350.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "F107_req_short", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_req_short", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_req_short", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as env_f107: below 60 sfu has never been observed, so a requirement there could never be met and is not a requirement" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_req_short", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as env_f107: above 400 sfu every consumer of F10.7 is extrapolating, and a requirement written past the range its consumers support is not checkable" });
    }
    Ok(answer)
}
