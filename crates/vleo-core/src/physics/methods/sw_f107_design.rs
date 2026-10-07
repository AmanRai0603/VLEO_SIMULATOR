//! GENERATED from the method of `sw_f107_design` by `cargo xtask docs`, translated by
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

/// The method of `sw_f107_design`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_f107_design/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// #
/// # What the code said beside it:
/// # The study's construction, and the whole of it: the centre plus the spread.
/// # Both come from rows that declare where their numbers came from and what
/// # they cannot do, so there is nothing to measure or choose here. The
/// # confidence is whatever sw_uncertainty_growth publishes, which is the 95th
/// # percentile, and the sheet says a mission needing another one must change
/// # that row rather than this one.
/// return central + spread
/// ```
pub fn evaluate(central: f64, spread: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(central) || pmath::is_nan(central) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(spread) || pmath::is_nan(spread) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin((central + spread), 11)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
