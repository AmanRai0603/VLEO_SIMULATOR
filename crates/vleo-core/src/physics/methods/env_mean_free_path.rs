//! GENERATED from the method of `env_mean_free_path` by `cargo xtask docs`, translated by
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

/// The method of `env_mean_free_path`, source `us_std_1976`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-envorbit/nodes/env_mean_free_path/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return mean_free_path(n)
/// ```
pub fn evaluate(n: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(n) || pmath::is_nan(n) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::env::mean_free_path(vleo_units::NumberDensity::new(n)).get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
