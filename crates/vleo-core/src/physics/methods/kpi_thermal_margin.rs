//! GENERATED from the method of `kpi_thermal_margin` by `cargo xtask docs`, translated by
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

/// The method of `kpi_thermal_margin`, source `ecss_e_st_10_02`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-closure/nodes/kpi_thermal_margin/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return margin_at_least(req, ach)
/// ```
pub fn evaluate(req: f64, ach: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(req) || pmath::is_nan(req) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ach) || pmath::is_nan(ach) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::mission::closure(
            req,
            ach,
            vleo_core::physics::mission::Sense::AtLeast,
        )
        .margin),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
