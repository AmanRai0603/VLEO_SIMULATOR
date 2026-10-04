//! GENERATED from the method of `example_orbit_speed` by `cargo xtask docs`, translated by
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

/// The method of `example_orbit_speed`, source `vallado2013`:
///
/// ```text
/// # Vallado (2013), eq. 1-18: the speed of a circular two-body orbit.
/// if r <= R_EARTH then
///   refuse "the orbit is inside the Earth"
/// end
/// let v : Velocity = sqrt(MU_EARTH / r)
/// return v
/// ```
pub fn evaluate(r: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(r) || pmath::is_nan(r) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if (r <= R_EARTH.get()) {
        return Err(MethodError::Refused("the orbit is inside the Earth"));
    }
    let v: f64 = rt::fin(rt::sqrt(rt::div(MU_EARTH, r, 5)?, 5)?, 5)?;
    return Ok(rt::fin(v, 6)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
