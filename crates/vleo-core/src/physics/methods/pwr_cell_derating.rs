//! GENERATED from the method of `pwr_cell_derating` by `cargo xtask docs`, translated by
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

/// The method of `pwr_cell_derating`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-power/nodes/pwr_cell_derating/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # "* 1 [unit]" gives a number this design states as a pure ratio the unit the kernel reads it in,
/// # and "/ 1 [unit]" the other way; the number itself does not change.
/// # Constants the code held, written out here as it held them: 0.3; 298.15 [K].
/// return max(0.3, 1 + k * 1 [1/K] * (t - 298.15 [K]))
/// ```
pub fn evaluate(k: f64, t: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(k) || pmath::is_nan(k) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(t) || pmath::is_nan(t) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        pmath::max(0.3, (1.0 + ((k * 1.0) * (t - 298.15)))),
        6,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
