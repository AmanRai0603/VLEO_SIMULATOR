//! GENERATED from the method of `sw_f107_design_short` by `cargo xtask docs`, translated by
//! the fixed rules in crates/vleo-sheet/src/method.rs. Do not edit: the method
//! is changed on the node's form, and this is written again from it.

#![allow(
    clippy::all,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    unreachable_code,
    unused_assignments,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_parens,
    non_snake_case
)]

use vleo_units::constants::*;
use vleo_units::method_rt::{self as rt, MethodError};
use vleo_units::pmath;

/// The method of `sw_f107_design_short`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_f107_design_short/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: none of its own; the sustained level and the within-rotation daily departure come from the NOAA SWPC record (noaa_swpc) via their rows; migrated from prf_density.m:227 designWindow_.
/// # A forecast error about the rotation's level and the sun's variability within it are different things, so they add rather than combine in quadrature.
/// let single_day = sustained + daily
///
/// # Generated guards: the declared domain of F107_short.
/// if single_day < 60 then
///   refuse "below 60 sfu has never been observed; a single-day design level below it means a spread has been subtracted rather than added"
/// end
/// if single_day > 400 then
///   refuse "above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value"
/// end
/// return single_day
/// ```
pub fn evaluate(sustained: f64, daily: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(sustained) || pmath::is_nan(sustained) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(daily) || pmath::is_nan(daily) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let single_day: f64 = rt::fin((sustained + daily), 4)?;
    if (single_day < 60.0) {
        return Err(MethodError::Refused("below 60 sfu has never been observed; a single-day design level below it means a spread has been subtracted rather than added"));
    }
    if (single_day > 400.0) {
        return Err(MethodError::Refused("above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value"));
    }
    return Ok(rt::fin(single_day, 13)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
