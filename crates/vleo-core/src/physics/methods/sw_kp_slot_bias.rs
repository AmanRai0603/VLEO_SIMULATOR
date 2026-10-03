//! GENERATED from the method of `sw_kp_slot_bias` by `cargo xtask docs`, translated by
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

/// The method of `sw_kp_slot_bias`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_kp_slot_bias/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: vleo_core::physics::env::kp_peak_slot_bias — median of max_8(Kp) - kp_from_ap(Ap) in each of prf_ap2kp's bins (centres of edges 0 5 10 15 20 30 45 70 110 400), measured on bundles/solar-weather@2026.09.14; held at the end bins, never extrapolated.
/// # The table clamps at both ends, so the only input the code refuses is one that is not a number:
/// # it makes the answer not a number, which the generated guard refuses.
/// if not (ap == ap) then
///   refuse "the computation produced a value that is not a number"
/// end
/// let off = interp(ap, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0] [1], [1.0, 0.8333333333333333, 1.1144067796610169, 1.0, 1.2, 1.3333333333333333, 1.5454545454545454, 1.7469135802469136, 1.3174603174603174] [1])
/// if off < 0 then
///   refuse "a negative correction contradicts Jensen's inequality: the sign or the slot has been swapped"
/// end
/// if off > 2 then
///   refuse "above 2.0 is unreachable by this clamped table, so the table is broken"
/// end
/// return off
/// ```
pub fn evaluate(ap: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(ap) || pmath::is_nan(ap) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if (!(ap == ap)) {
        return Err(MethodError::Refused(
            "the computation produced a value that is not a number",
        ));
    }
    let off: f64 = rt::fin(
        pmath::interp(
            ap,
            &[2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0],
            &[
                1.0,
                0.8333333333333333,
                1.1144067796610169,
                1.0,
                1.2,
                1.3333333333333333,
                1.5454545454545454,
                1.7469135802469136,
                1.3174603174603174,
            ],
        ),
        8,
    )?;
    if (off < 0.0) {
        return Err(MethodError::Refused("a negative correction contradicts Jensen's inequality: the sign or the slot has been swapped"));
    }
    if (off > 2.0) {
        return Err(MethodError::Refused(
            "above 2.0 is unreachable by this clamped table, so the table is broken",
        ));
    }
    return Ok(rt::fin(off, 15)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
