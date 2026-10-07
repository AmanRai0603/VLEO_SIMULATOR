//! GENERATED from the method of `pwr_demand` by `cargo xtask docs`, translated by
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

/// The method of `pwr_demand`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-power/nodes/pwr_demand/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return power_demand(pp, pay, av, com, th, lh)
/// ```
pub fn evaluate(
    pp: f64,
    pay: f64,
    av: f64,
    com: f64,
    th: f64,
    lh: f64,
) -> Result<f64, MethodError> {
    if !pmath::is_finite(pp) || pmath::is_nan(pp) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(pay) || pmath::is_nan(pay) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(av) || pmath::is_nan(av) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(com) || pmath::is_nan(com) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(th) || pmath::is_nan(th) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(lh) || pmath::is_nan(lh) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::power::power_demand(
            vleo_units::Power::new(pp),
            vleo_units::Power::new(pay),
            vleo_units::Power::new(av),
            vleo_units::Power::new(com),
            vleo_units::Power::new(th),
            vleo_units::Ratio::new(lh),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
