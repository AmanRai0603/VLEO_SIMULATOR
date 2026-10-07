//! GENERATED from the method of `l3_solar_ach_02` by `cargo xtask docs`, translated by
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

/// The method of `l3_solar_ach_02`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/l3_solar_ach_02/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// #
/// # What the code said beside it:
/// # The requirement declares sense "<=", so the achieved value must stay UNDER
/// # the bound and the margin is (required - achieved) / required. Positive is
/// # room; negative is a violation and its size. mission::closure is the twelve
/// # KPI closures' own function rather than the arithmetic written out again,
/// # because a second way of computing a margin is a second way of getting its
/// # sign wrong -- and a sign error here still produces a plausible number.
/// return margin_at_most(req, ach)
/// ```
pub fn evaluate(ach: f64, req: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(ach) || pmath::is_nan(ach) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(req) || pmath::is_nan(req) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::mission::closure(
            req,
            ach,
            vleo_core::physics::mission::Sense::AtMost,
        )
        .margin),
        11,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
