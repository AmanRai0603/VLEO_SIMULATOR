//! GENERATED from the method of `env_atomic_oxygen_density` by `cargo xtask docs`, translated by
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

/// The method of `env_atomic_oxygen_density`, source `jacchia1971`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-envorbit/nodes/env_atomic_oxygen_density/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return thermosphere_o(h, t_inf)
/// ```
pub fn evaluate(h: f64, t_inf: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(h) || pmath::is_nan(h) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(t_inf) || pmath::is_nan(t_inf) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::env::composition(
            vleo_units::Length::new(h),
            vleo_units::Temperature::new(t_inf),
        )
        .o
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
