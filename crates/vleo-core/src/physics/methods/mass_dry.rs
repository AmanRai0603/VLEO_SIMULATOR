//! GENERATED from the method of `mass_dry` by `cargo xtask docs`, translated by
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

/// The method of `mass_dry`, source `ecss_e_st_10_02`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-massaero/nodes/mass_dry/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// # Constants the code held, written out here as it held them: 0 [kg] for thermal: the thermal hardware is carried in the harness line, as the code carried it.
/// return dry_mass(st, pr, pw, 0 [kg], av, cm, gn, th, pa, mg)
/// ```
pub fn evaluate(
    st: f64,
    pr: f64,
    pw: f64,
    th: f64,
    av: f64,
    cm: f64,
    gn: f64,
    pa: f64,
    mg: f64,
) -> Result<f64, MethodError> {
    if !pmath::is_finite(st) || pmath::is_nan(st) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(pr) || pmath::is_nan(pr) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(pw) || pmath::is_nan(pw) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(th) || pmath::is_nan(th) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(av) || pmath::is_nan(av) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(cm) || pmath::is_nan(cm) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(gn) || pmath::is_nan(gn) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(pa) || pmath::is_nan(pa) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(mg) || pmath::is_nan(mg) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::mass::dry_mass(
            vleo_units::Mass::new(st),
            vleo_units::Mass::new(pr),
            vleo_units::Mass::new(pw),
            vleo_units::Mass::new(0.0),
            vleo_units::Mass::new(av),
            vleo_units::Mass::new(cm),
            vleo_units::Mass::new(gn),
            vleo_units::Mass::new(th),
            vleo_units::Mass::new(pa),
            vleo_units::Ratio::new(mg),
        )
        .get()),
        4,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
