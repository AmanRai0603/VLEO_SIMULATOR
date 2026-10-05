//! GENERATED from the method of `sw_uncertainty_growth` by `cargo xtask docs`, translated by
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

/// The method of `sw_uncertainty_growth`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_uncertainty_growth/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: seventeen leads and their measured 95th-percentile F10.7 growth, from solar-weather@2026.09.14 (noaa_swpc), pairs of observed days only.
/// # The table clamps at both ends, so the only input the code refuses is one that is not a number:
/// # it makes the answer not a number, which the generated guard refuses.
/// if not (lead == lead) then
///   refuse "the computation produced a value that is not a number"
/// end
/// let growth = interp(lead, [183, 365, 548, 730, 1096, 1461, 1826, 2191, 2557, 2922, 3287, 3653, 4018, 4383, 4748, 5113, 5478] [d], [57.0, 68.3, 77.0, 91.0, 107.0, 114.0, 113.3, 104.0, 92.0, 88.0, 78.0, 66.0, 66.0, 76.0, 91.0, 106.0, 115.0] [1])
/// if growth < 0 then
///   refuse "a negative 95th-percentile growth means the difference was taken the wrong way round"
/// end
/// if growth > 120 then
///   refuse "above 120 sfu is unreachable by this clamped table, so the table is broken"
/// end
/// return growth
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
    let growth: f64 = rt::fin(
        pmath::interp(
            lead,
            &[
                15811200.0,
                31536000.0,
                47347200.0,
                63072000.0,
                94694400.0,
                126230400.0,
                157766400.0,
                189302400.0,
                220924800.0,
                252460800.0,
                283996800.0,
                315619200.0,
                347155200.0,
                378691200.0,
                410227200.0,
                441763200.0,
                473299200.0,
            ],
            &[
                57.0, 68.3, 77.0, 91.0, 107.0, 114.0, 113.3, 104.0, 92.0, 88.0, 78.0, 66.0, 66.0,
                76.0, 91.0, 106.0, 115.0,
            ],
        ),
        8,
    )?;
    if (growth < 0.0) {
        return Err(MethodError::Refused(
            "a negative 95th-percentile growth means the difference was taken the wrong way round",
        ));
    }
    if (growth > 120.0) {
        return Err(MethodError::Refused(
            "above 120 sfu is unreachable by this clamped table, so the table is broken",
        ));
    }
    return Ok(rt::fin(growth, 15)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
