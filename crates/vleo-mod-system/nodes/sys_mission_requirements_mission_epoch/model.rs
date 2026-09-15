// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// On what date does the mission begin?
///
/// `T_epoch = 9862 d  (2027-01-01)`
///
/// Source: `orbitt_case_c1`
///
/// The date every dated question in the design keys off. It is a mission
/// requirement and it lives here, at the system layer, because the mission
/// owns it and several subsystems read it — the solar-weather subsystem is
/// the first to do so. Carried as days since 2000-01-01 so that nothing
/// downstream needs a calendar.
///
/// # Assumptions
///
/// * It is a single date, so the design is sized for a mission that starts then and not for one that slips — fails when solar activity is cyclical, so slipping the start changes the sky the mission flies through rather than merely delaying it. 2027-01-01 sits past the maximum of cycle 25 on the declining side, which is the storm-rich phase; a slip to 2030 would move it toward minimum and every cycle-phase row would return a quieter sky. The record's F10.7 runs 64 to 343 sfu across a cycle, so this is not a rounding difference. A mission whose launch date is uncertain needs the design re-run at both ends of the window, and this row is the one to change.
/// * It is 366 days past the end of the solar-weather record — fails when the record runs to 2025-12-31, day 9496, and this epoch is day 9862. So every row that reads it is answering about a date the record does not cover, by extrapolating a pattern rather than reading an observation. That is the correct thing to do for a design — a mission in the future has no record — and it means the cycle-phase rows fold the phase using a mean cycle length instead of measuring one. Which is exactly what prf_design does beyond its last cycle, and is declared on each row that does it.
pub const NODE_ID: &str = "sys_mission_requirements_mission_epoch";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xca31587a5eb0871e;

pub fn evaluate() -> Result<Time, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Time = match Time::from_unit(9862.0, Unit::Day) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "T_epoch", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "T_epoch", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 568080000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_epoch", value: answer.get(), bound: 568080000.0, edge: Edge::Lower, unit: Time::UNIT, reason: "day 6575 is 2018-01-01. A mission epoch before it would sit inside the archive rather than ahead of the design, and every forward-looking row would be answering a question about the past while presenting it as a prediction" });
    }
    if answer.get() > 1262304000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "T_epoch", value: answer.get(), bound: 1262304000.0, edge: Edge::Upper, unit: Time::UNIT, reason: "day 14610 is 2040-01-01. Beyond it the cycle-phase rows would be folding the phase through more than one unobserved cycle, and the mean cycle length they fold with is measured from two complete cycles. Extrapolating a 11.4-year mean across fifteen years is not a design input, it is a guess with a date on it" });
    }
    Ok(answer)
}
