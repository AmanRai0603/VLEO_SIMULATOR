// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How far ahead does the published short-term outlook reach?
///
/// `L_short = 27 d`
///
/// Source: `noaa_swpc`
///
/// The lead the forecast-verification rows are evaluated at.
/// sw_forecast_skill and sw_forecast_bias both answer 'at what lead', and
/// this is the lead worth asking about: the end of the published window,
/// where the outlook is weakest and a design reading it is most exposed.
///
/// # Assumptions
///
/// * One lead, and the skill of the outlook is strongly lead-dependent — fails when measured on this record the issued outlook LOSES to persistence at leads 1 to 4, beats it from 5 to 23 with a peak skill of +0.24 near 13, and loses again from 24. So a single lead cannot characterise the product: 27 days is the pessimistic end and a row evaluated there says nothing about the useful middle. It is chosen because a design wants to know how bad the far edge is, not how good the centre is, and because it is the window's own length rather than a point somebody picked inside it.
/// * 27 days is the rotation and the window at once, and those are two different reasons — fails when the synodic solar rotation is about 27.3 days and the published outlook is exactly 27 rows, so the two coincide closely enough that the sheet cannot tell them apart. If SWPC changed the outlook length the row would need to say which reason it meant. It is carried as 27 exactly, matching the data rather than the rotation, because what it indexes is the forecast table.
pub const NODE_ID: &str = "sw_outlook_lead";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x5de6ab75180c337b;

pub fn evaluate() -> Result<Time, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Time = match Time::from_unit(27.0, Unit::Day) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "L_short", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "L_short", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 86400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L_short", value: answer.get(), bound: 86400.0, edge: Edge::Lower, unit: Time::UNIT, reason: "a lead of less than a day is not a forecast the published outlook makes; its first row is lead 1" });
    }
    if answer.get() > 2332800.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L_short", value: answer.get(), bound: 2332800.0, edge: Edge::Upper, unit: Time::UNIT, reason: "27 is the length of the published window and the last lead the forecast table contains. Beyond it there is no issued forecast to verify against, so a larger value would be asking the verification rows about rows that do not exist" });
    }
    Ok(answer)
}
