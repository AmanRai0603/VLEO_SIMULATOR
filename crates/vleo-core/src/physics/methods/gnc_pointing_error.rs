//! GENERATED from the method of `gnc_pointing_error` by `cargo xtask docs`, translated by
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

/// The method of `gnc_pointing_error`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-acs/nodes/gnc_pointing_error/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return 3 * pointing_error_rss(sen, ali, ctl, thm)
/// ```
pub fn evaluate(sen: f64, ali: f64, ctl: f64, thm: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(sen) || pmath::is_nan(sen) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ali) || pmath::is_nan(ali) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ctl) || pmath::is_nan(ctl) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(thm) || pmath::is_nan(thm) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (3.0 * (vleo_core::physics::gnc::pointing_error_rss(&[
            vleo_units::Angle::new(sen),
            vleo_units::Angle::new(ali),
            vleo_units::Angle::new(ctl),
            vleo_units::Angle::new(thm),
        ])
        .get())),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
