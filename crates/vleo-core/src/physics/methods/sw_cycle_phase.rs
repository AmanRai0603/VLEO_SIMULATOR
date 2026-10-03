//! GENERATED from the method of `sw_cycle_phase` by `cargo xtask docs`, translated by
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

/// The method of `sw_cycle_phase`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_cycle_phase/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: NOAA SWPC (noaa_swpc) solar_cycles.csv in solar-weather — cycle 25 began day 7274 (2019-12-01); mean of complete cycles 23 (11.88 yr) and 24 (11.00 yr) is 11.44 yr.
/// # The runtime's own finiteness check stands in for the generated "not a number" guard.
/// const start_25 = 7274 [d]   # cycle 25 start, days on the mission epoch scale
/// const mean_length = 11.44 [1]   # mean length of cycles 23 and 24, in Julian years
/// let out = (epoch - start_25) / (mean_length * 365.25 [d])
/// if out < 0 then
///   refuse "a phase cannot be negative: the epoch precedes the cycle it was assigned to"
/// end
/// if out > 1 then
///   refuse "past the end of the cycle the epoch belongs to the next one, whose start the record does not contain"
/// end
/// return out
/// ```
pub fn evaluate(epoch: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(epoch) || pmath::is_nan(epoch) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let start_25: f64 = rt::fin(628473600.0, 4)?;
    let mean_length: f64 = rt::fin(11.44, 5)?;
    let out: f64 = rt::fin(
        rt::div((epoch - start_25), (mean_length * 31557600.0), 6)?,
        6,
    )?;
    if (out < 0.0) {
        return Err(MethodError::Refused(
            "a phase cannot be negative: the epoch precedes the cycle it was assigned to",
        ));
    }
    if (out > 1.0) {
        return Err(MethodError::Refused("past the end of the cycle the epoch belongs to the next one, whose start the record does not contain"));
    }
    return Ok(rt::fin(out, 13)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
