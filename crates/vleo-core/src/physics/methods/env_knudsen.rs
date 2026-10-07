//! GENERATED from the method of `env_knudsen` by `cargo xtask docs`, translated by
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

/// The method of `env_knudsen`, source `us_std_1976`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-envorbit/nodes/env_knudsen/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return knudsen(lam, l_body)
/// ```
pub fn evaluate(lam: f64, l_body: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(lam) || pmath::is_nan(lam) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(l_body) || pmath::is_nan(l_body) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::env::knudsen(
            vleo_units::Length::new(lam),
            vleo_units::Length::new(l_body),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
