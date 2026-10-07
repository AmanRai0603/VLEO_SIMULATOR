//! GENERATED from the method of `env_magnetic_field` by `cargo xtask docs`, translated by
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

/// The method of `env_magnetic_field`, source `igrf2020`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-envorbit/nodes/env_magnetic_field/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return magnetic_field(r, lat_m)
/// ```
pub fn evaluate(r: f64, lat_m: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(r) || pmath::is_nan(r) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(lat_m) || pmath::is_nan(lat_m) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::env::magnetic_field(
            vleo_units::Length::new(r),
            vleo_units::Angle::new(lat_m),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
