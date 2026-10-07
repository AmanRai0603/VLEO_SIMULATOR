//! GENERATED from the method of `aero_drag_force` by `cargo xtask docs`, translated by
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

/// The method of `aero_drag_force`, source `vallado2013`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-massaero/nodes/aero_drag_force/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return q * cd * a
/// ```
pub fn evaluate(q: f64, cd: f64, a: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(q) || pmath::is_nan(q) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(cd) || pmath::is_nan(cd) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(a) || pmath::is_nan(a) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(((q * cd) * a), 3)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
