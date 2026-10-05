//! GENERATED from the method of `sw_forecast_skill` by `cargo xtask docs`, translated by
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

/// The method of `sw_forecast_skill`, source `noaa_swpc`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_forecast_skill/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: the measured skill of the issued 27-day F10.7 outlook against persistence, leads 1 to 26 days (noaa_swpc record).
/// # The table clamps at both ends, so the only input the code refuses is one that is not a number:
/// # it makes the answer not a number, which the generated guard refuses.
/// if not (lead == lead) then
///   refuse "the computation produced a value that is not a number"
/// end
/// let out = interp(lead, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26] [d], [0.0685159110, 0.1988398832, 0.2248475213, 0.2138431310, 0.2768782923, 0.3179547804, 0.3625942984, 0.4020897345, 0.4375371731, 0.4325518344, 0.4188543510, 0.4137682014, 0.4016285468, 0.4186771205, 0.3918289086, 0.3698739969, 0.3041719673, 0.2475268244, 0.2117196579, 0.1696147673, 0.1404618270, 0.0784955060, 0.0176508866, -0.0355195984, -0.0324460638, -0.0220863078] [1])
/// if out < -0.1 then
///   refuse "a skill below -0.1 says the outlook is substantially worse than persistence, which the record does not support"
/// end
/// if out > 0.5 then
///   refuse "a skill above 0.5 against persistence is beyond anything in the record, so the table was misread"
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
                0.068515911,
                0.1988398832,
                0.2248475213,
                0.213843131,
                0.2768782923,
                0.3179547804,
                0.3625942984,
                0.4020897345,
                0.4375371731,
                0.4325518344,
                0.418854351,
                0.4137682014,
                0.4016285468,
                0.4186771205,
                0.3918289086,
                0.3698739969,
                0.3041719673,
                0.2475268244,
                0.2117196579,
                0.1696147673,
                0.140461827,
                0.078495506,
                0.0176508866,
                (-0.0355195984),
                (-0.0324460638),
                (-0.0220863078),
            ],
        ),
        8,
    )?;
    if (out < (-0.1)) {
        return Err(MethodError::Refused("a skill below -0.1 says the outlook is substantially worse than persistence, which the record does not support"));
    }
    if (out > 0.5) {
        return Err(MethodError::Refused("a skill above 0.5 against persistence is beyond anything in the record, so the table was misread"));
    }
    return Ok(rt::fin(out, 15)?);
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
