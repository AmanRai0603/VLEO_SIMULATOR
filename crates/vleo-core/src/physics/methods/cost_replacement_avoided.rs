//! GENERATED from the method of `cost_replacement_avoided` by `cargo xtask docs`, translated by
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

/// The method of `cost_replacement_avoided`, source `orbitt_case_c1`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-closure/nodes/cost_replacement_avoided/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 1.
/// # Replacements avoided only where the design holds its orbit (thrust at least drag).
/// let replacements = 0
/// if td >= 1 then
///   set replacements = n
/// end
/// return replacement_avoided(cb + cp, lc, replacements)
/// ```
pub fn evaluate(cb: f64, cp: f64, lc: f64, n: f64, td: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(cb) || pmath::is_nan(cb) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(cp) || pmath::is_nan(cp) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(lc) || pmath::is_nan(lc) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(n) || pmath::is_nan(n) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(td) || pmath::is_nan(td) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let mut replacements: f64 = rt::fin(0.0, 5)?;
    if (td >= 1.0) {
        replacements = rt::fin(n, 7)?;
    }
    return Ok(rt::fin(
        (vleo_core::physics::cost::replacement_avoided(
            vleo_units::Money::new((cb + cp)),
            vleo_units::Money::new(lc),
            replacements,
        )
        .get()),
        9,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
