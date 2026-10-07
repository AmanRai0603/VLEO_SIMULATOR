//! GENERATED from the method of `pay_gsd_diffraction` by `cargo xtask docs`, translated by
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

/// The method of `pay_gsd_diffraction`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-payload/nodes/pay_gsd_diffraction/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return diffraction_limited_gsd(h, lam, d)
/// ```
pub fn evaluate(h: f64, lam: f64, d: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(h) || pmath::is_nan(h) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(lam) || pmath::is_nan(lam) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(d) || pmath::is_nan(d) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::payload::diffraction_limited_gsd(
            vleo_units::Length::new(h),
            vleo_units::Length::new(lam),
            vleo_units::Length::new(d),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
