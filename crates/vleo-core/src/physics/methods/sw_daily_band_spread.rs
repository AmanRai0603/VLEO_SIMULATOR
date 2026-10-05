//! GENERATED from the method of `sw_daily_band_spread` by `cargo xtask docs`, translated by
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

/// The method of `sw_daily_band_spread`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_daily_band_spread/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Measured data (noaa_swpc): 95th-percentile within-rotation departure of daily F10.7 from its 27-day mean, conditioned on rotation level, bins of 15 per cent with at least 200 days each.
/// # The generated finite check on the answer: the table is finite and held at its ends, so the only
/// # non-finite answer comes from a level that is not a number, and it is refused here, before the lookup.
/// if not (level == level) then
///   refuse "The computation produced a value that is not a number."
/// end
/// # Linear between knots, held at both ends (Table1::at): past the ends the record does not continue.
/// # x: rotation level, sfu; y: 95th-percentile departure above that rotation, sfu.
/// let band = interp(level, [70.0, 85.0, 100.0, 120.0, 145.0, 175.0, 210.0] [1], [4.6259, 11.4741, 18.2741, 27.1074, 34.6148, 40.6926, 46.9259] [1])
/// if band < 0 then
///   refuse "Below zero is not a spread."
/// end
/// if band > 120 then
///   refuse "Above 120 sfu the departure exceeds the record's largest single-day excursion, so it is an arithmetic error."
/// end
/// return band
/// ```
pub fn evaluate(level: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(level) || pmath::is_nan(level) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if (!(level == level)) {
        return Err(MethodError::Refused(
            "The computation produced a value that is not a number.",
        ));
    }
    let band: f64 = rt::fin(
        pmath::interp(
            level,
            &[70.0, 85.0, 100.0, 120.0, 145.0, 175.0, 210.0],
            &[4.6259, 11.4741, 18.2741, 27.1074, 34.6148, 40.6926, 46.9259],
        ),
        10,
    )?;
    if (band < 0.0) {
        return Err(MethodError::Refused("Below zero is not a spread."));
    }
    if (band > 120.0) {
        return Err(MethodError::Refused("Above 120 sfu the departure exceeds the record's largest single-day excursion, so it is an arithmetic error."));
    }
    return Ok(rt::fin(band, 17)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
