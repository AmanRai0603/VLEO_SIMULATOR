// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What daily planetary Ap should the mission window be expected to sit at?
///
/// `Ap_central = the last rotation forecast, held forward = 22.0954`
///
/// Source: `noaa_swpc`
///
/// Every other Ap row in this subsystem answers an EXTREME — what recurs
/// once per mission, what G level to survive, how often a threshold is
/// crossed. None of them says what the window sits at on an ordinary day, and
/// a band needs a centre before it can have edges.
///
/// # Assumptions
///
/// * The level the sun was last at is the level it will be at — fails when the window is long or far out. This carries no cycle trend at all: the same number is published for a window opening next month and one opening in 2032, and over a 365-day window the sun demonstrably moves
/// * A rotation-mean level stands in for a daily level — fails when a design reads it as a day. It is the mean of 27 days; half the days in the window are above it by construction, which is what sw_ap_daily_band_spread exists to say
/// * The phase past the cycle table is an extrapolation — fails when it is read as measured. The last rotations sit past cycle 25's tabulated end and their phase wraps on the mean length of three cycles, one of which is incomplete
pub const NODE_ID: &str = "sw_ap_central_expectation";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x634220313cf3c02c;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(22.095389, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_central", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_central", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_central", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a value below zero is not a spread, and Ap itself floors at zero — a quiet day really is Ap 0" });
    }
    if answer.get() > 80.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_central", value: answer.get(), bound: 80.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 80 the value exceeds anything the record supports for this quantity, so it is an arithmetic error rather than an active sun" });
    }
    Ok(answer)
}
