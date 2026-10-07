//! GENERATED from the method of `pay_scene_volume` by `cargo xtask docs`, translated by
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

/// The method of `pay_scene_volume`, source `ccsds122`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-payload/nodes/pay_scene_volume/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 4.
/// return scene_data_volume(n, n, b, 4, cr)
/// ```
pub fn evaluate(n: f64, b: f64, cr: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(n) || pmath::is_nan(n) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(b) || pmath::is_nan(b) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(cr) || pmath::is_nan(cr) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::payload::scene_data_volume(n, n, b, 4.0, cr).get()),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
