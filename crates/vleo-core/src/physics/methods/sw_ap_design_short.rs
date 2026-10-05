//! GENERATED from the method of `sw_ap_design_short` by `cargo xtask docs`, translated by
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

/// The method of `sw_ap_design_short`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_ap_design_short/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: none of its own; the sustained level and the within-rotation daily departure both come from the NOAA SWPC record (noaa_swpc) via their rows; migrated from prf_density.m:227 designWindow_ (hotday for ap).
/// # The rotation error and the within-rotation departure are of different things, so they add.
/// let single_day = sustained + daily
///
/// # Generated guards: the declared domain of Ap_short.
/// if single_day < 0 then
///   refuse "Ap floors at zero, so a single-day design level below it means a spread has been subtracted rather than added"
/// end
/// if single_day > 400 then
///   refuse "400 is the top of the Ap index itself; a value above it is not a geomagnetic index at all"
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
    if (single_day < 0.0) {
        return Err(MethodError::Refused("Ap floors at zero, so a single-day design level below it means a spread has been subtracted rather than added"));
    }
    if (single_day > 400.0) {
        return Err(MethodError::Refused("400 is the top of the Ap index itself; a value above it is not a geomagnetic index at all"));
    }
    return Ok(rt::fin(single_day, 13)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
