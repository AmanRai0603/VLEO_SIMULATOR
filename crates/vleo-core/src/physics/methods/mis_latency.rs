//! GENERATED from the method of `mis_latency` by `cargo xtask docs`, translated by
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

/// The method of `mis_latency`, source `orbitt_case_c1`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-envorbit/nodes/mis_latency/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return end_to_end_latency(tw, td, tp, tdl)
/// ```
pub fn evaluate(tw: f64, td: f64, tp: f64, tdl: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(tw) || pmath::is_nan(tw) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(td) || pmath::is_nan(td) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(tp) || pmath::is_nan(tp) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(tdl) || pmath::is_nan(tdl) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::mission::end_to_end_latency(
            vleo_units::Time::new(tw),
            vleo_units::Time::new(td),
            vleo_units::Time::new(tp),
            vleo_units::Time::new(tdl),
        )
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
