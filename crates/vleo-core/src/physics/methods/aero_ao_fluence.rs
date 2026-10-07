//! GENERATED from the method of `aero_ao_fluence` by `cargo xtask docs`, translated by
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

/// The method of `aero_ao_fluence`, source `moe2005`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-massaero/nodes/aero_ao_fluence/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # "* 1 [unit]" gives a number this design states as a pure ratio the unit the kernel reads it in,
/// # and "/ 1 [unit]" the other way; the number itself does not change.
/// return atomic_oxygen_fluence(n_o, v, t) / 1 [1/m^2]
/// ```
pub fn evaluate(n_o: f64, v: f64, t: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(n_o) || pmath::is_nan(n_o) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(v) || pmath::is_nan(v) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(t) || pmath::is_nan(t) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        rt::div(
            (vleo_core::physics::aero::atomic_oxygen_fluence(
                vleo_units::NumberDensity::new(n_o),
                vleo_units::Velocity::new(v),
                vleo_units::Time::new(t),
            )),
            1.0,
            5,
        )?,
        5,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
