//! GENERATED from the method of `prop_delivered_bus_power` by `cargo xtask docs`, translated by
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

/// The method of `prop_delivered_bus_power`, source `orbitt_case_c1`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-prop/nodes/prop_delivered_bus_power/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return pp * k
/// ```
pub fn evaluate(pp: f64, k: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(pp) || pmath::is_nan(pp) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(k) || pmath::is_nan(k) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin((pp * k), 3)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
