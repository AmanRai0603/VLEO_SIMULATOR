//! GENERATED from the method of `sw_ap_design` by `cargo xtask docs`, translated by
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

/// The method of `sw_ap_design`, source `iaga_kp_ap`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_ap_design/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # The published ap equivalent amplitude at the top Kp of each G band (iaga_kp_ap): G1 is Kp 5 is ap 48, G2 is Kp 6 is ap 80, G3 is Kp 7 is ap 132.
/// # A lookup on three integer levels, not an interpolation: the ap scale is close to geometric.
/// let out = 132
/// if g_level < 1.5 then
///   set out = 48
/// else if g_level < 2.5 then
///   set out = 80
/// end
/// if out < 40 then
///   refuse "Designing to less than a minor storm (G1, Ap 48) is not a design case."
/// end
/// if out > 140 then
///   refuse "Above G3's Ap 132 means a G level outside the range or a misread table."
/// end
/// return out
/// ```
pub fn evaluate(g_level: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(g_level) || pmath::is_nan(g_level) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let mut out: f64 = rt::fin(132.0, 4)?;
    if (g_level < 1.5) {
        out = rt::fin(48.0, 6)?;
    } else if (g_level < 2.5) {
        out = rt::fin(80.0, 8)?;
    }
    if (out < 40.0) {
        return Err(MethodError::Refused(
            "Designing to less than a minor storm (G1, Ap 48) is not a design case.",
        ));
    }
    if (out > 140.0) {
        return Err(MethodError::Refused(
            "Above G3's Ap 132 means a G level outside the range or a misread table.",
        ));
    }
    return Ok(rt::fin(out, 16)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
