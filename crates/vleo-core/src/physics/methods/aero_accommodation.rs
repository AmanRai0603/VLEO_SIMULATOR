//! GENERATED from the method of `aero_accommodation` by `cargo xtask docs`, translated by
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

/// The method of `aero_accommodation`, source `moe2005`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-massaero/nodes/aero_accommodation/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return accommodation_coefficient(n_o, t_inf)
/// ```
pub fn evaluate(n_o: f64, t_inf: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(n_o) || pmath::is_nan(n_o) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(t_inf) || pmath::is_nan(t_inf) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::aero::accommodation_coefficient(
            vleo_units::NumberDensity::new(n_o),
            vleo_units::Temperature::new(t_inf),
        )),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
