//! GENERATED from the method of `gnc_momentum_storage` by `cargo xtask docs`, translated by
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

/// The method of `gnc_momentum_storage`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-acs/nodes/gnc_momentum_storage/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 0.5.
/// return momentum_storage_required(t, p, 0.5)
/// ```
pub fn evaluate(t: f64, p: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(t) || pmath::is_nan(t) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(p) || pmath::is_nan(p) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::gnc::momentum_storage_required(
            vleo_units::Torque::new(t),
            vleo_units::Time::new(p),
            vleo_units::Ratio::new(0.5),
        )
        .get()),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
