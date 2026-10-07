//! GENERATED from the method of `sw_f107_81day` by `cargo xtask docs`, translated by
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

/// The method of `sw_f107_81day`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_f107_81day/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// #
/// # What the code said beside it:
/// # A pass-through, and the sheet says why at length: at an epoch past the
/// # record there are no 81 days to centre on, so the honest F10.7A is the
/// # phase-conditioned mean-cycle level. What this row adds is the question,
/// # the declared smoothing width and the named density-model contract, not
/// # arithmetic.
/// return level
/// ```
pub fn evaluate(level: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(level) || pmath::is_nan(level) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(level, 10)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
