//! GENERATED from the method of `sw_ap_daily_band_spread` by `cargo xtask docs`, translated by
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

/// The method of `sw_ap_daily_band_spread`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_ap_daily_band_spread/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: measured 95th-percentile within-rotation Ap departures by rotation level (noaa_swpc record; bins of 15 per cent, at least 200 days each).
/// # The table clamps at both ends, so the only input the code refuses is one that is not a number:
/// # it makes the answer not a number, which the generated guard refuses.
/// if not (level == level) then
///   refuse "the computation produced a value that is not a number"
/// end
/// let band = interp(level, [4.0, 6.0, 8.0, 11.0, 15.0, 20.0, 24.0, 26.0] [1], [6.0167, 8.1481, 11.2222, 14.6667, 23.7852, 28.6759, 40.6148, 63.8519] [1])
/// if band < 0 then
///   refuse "a value below zero is not a spread, and Ap itself floors at zero"
/// end
/// if band > 150 then
///   refuse "above 150 the value exceeds anything the record supports, so it is an arithmetic error"
/// end
/// return band
/// ```
pub fn evaluate(level: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(level) || pmath::is_nan(level) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if (!(level == level)) {
        return Err(MethodError::Refused(
            "the computation produced a value that is not a number",
        ));
    }
    let band: f64 = rt::fin(
        pmath::interp(
            level,
            &[4.0, 6.0, 8.0, 11.0, 15.0, 20.0, 24.0, 26.0],
            &[
                6.0167, 8.1481, 11.2222, 14.6667, 23.7852, 28.6759, 40.6148, 63.8519,
            ],
        ),
        8,
    )?;
    if (band < 0.0) {
        return Err(MethodError::Refused(
            "a value below zero is not a spread, and Ap itself floors at zero",
        ));
    }
    if (band > 150.0) {
        return Err(MethodError::Refused("above 150 the value exceeds anything the record supports, so it is an arithmetic error"));
    }
    return Ok(rt::fin(band, 15)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
