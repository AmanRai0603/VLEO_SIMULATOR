//! GENERATED from the method of `sw_ap_cold_short` by `cargo xtask docs`, translated by
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

/// The method of `sw_ap_cold_short`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_ap_cold_short/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: NOAA SWPC (noaa_swpc) record via prf_density.m:227 (designWindow_); the daily drop is the low tail's own magnitude, from sw_ap_daily_band_drop.
/// # The single-day cold level: the sustained cold level less the within-rotation daily Ap drop (a magnitude, so the sign lives here).
/// # The runtime's own finiteness check stands in for the generated "not a number" guard.
/// let single_day = sustained - daily
/// if single_day < 0 then
///   refuse "Ap floors at zero: a perfectly quiet day is Ap 0 and there is nothing below it"
/// end
/// if single_day > 400 then
///   refuse "400 is the top of the Ap index itself: a value above it means a sign is wrong in the chain above"
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
    let single_day: f64 = rt::fin((sustained - daily), 5)?;
    if (single_day < 0.0) {
        return Err(MethodError::Refused(
            "Ap floors at zero: a perfectly quiet day is Ap 0 and there is nothing below it",
        ));
    }
    if (single_day > 400.0) {
        return Err(MethodError::Refused("400 is the top of the Ap index itself: a value above it means a sign is wrong in the chain above"));
    }
    return Ok(rt::fin(single_day, 12)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
