//! GENERATED from the method of `gnc_along_track_error` by `cargo xtask docs`, translated by
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

/// The method of `gnc_along_track_error`, source `doornbos2011`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-acs/nodes/gnc_along_track_error/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 1 [d].
/// return along_track_error_from_drag(a * s, 1 [d])
/// ```
pub fn evaluate(a: f64, s: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(a) || pmath::is_nan(a) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(s) || pmath::is_nan(s) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::gnc::along_track_error_from_drag(
            vleo_units::Acceleration::new((a * s)),
            vleo_units::Time::new(86400.0),
        )
        .get()),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
