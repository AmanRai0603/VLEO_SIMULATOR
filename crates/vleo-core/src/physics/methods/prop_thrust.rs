//! GENERATED from the method of `prop_thrust` by `cargo xtask docs`, translated by
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

/// The method of `prop_thrust`, source `romano2021`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-prop/nodes/prop_thrust/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return beam_thrust(mi, ve, ad, ap)
/// ```
pub fn evaluate(mi: f64, ve: f64, ad: f64, ap: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(mi) || pmath::is_nan(mi) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ve) || pmath::is_nan(ve) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ad) || pmath::is_nan(ad) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ap) || pmath::is_nan(ap) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::prop::beam_thrust(
            vleo_units::MassFlow::new(mi),
            vleo_units::Velocity::new(ve),
            vleo_units::Ratio::new(ad),
            vleo_units::Ratio::new(ap),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
