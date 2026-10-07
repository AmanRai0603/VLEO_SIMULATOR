//! GENERATED from the method of `thm_aero_heating` by `cargo xtask docs`, translated by
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

/// The method of `thm_aero_heating`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-thermal/nodes/thm_aero_heating/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 0.9.
/// return free_molecular_heating(rho, v, a, 0.9)
/// ```
pub fn evaluate(rho: f64, v: f64, a: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(rho) || pmath::is_nan(rho) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(v) || pmath::is_nan(v) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(a) || pmath::is_nan(a) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::thermal::free_molecular_heating(
            vleo_units::MassDensity::new(rho),
            vleo_units::Velocity::new(v),
            vleo_units::Area::new(a),
            vleo_units::Ratio::new(0.9),
        )
        .get()),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
