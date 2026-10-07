//! GENERATED from the method of `mass_disposal_propellant` by `cargo xtask docs`, translated by
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

/// The method of `mass_disposal_propellant`, source `iso24113`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-massaero/nodes/mass_disposal_propellant/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 220 [s].
/// return propellant_mass(m, dv, exhaust_velocity(220 [s]))
/// ```
pub fn evaluate(m: f64, dv: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(m) || pmath::is_nan(m) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(dv) || pmath::is_nan(dv) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::orbit::propellant_mass(
            vleo_units::Mass::new(m),
            vleo_units::Velocity::new(dv),
            vleo_units::Velocity::new(
                (vleo_core::physics::orbit::exhaust_velocity(vleo_units::Time::new(220.0)).get()),
            ),
        )
        .get()),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
