// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What sustained F10.7 does the record say this mission will present?
///
/// `F107_ach_flux = sw_f107_design_long`
///
/// Source: `noaa_swpc`
///
/// The headline of the five. It is the same number the interface carries to
/// sys_space_environment as its primary member, restated on the achieved side
/// of the closure so that the comparison against what the spacecraft can
/// sustain happens on a row rather than in somebody's head.
///
/// # Assumptions
///
/// * It inherits every limitation of the row it restates, and a closure reading it sees none of them — fails when the same cost as the interface, and worth repeating on the row a closure actually binds. The number is sized on the cycle analogue at the mission's own epoch rather than on the record's unconditional mean, which this chain now reads the epoch to do; where it is a band it is 1.28 sigma, the 90th percentile, and not the 95 per cent the run is labelled; and where it is a return level its top end rests on two observations in 28.2 years. A margin computed from this row against a capability carries none of that, and will look like a clean number either way.
pub const NODE_ID: &str = "l3_solar_ach_01";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xddeb61e35dbb0a27;

pub fn evaluate(conclusion: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : restate the subsystem's conclusion on the achieved side of the closure -> Ratio
    let ach: Ratio = conclusion;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = ach;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_ach_flux", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_ach_flux", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as env_f107 and sw_f107_design_long: below 60 sfu has never been observed and no relation reading F10.7 has support there" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_ach_flux", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as env_f107 and sw_f107_design_long: above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value" });
    }
    Ok(answer)
}
