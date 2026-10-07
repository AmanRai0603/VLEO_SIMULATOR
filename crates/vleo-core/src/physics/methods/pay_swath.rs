//! GENERATED from the method of `pay_swath` by `cargo xtask docs`, translated by
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

/// The method of `pay_swath`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-payload/nodes/pay_swath/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return optical_swath(g, n)
/// ```
pub fn evaluate(g: f64, n: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(g) || pmath::is_nan(g) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(n) || pmath::is_nan(n) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::payload::optical_swath(vleo_units::Length::new(g), n).get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
