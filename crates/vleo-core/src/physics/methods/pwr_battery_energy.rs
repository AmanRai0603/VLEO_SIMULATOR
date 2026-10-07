//! GENERATED from the method of `pwr_battery_energy` by `cargo xtask docs`, translated by
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

/// The method of `pwr_battery_energy`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-power/nodes/pwr_battery_energy/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return battery_energy_required(dem, fe * t, dod, ed)
/// ```
pub fn evaluate(dem: f64, fe: f64, t: f64, dod: f64, ed: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(dem) || pmath::is_nan(dem) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(fe) || pmath::is_nan(fe) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(t) || pmath::is_nan(t) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(dod) || pmath::is_nan(dod) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ed) || pmath::is_nan(ed) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::power::battery_energy_required(
            vleo_units::Power::new(dem),
            vleo_units::Time::new((fe * t)),
            vleo_units::Ratio::new(dod),
            vleo_units::Ratio::new(ed),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
