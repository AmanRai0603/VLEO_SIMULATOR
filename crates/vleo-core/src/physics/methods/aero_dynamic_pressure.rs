//! GENERATED from the method of `aero_dynamic_pressure` by `cargo xtask docs`, translated by
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

/// The method of `aero_dynamic_pressure`, source `vallado2013`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-massaero/nodes/aero_dynamic_pressure/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return dynamic_pressure(rho, v)
/// ```
pub fn evaluate(rho: f64, v: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(rho) || pmath::is_nan(rho) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(v) || pmath::is_nan(v) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::aero::dynamic_pressure(
            vleo_units::MassDensity::new(rho),
            vleo_units::Velocity::new(v),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
