//! GENERATED from the method of `env_exospheric_temperature` by `cargo xtask docs`, translated by
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

/// The method of `env_exospheric_temperature`, source `jacchia1971`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-envorbit/nodes/env_exospheric_temperature/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return exospheric_temperature(f107, f107a, kp)
/// ```
pub fn evaluate(f107: f64, f107a: f64, kp: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(f107) || pmath::is_nan(f107) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(f107a) || pmath::is_nan(f107a) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kp) || pmath::is_nan(kp) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::env::exospheric_temperature(f107, f107a, kp).get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
