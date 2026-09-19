// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What sustained F10.7 must this design operate in, for as long as the mission lasts?
///
/// `F107_req = 260 sfu, above the record's largest 27-day mean of 252.67`
///
/// Source: `orbitt_case_c1`
///
/// The headline requirement of the five, and the first `required` row written
/// anywhere in this repository — no other subsystem has one, so the shape
/// here is the shape the other sixteen will copy. It is a ceiling the
/// environment must not exceed, and l3_solar_ach_01 is the number the record
/// actually presents. WHAT THIS ROW USED TO ASK, AND WHY IT CHANGED. It asked
/// "what solar flux must this design survive?" with no horizon on it, and
/// l3_solar_req_02 asked the same thing again at 95 per cent confidence —
/// two rows, one question, one number. §20 splits every driver into a level
/// the mission SITS AT for months and a level one day inside it reaches,
/// because a design uses them for different things: an array is sized on the
/// sustained level, a thermal transient on the day. This row is now the
/// sustained half and req_02 is the single day, so the duplication is gone
/// and two different commitments are stated instead of one commitment twice.
///
/// # Assumptions
///
/// * 260 is the record's hottest sustained rotation rounded up, and the SATELLITE-ERA record is not the sun's history — fails when the record is read as the sun's history. 260 is anchored at 252.67 sfu, the largest 27-day mean in 28.2 years of SATELLITE-ERA record, centred 2024-08-13. Cycle 19 in the late 1950s ran higher than anything in that record, so a design meant to operate through a cycle-19 maximum needs a larger number and the rounding to 260 buys seven sfu against it. The anchor is at least checkable, which its predecessor was not: 250 was the next round number above whatever the chain happened to give, so it moved with the design rather than with the sky and its headroom drifted from 9.6 per cent to 140 without anything being wrong.
/// * It is a ceiling on the DRIVER and says nothing about what the driver does to the vehicle — fails when the requirement is read as a survivability statement. F10.7 is an index of solar radio flux; what a spacecraft actually feels is the density that flux produces at its altitude, through a model this subsystem does not own. A design that meets F10.7 <= 250 and is sized on a density model with the wrong drag coefficient has met this requirement and will still deorbit early. The closure this row takes part in is on the environment, and the environment is only the first half.
/// * One number for the whole mission, with no epoch and no phase in it — fails when the mission slips. The record runs 64 to 343 sfu across a cycle, so the flux a mission sees depends on where in the cycle it flies; the declared epoch of 2027-01-01 sits at phase 0.619, past maximum on the declining side. A mission starting in 2030 would sit near minimum and 250 would be enormously conservative. The requirement does not move with the epoch and is not meant to — it is the vehicle's capability, and the sky's variation belongs on the achieved side.
pub const NODE_ID: &str = "l3_solar_req_01";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x133e78d61ed875e4;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(260.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "F107_req_long", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_req_long", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_req_long", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as env_f107: below 60 sfu has never been observed, so a requirement there could never be met and is not a requirement" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_req_long", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as env_f107: above 400 sfu every consumer of F10.7 is extrapolating, and a requirement written past the range its consumers support is not checkable" });
    }
    Ok(answer)
}
