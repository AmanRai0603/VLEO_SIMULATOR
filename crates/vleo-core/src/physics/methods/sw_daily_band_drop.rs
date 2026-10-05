//! GENERATED from the method of `sw_daily_band_drop` by `cargo xtask docs`, translated by
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

/// The method of `sw_daily_band_drop`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_daily_band_drop/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: measured 5th-percentile within-rotation F10.7 departures, as magnitudes, by rotation level (noaa_swpc record; bins of 15 per cent, at least 200 days each).
/// # The table clamps at both ends, so the only input the code refuses is one that is not a number:
/// # it makes the answer not a number, which the generated guard refuses.
/// if not (level == level) then
///   refuse "the computation produced a value that is not a number"
/// end
/// let band = interp(level, [70.0, 85.0, 100.0, 120.0, 145.0, 175.0, 210.0] [1], [4.3870, 10.2000, 16.7407, 23.2593, 32.3852, 37.2556, 40.7778] [1])
/// if band < 0 then
///   refuse "below zero is not a distance; the tail has been read from the wrong end"
/// end
/// if band > 120 then
///   refuse "above 120 sfu exceeds the largest single-day excursion in the record, so it is an arithmetic error"
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
            &[70.0, 85.0, 100.0, 120.0, 145.0, 175.0, 210.0],
            &[4.387, 10.2, 16.7407, 23.2593, 32.3852, 37.2556, 40.7778],
        ),
        8,
    )?;
    if (band < 0.0) {
        return Err(MethodError::Refused(
            "below zero is not a distance; the tail has been read from the wrong end",
        ));
    }
    if (band > 120.0) {
        return Err(MethodError::Refused("above 120 sfu exceeds the largest single-day excursion in the record, so it is an arithmetic error"));
    }
    return Ok(rt::fin(band, 15)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
