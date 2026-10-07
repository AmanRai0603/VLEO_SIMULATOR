//! GENERATED from the method of `mass_margin` by `cargo xtask docs`, translated by
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

/// The method of `mass_margin`, source `ecss_e_st_10_02`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-closure/nodes/mass_margin/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return mass_margin(l, w)
/// ```
pub fn evaluate(l: f64, w: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(l) || pmath::is_nan(l) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(w) || pmath::is_nan(w) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::mass::mass_margin(vleo_units::Mass::new(l), vleo_units::Mass::new(w))
            .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
