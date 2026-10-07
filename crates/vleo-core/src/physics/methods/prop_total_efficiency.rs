//! GENERATED from the method of `prop_total_efficiency` by `cargo xtask docs`, translated by
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

/// The method of `prop_total_efficiency`, source `romano2021`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-prop/nodes/prop_total_efficiency/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return total_efficiency(pj, pb)
/// ```
pub fn evaluate(pj: f64, pb: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(pj) || pmath::is_nan(pj) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(pb) || pmath::is_nan(pb) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::prop::total_efficiency(
            vleo_units::Power::new(pj),
            vleo_units::Power::new(pb),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
