//! GENERATED from the method of `gnc_gravity_gradient_torque` by `cargo xtask docs`, translated by
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

/// The method of `gnc_gravity_gradient_torque`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-acs/nodes/gnc_gravity_gradient_torque/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # "* 1 [unit]" gives a number this design states as a pure ratio the unit the kernel reads it in,
/// # and "/ 1 [unit]" the other way; the number itself does not change.
/// # Constants the code held, written out here as it held them: 10 [deg].
/// return gravity_gradient_torque(r, imax * 1 [kg.m^2], imin * 1 [kg.m^2], 10 [deg])
/// ```
pub fn evaluate(r: f64, imax: f64, imin: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(r) || pmath::is_nan(r) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(imax) || pmath::is_nan(imax) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(imin) || pmath::is_nan(imin) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::gnc::gravity_gradient_torque(
            vleo_units::Length::new(r),
            (imax * 1.0),
            (imin * 1.0),
            vleo_units::Angle::new(0.17453292519943295),
        )
        .get()),
        6,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
