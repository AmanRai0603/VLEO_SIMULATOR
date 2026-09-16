// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How much of the next rotation's level does the pattern fail to explain?
///
/// `sigma_total = std(pred - truth) over 361 walk-forward next-rotation forecasts = 13.5550 sfu`
///
/// Source: `noaa_swpc`
///
/// sw_mean_cycle_level and sw_cycle_phase say where in the cycle the mission
/// sits and what that phase usually brings. This says how wrong that has
/// been, measured on the record, one rotation at a time, never using a
/// rotation to predict itself. It is the number a design carries as margin:
/// the central expectation without it is a line with no width.
///
/// # Assumptions
///
/// * One sigma holds across the whole cycle — fails when it does not, and the source this was rebuilt from says so about its own number: 'sigma is NOT flat across the cycle; a design that uses one number is too tight somewhere and too loose somewhere else.' prf_rebuild reports sigma split into five phase bins for that reason. This row publishes the pooled number, so a design near solar maximum is given a band that is too narrow and one near minimum a band too wide
/// * The 273-day hole in 2017 is filled by straight-line interpolation before the rotations are cut — fails when those interpolated days are counted as observations. Nine months of invented flux sit inside about ten rotations, and they are smoother than the sun, so every one of those rotations is easier to predict than a real one and the pooled spread is a little narrower than the record can support
/// * It is the residual of a pattern, not of a forecast — fails when this is read as what a forecaster would get wrong. There is no forecast in it: no flare watch, no active-region count, no observation later than the rotation before. A real 27-day outlook does better, which is what sw_forecast_skill measures
pub const NODE_ID: &str = "sw_mean_band_spread";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x6614bf51245d4ff9;

pub fn evaluate() -> Result<Ratio, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Ratio = match Ratio::from_unit(13.554959, Unit::One) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "sigma_total", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "sigma_total", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sigma_total", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "a spread of zero would mean the pattern predicts every rotation exactly, which the record contradicts at every rotation it scores; below zero is not a spread" });
    }
    if answer.get() > 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "sigma_total", value: answer.get(), bound: 60.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 60 sfu the residual would exceed the standard deviation of the rotation means themselves, so the pattern would be worse than predicting the record's own mean and the band would be arithmetic rather than physics" });
    }
    Ok(answer)
}
