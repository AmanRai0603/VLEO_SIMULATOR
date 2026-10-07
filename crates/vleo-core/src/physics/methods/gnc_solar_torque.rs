//! GENERATED from the method of `gnc_solar_torque` by `cargo xtask docs`, translated by
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

/// The method of `gnc_solar_torque`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-acs/nodes/gnc_solar_torque/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 0.6.
/// return solar_pressure_torque(a, 0.6, x, th)
/// ```
pub fn evaluate(a: f64, x: f64, th: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(a) || pmath::is_nan(a) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(x) || pmath::is_nan(x) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(th) || pmath::is_nan(th) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::gnc::solar_pressure_torque(
            vleo_units::Area::new(a),
            vleo_units::Ratio::new(0.6),
            vleo_units::Length::new(x),
            vleo_units::Angle::new(th),
        )
        .get()),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
