// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What solar flux must this design survive?
///
/// `F107_req = 250 sfu`
///
/// Source: `orbitt_case_c1`
///
/// The headline requirement of the three, and the first `required` row
/// written anywhere in this repository — no other subsystem has one, so the
/// shape here is the shape the other sixteen will copy. It is a ceiling the
/// environment must not exceed, and l3_solar_ach_01 is the number the record
/// actually presents.
///
/// # Assumptions
///
/// * 250 is the design value with a round margin on it, and the margin is a decision rather than a calculation — fails when somebody looks for the derivation. At the declared five-year mission sw_f107_design comes to 228.14 sfu — the central expectation of 114.84 plus the 95th-percentile growth of 113.29 — and 250 is the next round number above it, 9.6 per cent of headroom. Nothing in the record picks 250. It is chosen so the closure passes with visible room rather than by a hair, and so that a modest change in mission length or epoch does not silently break it. A design that wants the margin to mean something specific should replace this with a number that has a derivation.
/// * It is a ceiling on the DRIVER and says nothing about what the driver does to the vehicle — fails when the requirement is read as a survivability statement. F10.7 is an index of solar radio flux; what a spacecraft actually feels is the density that flux produces at its altitude, through a model this subsystem does not own. A design that meets F10.7 <= 250 and is sized on a density model with the wrong drag coefficient has met this requirement and will still deorbit early. The closure this row takes part in is on the environment, and the environment is only the first half.
/// * One number for the whole mission, with no epoch and no phase in it — fails when the mission slips. The record runs 64 to 343 sfu across a cycle, so the flux a mission sees depends on where in the cycle it flies; the declared epoch of 2027-01-01 sits at phase 0.619, past maximum on the declining side. A mission starting in 2030 would sit near minimum and 250 would be enormously conservative. The requirement does not move with the epoch and is not meant to — it is the vehicle's capability, and the sky's variation belongs on the achieved side.
pub const NODE_ID: &str = "l3_solar_req_01";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x97ebce8d28fd300b;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(250.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "F107_req", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_req", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_req", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as env_f107: below 60 sfu has never been observed, so a requirement there could never be met and is not a requirement" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_req", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as env_f107: above 400 sfu every consumer of F10.7 is extrapolating, and a requirement written past the range its consumers support is not checkable" });
    }
    Ok(answer)
}
