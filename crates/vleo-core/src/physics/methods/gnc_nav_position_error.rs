//! GENERATED from the method of `gnc_nav_position_error` by `cargo xtask docs`, translated by
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

/// The method of `gnc_nav_position_error`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-acs/nodes/gnc_nav_position_error/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return navigation_position_error(u, g)
/// ```
pub fn evaluate(u: f64, g: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(u) || pmath::is_nan(u) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(g) || pmath::is_nan(g) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::gnc::navigation_position_error(vleo_units::Length::new(u), g).get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
