//! GENERATED from the method of `sw_kp_from_ap` by `cargo xtask docs`, translated by
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

/// The method of `sw_kp_from_ap`, source `iaga_kp_ap`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_kp_from_ap/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: the IAGA published Kp-ap scale (iaga_kp_ap), 28 pairs, Kp in thirds 0..9 — vleo_core::physics::env::kp_from_ap (AP_AT_KP_THIRDS), via prf_ap2kp.m:100.
/// # kp_from_ap is linear interpolation held at both ends (pmath::interp): below ap 0 and above ap 400 the scale does not continue.
/// # Kp values are the thirds the kernel writes as k/3.0, given here as their exact double values.
/// # The generated "not a number" guard, moved to the input: the table is finite and held at both ends, so the answer is NaN exactly when ap is.
/// if not (ap == ap) then
///   refuse "the computation produced a value that is not a number"
/// end
/// let k = interp(ap, [0, 2, 3, 4, 5, 6, 7, 9, 12, 15, 18, 22, 27, 32, 39, 48, 56, 67, 80, 94, 111, 132, 154, 179, 207, 236, 300, 400] [1], [0.0, 0.3333333333333333, 0.6666666666666666, 1.0, 1.3333333333333333, 1.6666666666666667, 2.0, 2.3333333333333335, 2.6666666666666665, 3.0, 3.3333333333333335, 3.6666666666666665, 4.0, 4.333333333333333, 4.666666666666667, 5.0, 5.333333333333333, 5.666666666666667, 6.0, 6.333333333333333, 6.666666666666667, 7.0, 7.333333333333333, 7.666666666666667, 8.0, 8.333333333333334, 8.666666666666666, 9.0] [1])
/// if k < 0 then
///   refuse "Kp is defined on 0..9: a negative index is a sign error, not a quiet day"
/// end
/// if k > 9 then
///   refuse "Kp is defined on 0..9: an ap value has reached a consumer that wanted Kp"
/// end
/// return k
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
    let k: f64 = rt::fin(
        pmath::interp(
            ap,
            &[
                0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 12.0, 15.0, 18.0, 22.0, 27.0, 32.0, 39.0,
                48.0, 56.0, 67.0, 80.0, 94.0, 111.0, 132.0, 154.0, 179.0, 207.0, 236.0, 300.0,
                400.0,
            ],
            &[
                0.0,
                0.3333333333333333,
                0.6666666666666666,
                1.0,
                1.3333333333333333,
                1.6666666666666667,
                2.0,
                2.3333333333333335,
                2.6666666666666665,
                3.0,
                3.3333333333333335,
                3.6666666666666665,
                4.0,
                4.333333333333333,
                4.666666666666667,
                5.0,
                5.333333333333333,
                5.666666666666667,
                6.0,
                6.333333333333333,
                6.666666666666667,
                7.0,
                7.333333333333333,
                7.666666666666667,
                8.0,
                8.333333333333334,
                8.666666666666666,
                9.0,
            ],
        ),
        9,
    )?;
    if (k < 0.0) {
        return Err(MethodError::Refused(
            "Kp is defined on 0..9: a negative index is a sign error, not a quiet day",
        ));
    }
    if (k > 9.0) {
        return Err(MethodError::Refused(
            "Kp is defined on 0..9: an ap value has reached a consumer that wanted Kp",
        ));
    }
    return Ok(rt::fin(k, 16)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
