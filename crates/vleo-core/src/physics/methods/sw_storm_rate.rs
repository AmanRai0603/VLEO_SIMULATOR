//! GENERATED from the method of `sw_storm_rate` by `cargo xtask docs`, translated by
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

/// The method of `sw_storm_rate`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_storm_rate/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: the 28-point table.
/// #
/// # What the code said beside it:
/// # The exceedance rate at each of the 28 PUBLISHED Kp values, counted on
/// # solar-weather@2026.09.14: days whose daily Ap exceeds that Kp's tabulated ap,
/// # per year of record. Tabulating at the scale's own resolution leaves at most a
/// # third of a Kp unit for the interpolation to be wrong in, which matters because
/// # the rate falls four orders of magnitude from end to end.
/// #
/// # The last two entries are 0.0, and the sheet is emphatic that this means the
/// # record contains no such day rather than that no such day occurs.
/// const KP = [0, 0.333333, 0.666667, 1, 1.333333, 1.666667, 2, 2.333333, 2.666667, 3, 3.333333, 3.666667, 4, 4.333333, 4.666667, 5, 5.333333, 5.666667, 6, 6.333333, 6.666667, 7, 7.333333, 7.666667, 8, 8.333333, 8.666667, 9] [1]
/// const RATE = [364.1506, 343.8648, 316.2024, 275.5245, 239.1733, 207.2552, 180.4794, 135.8649, 90.3993, 63.588, 45.572, 30.7478, 18.6898, 12.5899, 7.6958, 4.3621, 3.1209, 1.7378, 1.2413, 1.0285, 0.6738, 0.2837, 0.2483, 0.1419, 0.0355, 0.0355, 0, 0] [1]
/// return interp(kp, KP, RATE)
/// ```
pub fn evaluate(kp: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(kp) || pmath::is_nan(kp) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let KP: [f64; 28] = [
        0.0, 0.333333, 0.666667, 1.0, 1.333333, 1.666667, 2.0, 2.333333, 2.666667, 3.0, 3.333333,
        3.666667, 4.0, 4.333333, 4.666667, 5.0, 5.333333, 5.666667, 6.0, 6.333333, 6.666667, 7.0,
        7.333333, 7.666667, 8.0, 8.333333, 8.666667, 9.0,
    ];
    let RATE: [f64; 28] = [
        364.1506, 343.8648, 316.2024, 275.5245, 239.1733, 207.2552, 180.4794, 135.8649, 90.3993,
        63.588, 45.572, 30.7478, 18.6898, 12.5899, 7.6958, 4.3621, 3.1209, 1.7378, 1.2413, 1.0285,
        0.6738, 0.2837, 0.2483, 0.1419, 0.0355, 0.0355, 0.0, 0.0,
    ];
    return Ok(rt::fin(pmath::interp(kp, &KP, &RATE), 16)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
