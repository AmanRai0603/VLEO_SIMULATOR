//! GENERATED from the method of `com_cn0` by `cargo xtask docs`, translated by
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

/// The method of `com_cn0`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-ttc/nodes/com_cn0/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return carrier_to_noise_density_db(e, lf, la, gt)
/// ```
pub fn evaluate(e: f64, lf: f64, la: f64, gt: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(e) || pmath::is_nan(e) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(lf) || pmath::is_nan(lf) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(la) || pmath::is_nan(la) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(gt) || pmath::is_nan(gt) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::comms::carrier_to_noise_density_db(e, lf, la, gt)),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
