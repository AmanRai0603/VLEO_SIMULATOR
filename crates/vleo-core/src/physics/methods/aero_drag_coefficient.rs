//! GENERATED from the method of `aero_drag_coefficient` by `cargo xtask docs`, translated by
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

/// The method of `aero_drag_coefficient`, source `doornbos2011`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-massaero/nodes/aero_drag_coefficient/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return cylinder_drag_coefficient(s, l, d, alpha, t_w, v, m)
/// ```
pub fn evaluate(
    s: f64,
    l: f64,
    d: f64,
    alpha: f64,
    t_w: f64,
    v: f64,
    m: f64,
    kn: f64,
) -> Result<f64, MethodError> {
    if !pmath::is_finite(s) || pmath::is_nan(s) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(l) || pmath::is_nan(l) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(d) || pmath::is_nan(d) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(alpha) || pmath::is_nan(alpha) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(t_w) || pmath::is_nan(t_w) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(v) || pmath::is_nan(v) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(m) || pmath::is_nan(m) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(kn) || pmath::is_nan(kn) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::aero::cylinder_drag_coefficient(
            s,
            vleo_units::Length::new(l),
            vleo_units::Length::new(d),
            alpha,
            vleo_units::Temperature::new(t_w),
            vleo_units::Velocity::new(v),
            vleo_units::MolarMass::new(m),
        )),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
