// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What is the 10.7 cm radio flux doing now, for the forecast to start from?
///
/// `F107_obs = 150`
///
/// Source: `noaa_swpc`
///
/// This is the PERSISTENCE ANCHOR and nothing else. It is not the flux the
/// design is sized to — that is what the subsystem computes and publishes
/// through l3_solar_interface, and what env_f107 now carries into the density
/// chain. It is worth knowing how little rests on it. The estimator that
/// reads it weights it exp(-L / 27 d), which sw_central_expectation's own
/// reading section puts at 0.00114 at the SHORTEST lead its declared range
/// allows, and at the declared five-year mission it is around 5e-30. Measured
/// rather than argued: moving this row across its whole declared range, 60 to
/// 400 sfu, leaves sw_central_expectation at 86.8497 to six figures.
///
/// # Assumptions
///
/// * One number stands for the current state of the Sun — fails when the question is asked at a lead short enough for persistence to carry weight. At a lead of days the flux on the day matters, the 27-day decay has barely started, and a round 150 would be doing real work badly. The declared lead range of this tree starts well past that, which is why it does not.
/// * It is a daily value, not an 81-day mean — fails when it is read as F10.7A. The two are different quantities with different ranges and the thermosphere relation uses both, weighting the mean 3.24 and the daily departure 1.3. sw_f107a_ratio is the row that relates them.
pub const NODE_ID: &str = "sw_f107_observed";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x91781af11d20406f;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(150.0, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "F107_obs", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_obs", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_obs", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor env_f107 declared and for the same reason: below 60 sfu has never been observed and every relation reading F10.7 has no support there" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_obs", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling env_f107 declared: above 400 sfu is beyond the largest recorded daily value, so anything reading it is extrapolating" });
    }
    Ok(answer)
}
