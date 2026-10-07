//! GENERATED from the method of `thm_required_radiator_area` by `cargo xtask docs`, translated by
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

/// The method of `thm_required_radiator_area`, source `larson_wertz`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-thermal/nodes/thm_required_radiator_area/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 250 [K].
/// return required_radiator_area(q, e, lim, 250 [K])
/// ```
pub fn evaluate(q: f64, e: f64, lim: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(q) || pmath::is_nan(q) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(e) || pmath::is_nan(e) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(lim) || pmath::is_nan(lim) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::thermal::required_radiator_area(
            vleo_units::Power::new(q),
            vleo_units::Ratio::new(e),
            vleo_units::Temperature::new(lim),
            vleo_units::Temperature::new(250.0),
        )
        .get()),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
