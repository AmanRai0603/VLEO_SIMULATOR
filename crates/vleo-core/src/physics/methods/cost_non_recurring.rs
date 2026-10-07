//! GENERATED from the method of `cost_non_recurring` by `cargo xtask docs`, translated by
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

/// The method of `cost_non_recurring`, source `nasa_cer`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-closure/nodes/cost_non_recurring/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: a = 14.2; b = 0.55; base year 2020; 2026.
/// return cer_inflated(m, 14.2, 0.55, 2020, 2026, i)
/// ```
pub fn evaluate(m: f64, i: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(m) || pmath::is_nan(m) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(i) || pmath::is_nan(i) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::cost::Cer {
            a: 14.2,
            b: 0.55,
            base_year: (2020.0) as u16,
            sigma: 0.0,
        }
        .evaluate_inflated(m, (2026.0) as u16, i)
        .get()),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
