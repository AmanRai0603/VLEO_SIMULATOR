//! GENERATED from the method of `sw_horizon_climatology` by `cargo xtask docs`, translated by
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

/// The method of `sw_horizon_climatology`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_horizon_climatology/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: NOAA SWPC (noaa_swpc) F10.7 record, RMS departure of the target day from the record mean 114.8437 sfu, from solar-weather@2026.09.14.
/// # The eighteen measured leads in days; linear between them, held at both ends (as Table1 does).
/// # The generated "not a number" guard, moved to the input: the table is finite and held at both ends, so the answer is NaN exactly when the lead is.
/// if not (lead == lead) then
///   refuse "the computation produced a value that is not a number"
/// end
/// let err = interp(lead, [1, 2, 3, 5, 7, 10, 14, 20, 27, 40, 60, 90, 135, 180, 270, 365, 547, 730] [d], [44.3929, 44.3953, 44.3968, 44.3993, 44.4012, 44.4036, 44.4069, 44.4071, 44.4147, 44.4242, 44.4344, 44.4458, 44.4635, 44.4714, 44.5344, 44.6734, 45.0333, 45.3201] [1])
/// if err < 0 then
///   refuse "an RMS cannot be negative, and this one is the spread of the record about its own mean, 44 sfu"
/// end
/// if err > 50 then
///   refuse "the measured entries run 44.39 to 45.32 sfu and the table clamps: above 50 the table is broken"
/// end
/// return err
/// ```
pub fn evaluate(lead: f64) -> Result<f64, MethodError> {
    if !pmath::is_finite(lead) || pmath::is_nan(lead) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if (!(lead == lead)) {
        return Err(MethodError::Refused(
            "the computation produced a value that is not a number",
        ));
    }
    let err: f64 = rt::fin(
        pmath::interp(
            lead,
            &[
                86400.0, 172800.0, 259200.0, 432000.0, 604800.0, 864000.0, 1209600.0, 1728000.0,
                2332800.0, 3456000.0, 5184000.0, 7776000.0, 11664000.0, 15552000.0, 23328000.0,
                31536000.0, 47260800.0, 63072000.0,
            ],
            &[
                44.3929, 44.3953, 44.3968, 44.3993, 44.4012, 44.4036, 44.4069, 44.4071, 44.4147,
                44.4242, 44.4344, 44.4458, 44.4635, 44.4714, 44.5344, 44.6734, 45.0333, 45.3201,
            ],
        ),
        8,
    )?;
    if (err < 0.0) {
        return Err(MethodError::Refused("an RMS cannot be negative, and this one is the spread of the record about its own mean, 44 sfu"));
    }
    if (err > 50.0) {
        return Err(MethodError::Refused("the measured entries run 44.39 to 45.32 sfu and the table clamps: above 50 the table is broken"));
    }
    return Ok(rt::fin(err, 15)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
