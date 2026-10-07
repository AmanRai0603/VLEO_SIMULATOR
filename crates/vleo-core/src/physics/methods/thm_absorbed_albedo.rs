//! GENERATED from the method of `thm_absorbed_albedo` by `cargo xtask docs`, translated by
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

/// The method of `thm_absorbed_albedo`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-thermal/nodes/thm_absorbed_albedo/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return absorbed_albedo(a, al, f)
/// ```
pub fn evaluate(a: f64, al: f64, f: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(a) || pmath::is_nan(a) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(al) || pmath::is_nan(al) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(f) || pmath::is_nan(f) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::thermal::absorbed_albedo(
            vleo_units::Area::new(a),
            vleo_units::Ratio::new(al),
            vleo_units::Ratio::new(f),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
