//! GENERATED from the method of `thm_equilibrium_temperature` by `cargo xtask docs`, translated by
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

/// The method of `thm_equilibrium_temperature`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-thermal/nodes/thm_equilibrium_temperature/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return equilibrium_temperature(qs + qa + qi + qh, qd, ar, e)
/// ```
pub fn evaluate(
    qs: f64,
    qa: f64,
    qi: f64,
    qh: f64,
    qd: f64,
    ar: f64,
    e: f64,
) -> Result<f64, MethodError> {
    if !pmath::is_finite(qs) || pmath::is_nan(qs) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(qa) || pmath::is_nan(qa) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(qi) || pmath::is_nan(qi) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(qh) || pmath::is_nan(qh) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(qd) || pmath::is_nan(qd) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ar) || pmath::is_nan(ar) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(e) || pmath::is_nan(e) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::thermal::equilibrium_temperature(
            vleo_units::Power::new((((qs + qa) + qi) + qh)),
            vleo_units::Power::new(qd),
            vleo_units::Area::new(ar),
            vleo_units::Ratio::new(e),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
