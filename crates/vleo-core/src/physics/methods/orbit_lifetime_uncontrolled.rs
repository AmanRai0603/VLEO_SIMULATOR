//! GENERATED from the method of `orbit_lifetime_uncontrolled` by `cargo xtask docs`, translated by
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

/// The method of `orbit_lifetime_uncontrolled`, source `vallado2013`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-envorbit/nodes/orbit_lifetime_uncontrolled/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # "* 1 [unit]" gives a number this design states as a pure ratio the unit the kernel reads it in,
/// # and "/ 1 [unit]" the other way; the number itself does not change.
/// # Constants the code held, written out here as it held them: 120 [km].
/// return orbit_decay_time(h, 120 [km], bc * 1 [kg/m^2], t_inf)
/// ```
pub fn evaluate(h: f64, bc: f64, t_inf: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(h) || pmath::is_nan(h) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(bc) || pmath::is_nan(bc) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(t_inf) || pmath::is_nan(t_inf) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::orbit::lifetime_estimate(
            vleo_units::Length::new(h),
            vleo_units::Length::new(120000.0),
            (bc * 1.0),
            64,
            |z| vleo_core::physics::env::mass_density(z, vleo_units::Temperature::new(t_inf)),
        )
        .get()),
        6,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
