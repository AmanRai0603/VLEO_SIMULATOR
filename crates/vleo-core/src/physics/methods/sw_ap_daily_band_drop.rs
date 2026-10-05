//! GENERATED from the method of `sw_ap_daily_band_drop` by `cargo xtask docs`, translated by
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

/// The method of `sw_ap_daily_band_drop`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_ap_daily_band_drop/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: measured data, the 5th-percentile within-rotation Ap departure conditioned on rotation level, from the NOAA SWPC record (noaa_swpc); bins of 15 per cent, at least 200 days each.
/// # A Table1 lookup: linear between knots, clamped at both ends (never extrapolated). The answer is a magnitude; sw_ap_cold_short subtracts it.
/// let band = interp(level, [4.0, 6.0, 8.0, 11.0, 15.0, 20.0, 24.0, 26.0], [2.9704, 4.1685, 5.5926, 7.5556, 10.8370, 14.2963, 17.6926, 21.0185])
///
/// # Generated guards: the declared domain of dAp_day_low.
/// if band < 0 then
///   refuse "below zero is not a distance; a negative drop means the tail has been read from the wrong end"
/// end
/// if band > 150 then
///   refuse "above 150 the value exceeds anything the record supports, so it is an arithmetic error rather than a quiet sky"
/// end
/// return band
/// ```
pub fn evaluate(level: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(level) || pmath::is_nan(level) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let band: f64 = rt::fin(
        pmath::interp(
            level,
            &[4.0, 6.0, 8.0, 11.0, 15.0, 20.0, 24.0, 26.0],
            &[
                2.9704, 4.1685, 5.5926, 7.5556, 10.837, 14.2963, 17.6926, 21.0185,
            ],
        ),
        4,
    )?;
    if (band < 0.0) {
        return Err(MethodError::Refused("below zero is not a distance; a negative drop means the tail has been read from the wrong end"));
    }
    if (band > 150.0) {
        return Err(MethodError::Refused("above 150 the value exceeds anything the record supports, so it is an arithmetic error rather than a quiet sky"));
    }
    return Ok(rt::fin(band, 13)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
