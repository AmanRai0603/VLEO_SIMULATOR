//! GENERATED from the method of `pwr_battery_cycles` by `cargo xtask docs`, translated by
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

/// The method of `pwr_battery_cycles`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-power/nodes/pwr_battery_cycles/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return battery_cycles(tm, t_orbit)
/// ```
pub fn evaluate(tm: f64, t_orbit: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(tm) || pmath::is_nan(tm) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(t_orbit) || pmath::is_nan(t_orbit) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::power::battery_cycles(
            vleo_units::Time::new(tm),
            vleo_units::Time::new(t_orbit),
        )),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
