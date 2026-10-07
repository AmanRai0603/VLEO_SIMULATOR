//! GENERATED from the method of `cost_per_year` by `cargo xtask docs`, translated by
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

/// The method of `cost_per_year`, source `nasa_cer`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-closure/nodes/cost_per_year/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return cost_per_service_unit(c, y / 31557600 [s])
/// ```
pub fn evaluate(c: f64, y: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(c) || pmath::is_nan(c) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(y) || pmath::is_nan(y) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::cost::cost_per_service_unit(
            vleo_units::Money::new(c),
            rt::div(y, 31557600.0, 3)?,
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
