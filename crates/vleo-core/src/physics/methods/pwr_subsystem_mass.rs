//! GENERATED from the method of `pwr_subsystem_mass` by `cargo xtask docs`, translated by
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

/// The method of `pwr_subsystem_mass`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-power/nodes/pwr_subsystem_mass/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 1.25.
/// return (ma + mb) * 1.25
/// ```
pub fn evaluate(ma: f64, mb: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(ma) || pmath::is_nan(ma) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(mb) || pmath::is_nan(mb) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(((ma + mb) * 1.25), 4)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
