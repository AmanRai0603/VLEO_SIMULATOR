//! GENERATED from the method of `prop_capture_efficiency` by `cargo xtask docs`, translated by
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

/// The method of `prop_capture_efficiency`, source `romano2021`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-prop/nodes/prop_capture_efficiency/model.rs (its HOLE): the relation the code ran, line for line.
/// # The relation itself stays in the kernel; this says which one, with which inputs, in which units.
/// return intake_collection_efficiency(n, v, a_in, a_out, eta_geo, beta, t_c, m)
/// ```
pub fn evaluate(
    a_in: f64,
    a_out: f64,
    eta_geo: f64,
    beta: f64,
    t_c: f64,
    m: f64,
    n: f64,
    v: f64,
) -> Result<f64, MethodError> {
    if !pmath::is_finite(a_in) || pmath::is_nan(a_in) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(a_out) || pmath::is_nan(a_out) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(eta_geo) || pmath::is_nan(eta_geo) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(beta) || pmath::is_nan(beta) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(t_c) || pmath::is_nan(t_c) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(m) || pmath::is_nan(m) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(n) || pmath::is_nan(n) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(v) || pmath::is_nan(v) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    return Ok(rt::fin(
        (vleo_core::physics::prop::intake_balance(
            vleo_units::NumberDensity::new(n),
            vleo_units::Velocity::new(v),
            vleo_units::Area::new(a_in),
            vleo_units::Area::new(a_out),
            vleo_units::Ratio::new(eta_geo),
            vleo_units::Ratio::new(beta),
            vleo_units::Temperature::new(t_c),
            vleo_units::MolarMass::new(m),
        )
        .collection_efficiency
        .get()),
        3,
    )?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
