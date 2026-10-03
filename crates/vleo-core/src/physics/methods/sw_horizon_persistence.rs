//! GENERATED from the method of `sw_horizon_persistence` by `cargo xtask docs`, translated by
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

/// The method of `sw_horizon_persistence`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_horizon_persistence/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: eighteen measured RMS values of F107(t+L) - F107(t) over observed pairs, from bundles solar-weather@2026.09.14 (NOAA SWPC record, noaa_swpc).
/// # A Table1 lookup on the lead in days: linear between leads, held at both ends. The dip at 27 d is the
/// # synodic solar rotation and is deliberate, not a transcription error.
/// let err = interp(lead, [1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 14.0, 20.0, 27.0, 40.0, 60.0, 90.0, 135.0, 180.0, 270.0, 365.0, 547.0, 730.0] [d], [7.0784, 10.3796, 13.6671, 19.3021, 23.6294, 27.6511, 29.1328, 25.4695, 22.4321, 29.602, 27.9575, 30.1549, 31.0394, 32.9634, 35.7538, 38.8277, 43.7693, 49.6285])
///
/// # Generated guards: the declared domain of D_pers.
/// if err < 0 then
///   refuse "an RMS cannot be negative"
/// end
/// if err > 65 then
///   refuse "above 65 is unreachable by this relation (saturation is 62.77 sfu, the largest entry 49.63), so it is a broken table rather than an extreme sky"
/// end
/// return err
/// ```
pub fn evaluate(lead: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(lead) || pmath::is_nan(lead) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let err: f64 = rt::fin(
        pmath::interp(
            lead,
            &[
                86400.0, 172800.0, 259200.0, 432000.0, 604800.0, 864000.0, 1209600.0, 1728000.0,
                2332800.0, 3456000.0, 5184000.0, 7776000.0, 11664000.0, 15552000.0, 23328000.0,
                31536000.0, 47260800.0, 63072000.0,
            ],
            &[
                7.0784, 10.3796, 13.6671, 19.3021, 23.6294, 27.6511, 29.1328, 25.4695, 22.4321,
                29.602, 27.9575, 30.1549, 31.0394, 32.9634, 35.7538, 38.8277, 43.7693, 49.6285,
            ],
        ),
        5,
    )?;
    if (err < 0.0) {
        return Err(MethodError::Refused("an RMS cannot be negative"));
    }
    if (err > 65.0) {
        return Err(MethodError::Refused("above 65 is unreachable by this relation (saturation is 62.77 sfu, the largest entry 49.63), so it is a broken table rather than an extreme sky"));
    }
    return Ok(rt::fin(err, 14)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
