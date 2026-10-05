//! GENERATED from the method of `sw_forecast_bias` by `cargo xtask docs`, translated by
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

/// The method of `sw_forecast_bias`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_forecast_bias/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: NOAA SWPC (noaa_swpc) issued 27-day outlook verified against observed F10.7, 1997-2025, mean signed error per lead (bundle solar-weather).
/// # The measured table, lead 1 to 26 days; linear between leads, held at both ends (as Table1 does).
/// # The generated "not a number" guard, moved to the input: the table is finite and held at both ends, so the answer is NaN exactly when the lead is.
/// if not (lead == lead) then
///   refuse "the computation produced a value that is not a number"
/// end
/// let out = interp(lead, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26] [d], [-0.5930701048, -0.7977346278, -1.2774613507, -1.7390243902, -1.9780309194, -2.4073190135, -2.6042173560, -2.5690072639, -2.7477840451, -2.4782258065, -2.1673403395, -1.9443099274, -1.8480194018, -1.9268292683, -2.3772522523, -2.3595890411, -2.2710706150, -2.2514285714, -2.0547320410, -2.1760000000, -2.2803203661, -2.5414746544, -2.9447640967, -3.3091118800, -3.7575057737, -3.9677419355] [1])
/// if out < -4 then
///   refuse "below the deepest measured bias of -3.968 sfu at lead 26: the table was misread or the bundle changed"
/// end
/// if out > -0.5 then
///   refuse "above the shallowest measured bias of -0.593 sfu at lead 1: the record shows no over-forecast at any lead"
/// end
/// return out
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
    let out: f64 = rt::fin(
        pmath::interp(
            lead,
            &[
                86400.0, 172800.0, 259200.0, 345600.0, 432000.0, 518400.0, 604800.0, 691200.0,
                777600.0, 864000.0, 950400.0, 1036800.0, 1123200.0, 1209600.0, 1296000.0,
                1382400.0, 1468800.0, 1555200.0, 1641600.0, 1728000.0, 1814400.0, 1900800.0,
                1987200.0, 2073600.0, 2160000.0, 2246400.0,
            ],
            &[
                (-0.5930701048),
                (-0.7977346278),
                (-1.2774613507),
                (-1.7390243902),
                (-1.9780309194),
                (-2.4073190135),
                (-2.604217356),
                (-2.5690072639),
                (-2.7477840451),
                (-2.4782258065),
                (-2.1673403395),
                (-1.9443099274),
                (-1.8480194018),
                (-1.9268292683),
                (-2.3772522523),
                (-2.3595890411),
                (-2.271070615),
                (-2.2514285714),
                (-2.054732041),
                (-2.176),
                (-2.2803203661),
                (-2.5414746544),
                (-2.9447640967),
                (-3.30911188),
                (-3.7575057737),
                (-3.9677419355),
            ],
        ),
        8,
    )?;
    if (out < (-4.0)) {
        return Err(MethodError::Refused("below the deepest measured bias of -3.968 sfu at lead 26: the table was misread or the bundle changed"));
    }
    if (out > (-0.5)) {
        return Err(MethodError::Refused("above the shallowest measured bias of -0.593 sfu at lead 1: the record shows no over-forecast at any lead"));
    }
    return Ok(rt::fin(out, 15)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
