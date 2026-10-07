//! GENERATED from the method of `prop_throttle` by `cargo xtask docs`, translated by
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

/// The method of `prop_throttle`, source `orbitt_case_c1`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-prop/nodes/prop_throttle/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 1.
/// # Whatever the other loads leave of the power available goes to the thruster.
/// let other = (pay + ax + cm + th) * (1 + lh)
/// let spare = max(0, av - other)
/// if pp <= 0 [W] then
///   return 0
/// end
/// return min(1, spare / pp)
/// ```
pub fn evaluate(
    av: f64,
    pp: f64,
    pay: f64,
    ax: f64,
    cm: f64,
    th: f64,
    lh: f64,
) -> Result<f64, MethodError> {
    if !pmath::is_finite(av) || pmath::is_nan(av) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(pp) || pmath::is_nan(pp) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(pay) || pmath::is_nan(pay) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ax) || pmath::is_nan(ax) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(cm) || pmath::is_nan(cm) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(th) || pmath::is_nan(th) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(lh) || pmath::is_nan(lh) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let other: f64 = rt::fin(((((pay + ax) + cm) + th) * (1.0 + lh)), 5)?;
    let spare: f64 = rt::fin(pmath::max(0.0, (av - other)), 6)?;
    if (pp <= 0.0) {
        return Ok(rt::fin(0.0, 8)?);
    }
    return Ok(rt::fin(pmath::min(1.0, rt::div(spare, pp, 10)?), 10)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
