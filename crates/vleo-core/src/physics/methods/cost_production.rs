//! GENERATED from the method of `cost_production` by `cargo xtask docs`, translated by
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

/// The method of `cost_production`, source `nasa_cer`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-closure/nodes/cost_production/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return production_run_cost(cb + cp, n, b)
/// ```
pub fn evaluate(cb: f64, cp: f64, n: f64, b: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(cb) || pmath::is_nan(cb) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(cp) || pmath::is_nan(cp) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(n) || pmath::is_nan(n) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(b) || pmath::is_nan(b) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::cost::production_run_cost(
            vleo_units::Money::new((cb + cp)),
            (n) as u32,
            b,
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
