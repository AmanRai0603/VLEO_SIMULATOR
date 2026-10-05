//! GENERATED from the method of `sw_kp_scenarios` by `cargo xtask docs`, translated by
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

/// The method of `sw_kp_scenarios`, source `iaga_kp_ap`:
///
/// ```text
/// # Transcribed from crates/vleo-mod-solar/nodes/sw_kp_scenarios/model.rs (HOLE 1 and its guards) — the method the code already runs.
/// # Numbers: the IAGA Kp-ap scale (iaga_kp_ap) as vleo_core::physics::env::kp_from_ap, and the two slot offsets kp_mean_slot_bias / kp_peak_slot_bias, medians over bundles/solar-weather@2026.09.14 in prf_ap2kp's nine Ap bins.
/// # The node publishes ten values: it publishes the nine members, as the Rust's Answer carries them,
/// # and returns the primary (Kp_peak_hotday). All ten are guarded first, because the Rust refuses the
/// # whole set when any member is out of domain.
/// # Every table is linear interpolation held at its ends (pmath::interp), as the kernel's are.
///
/// let base_nominal = interp(ap_nominal, [0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 12.0, 15.0, 18.0, 22.0, 27.0, 32.0, 39.0, 48.0, 56.0, 67.0, 80.0, 94.0, 111.0, 132.0, 154.0, 179.0, 207.0, 236.0, 300.0, 400.0], [0.0, 0.3333333333333333, 0.6666666666666666, 1.0, 1.3333333333333333, 1.6666666666666667, 2.0, 2.3333333333333335, 2.6666666666666665, 3.0, 3.3333333333333335, 3.6666666666666665, 4.0, 4.333333333333333, 4.666666666666667, 5.0, 5.333333333333333, 5.666666666666667, 6.0, 6.333333333333333, 6.666666666666667, 7.0, 7.333333333333333, 7.666666666666667, 8.0, 8.333333333333334, 8.666666666666666, 9.0])
/// let base_hotmean = interp(ap_hotmean, [0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 12.0, 15.0, 18.0, 22.0, 27.0, 32.0, 39.0, 48.0, 56.0, 67.0, 80.0, 94.0, 111.0, 132.0, 154.0, 179.0, 207.0, 236.0, 300.0, 400.0], [0.0, 0.3333333333333333, 0.6666666666666666, 1.0, 1.3333333333333333, 1.6666666666666667, 2.0, 2.3333333333333335, 2.6666666666666665, 3.0, 3.3333333333333335, 3.6666666666666665, 4.0, 4.333333333333333, 4.666666666666667, 5.0, 5.333333333333333, 5.666666666666667, 6.0, 6.333333333333333, 6.666666666666667, 7.0, 7.333333333333333, 7.666666666666667, 8.0, 8.333333333333334, 8.666666666666666, 9.0])
/// let base_coldmean = interp(ap_coldmean, [0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 12.0, 15.0, 18.0, 22.0, 27.0, 32.0, 39.0, 48.0, 56.0, 67.0, 80.0, 94.0, 111.0, 132.0, 154.0, 179.0, 207.0, 236.0, 300.0, 400.0], [0.0, 0.3333333333333333, 0.6666666666666666, 1.0, 1.3333333333333333, 1.6666666666666667, 2.0, 2.3333333333333335, 2.6666666666666665, 3.0, 3.3333333333333335, 3.6666666666666665, 4.0, 4.333333333333333, 4.666666666666667, 5.0, 5.333333333333333, 5.666666666666667, 6.0, 6.333333333333333, 6.666666666666667, 7.0, 7.333333333333333, 7.666666666666667, 8.0, 8.333333333333334, 8.666666666666666, 9.0])
/// let base_hotday = interp(ap_hotday, [0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 12.0, 15.0, 18.0, 22.0, 27.0, 32.0, 39.0, 48.0, 56.0, 67.0, 80.0, 94.0, 111.0, 132.0, 154.0, 179.0, 207.0, 236.0, 300.0, 400.0], [0.0, 0.3333333333333333, 0.6666666666666666, 1.0, 1.3333333333333333, 1.6666666666666667, 2.0, 2.3333333333333335, 2.6666666666666665, 3.0, 3.3333333333333335, 3.6666666666666665, 4.0, 4.333333333333333, 4.666666666666667, 5.0, 5.333333333333333, 5.666666666666667, 6.0, 6.333333333333333, 6.666666666666667, 7.0, 7.333333333333333, 7.666666666666667, 8.0, 8.333333333333334, 8.666666666666666, 9.0])
/// let base_coldday = interp(ap_coldday, [0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 12.0, 15.0, 18.0, 22.0, 27.0, 32.0, 39.0, 48.0, 56.0, 67.0, 80.0, 94.0, 111.0, 132.0, 154.0, 179.0, 207.0, 236.0, 300.0, 400.0], [0.0, 0.3333333333333333, 0.6666666666666666, 1.0, 1.3333333333333333, 1.6666666666666667, 2.0, 2.3333333333333335, 2.6666666666666665, 3.0, 3.3333333333333335, 3.6666666666666665, 4.0, 4.333333333333333, 4.666666666666667, 5.0, 5.333333333333333, 5.666666666666667, 6.0, 6.333333333333333, 6.666666666666667, 7.0, 7.333333333333333, 7.666666666666667, 8.0, 8.333333333333334, 8.666666666666666, 9.0])
///
/// let kp_mean_nominal = base_nominal + interp(ap_nominal, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0], [0.041666666666666685, -0.125, -0.09763888888888905, -0.11111111111111116, -0.13333333333333286, -0.24099537037037067, -0.33333333333333304, -0.4487179487179489, -0.4868948412698413])
/// let kp_peak_nominal = base_nominal + interp(ap_nominal, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0], [1.0, 0.8333333333333333, 1.1144067796610169, 1.0, 1.2, 1.3333333333333333, 1.5454545454545454, 1.7469135802469136, 1.3174603174603174])
/// let kp_mean_hotmean = base_hotmean + interp(ap_hotmean, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0], [0.041666666666666685, -0.125, -0.09763888888888905, -0.11111111111111116, -0.13333333333333286, -0.24099537037037067, -0.33333333333333304, -0.4487179487179489, -0.4868948412698413])
/// let kp_peak_hotmean = base_hotmean + interp(ap_hotmean, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0], [1.0, 0.8333333333333333, 1.1144067796610169, 1.0, 1.2, 1.3333333333333333, 1.5454545454545454, 1.7469135802469136, 1.3174603174603174])
/// let kp_mean_coldmean = base_coldmean + interp(ap_coldmean, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0], [0.041666666666666685, -0.125, -0.09763888888888905, -0.11111111111111116, -0.13333333333333286, -0.24099537037037067, -0.33333333333333304, -0.4487179487179489, -0.4868948412698413])
/// let kp_peak_coldmean = base_coldmean + interp(ap_coldmean, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0], [1.0, 0.8333333333333333, 1.1144067796610169, 1.0, 1.2, 1.3333333333333333, 1.5454545454545454, 1.7469135802469136, 1.3174603174603174])
/// let kp_mean_hotday = base_hotday + interp(ap_hotday, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0], [0.041666666666666685, -0.125, -0.09763888888888905, -0.11111111111111116, -0.13333333333333286, -0.24099537037037067, -0.33333333333333304, -0.4487179487179489, -0.4868948412698413])
/// let kp_peak_hotday = base_hotday + interp(ap_hotday, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0], [1.0, 0.8333333333333333, 1.1144067796610169, 1.0, 1.2, 1.3333333333333333, 1.5454545454545454, 1.7469135802469136, 1.3174603174603174])
/// let kp_mean_coldday = base_coldday + interp(ap_coldday, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0], [0.041666666666666685, -0.125, -0.09763888888888905, -0.11111111111111116, -0.13333333333333286, -0.24099537037037067, -0.33333333333333304, -0.4487179487179489, -0.4868948412698413])
/// let kp_peak_coldday = base_coldday + interp(ap_coldday, [2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0], [1.0, 0.8333333333333333, 1.1144067796610169, 1.0, 1.2, 1.3333333333333333, 1.5454545454545454, 1.7469135802469136, 1.3174603174603174])
///
/// # Generated guards: each member's declared domain, the primary first, then the published members in order.
/// if kp_peak_hotday < 0 then
///   refuse "Kp_peak_hotday: Kp is defined on 0 to 9; a negative index is a sign error, not a quiet sky"
/// end
/// if kp_peak_hotday > 9 then
///   refuse "Kp_peak_hotday: Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400"
/// end
/// if kp_mean_nominal < 0 then
///   refuse "Kp_mean_nominal: Kp is defined on 0 to 9; a negative value is a sign error"
/// end
/// if kp_mean_nominal > 9 then
///   refuse "Kp_mean_nominal: Kp is defined on 0 to 9"
/// end
/// if kp_mean_hotmean < 0 then
///   refuse "Kp_mean_hotmean: Kp is defined on 0 to 9; a negative value is a sign error"
/// end
/// if kp_mean_hotmean > 9 then
///   refuse "Kp_mean_hotmean: Kp is defined on 0 to 9"
/// end
/// if kp_mean_coldmean < 0 then
///   refuse "Kp_mean_coldmean: Kp is defined on 0 to 9; a negative value is a sign error"
/// end
/// if kp_mean_coldmean > 9 then
///   refuse "Kp_mean_coldmean: Kp is defined on 0 to 9"
/// end
/// if kp_mean_hotday < 0 then
///   refuse "Kp_mean_hotday: Kp is defined on 0 to 9; a negative value is a sign error"
/// end
/// if kp_mean_hotday > 9 then
///   refuse "Kp_mean_hotday: Kp is defined on 0 to 9"
/// end
/// if kp_mean_coldday < 0 then
///   refuse "Kp_mean_coldday: Kp is defined on 0 to 9; a negative value is a sign error"
/// end
/// if kp_mean_coldday > 9 then
///   refuse "Kp_mean_coldday: Kp is defined on 0 to 9"
/// end
/// if kp_peak_nominal < 0 then
///   refuse "Kp_peak_nominal: Kp is defined on 0 to 9; a negative value is a sign error"
/// end
/// if kp_peak_nominal > 9 then
///   refuse "Kp_peak_nominal: Kp is defined on 0 to 9"
/// end
/// if kp_peak_hotmean < 0 then
///   refuse "Kp_peak_hotmean: Kp is defined on 0 to 9; a negative value is a sign error"
/// end
/// if kp_peak_hotmean > 9 then
///   refuse "Kp_peak_hotmean: Kp is defined on 0 to 9"
/// end
/// if kp_peak_coldmean < 0 then
///   refuse "Kp_peak_coldmean: Kp is defined on 0 to 9; a negative value is a sign error"
/// end
/// if kp_peak_coldmean > 9 then
///   refuse "Kp_peak_coldmean: Kp is defined on 0 to 9"
/// end
/// if kp_peak_coldday < 0 then
///   refuse "Kp_peak_coldday: Kp is defined on 0 to 9; a negative value is a sign error"
/// end
/// if kp_peak_coldday > 9 then
///   refuse "Kp_peak_coldday: Kp is defined on 0 to 9"
/// end
///
/// publish Kp_mean_nominal = kp_mean_nominal
/// publish Kp_mean_hotmean = kp_mean_hotmean
/// publish Kp_mean_coldmean = kp_mean_coldmean
/// publish Kp_mean_hotday = kp_mean_hotday
/// publish Kp_mean_coldday = kp_mean_coldday
/// publish Kp_peak_nominal = kp_peak_nominal
/// publish Kp_peak_hotmean = kp_peak_hotmean
/// publish Kp_peak_coldmean = kp_peak_coldmean
/// publish Kp_peak_coldday = kp_peak_coldday
/// return kp_peak_hotday
/// ```
/// Returns the answer, and the published members in this order: Kp_mean_nominal, Kp_mean_hotmean, Kp_mean_coldmean, Kp_mean_hotday, Kp_mean_coldday, Kp_peak_nominal, Kp_peak_hotmean, Kp_peak_coldmean, Kp_peak_coldday.
pub fn evaluate(
    ap_nominal: f64,
    ap_hotmean: f64,
    ap_coldmean: f64,
    ap_hotday: f64,
    ap_coldday: f64,
) -> Result<(f64, [f64; 9]), MethodError> {
    let mut published = [0.0_f64; 9];
    if !pmath::is_finite(ap_nominal) || pmath::is_nan(ap_nominal) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ap_hotmean) || pmath::is_nan(ap_hotmean) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ap_coldmean) || pmath::is_nan(ap_coldmean) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ap_hotday) || pmath::is_nan(ap_hotday) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    if !pmath::is_finite(ap_coldday) || pmath::is_nan(ap_coldday) {
        return Err(MethodError::Refused("an input is not a finite number"));
    }
    let base_nominal: f64 = rt::fin(
        pmath::interp(
            ap_nominal,
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
        8,
    )?;
    let base_hotmean: f64 = rt::fin(
        pmath::interp(
            ap_hotmean,
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
    let base_coldmean: f64 = rt::fin(
        pmath::interp(
            ap_coldmean,
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
        10,
    )?;
    let base_hotday: f64 = rt::fin(
        pmath::interp(
            ap_hotday,
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
        11,
    )?;
    let base_coldday: f64 = rt::fin(
        pmath::interp(
            ap_coldday,
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
        12,
    )?;
    let kp_mean_nominal: f64 = rt::fin(
        (base_nominal
            + pmath::interp(
                ap_nominal,
                &[2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0],
                &[
                    0.041666666666666685,
                    (-0.125),
                    (-0.09763888888888905),
                    (-0.11111111111111116),
                    (-0.13333333333333286),
                    (-0.24099537037037067),
                    (-0.33333333333333304),
                    (-0.4487179487179489),
                    (-0.4868948412698413),
                ],
            )),
        14,
    )?;
    let kp_peak_nominal: f64 = rt::fin(
        (base_nominal
            + pmath::interp(
                ap_nominal,
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
            )),
        15,
    )?;
    let kp_mean_hotmean: f64 = rt::fin(
        (base_hotmean
            + pmath::interp(
                ap_hotmean,
                &[2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0],
                &[
                    0.041666666666666685,
                    (-0.125),
                    (-0.09763888888888905),
                    (-0.11111111111111116),
                    (-0.13333333333333286),
                    (-0.24099537037037067),
                    (-0.33333333333333304),
                    (-0.4487179487179489),
                    (-0.4868948412698413),
                ],
            )),
        16,
    )?;
    let kp_peak_hotmean: f64 = rt::fin(
        (base_hotmean
            + pmath::interp(
                ap_hotmean,
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
            )),
        17,
    )?;
    let kp_mean_coldmean: f64 = rt::fin(
        (base_coldmean
            + pmath::interp(
                ap_coldmean,
                &[2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0],
                &[
                    0.041666666666666685,
                    (-0.125),
                    (-0.09763888888888905),
                    (-0.11111111111111116),
                    (-0.13333333333333286),
                    (-0.24099537037037067),
                    (-0.33333333333333304),
                    (-0.4487179487179489),
                    (-0.4868948412698413),
                ],
            )),
        18,
    )?;
    let kp_peak_coldmean: f64 = rt::fin(
        (base_coldmean
            + pmath::interp(
                ap_coldmean,
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
            )),
        19,
    )?;
    let kp_mean_hotday: f64 = rt::fin(
        (base_hotday
            + pmath::interp(
                ap_hotday,
                &[2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0],
                &[
                    0.041666666666666685,
                    (-0.125),
                    (-0.09763888888888905),
                    (-0.11111111111111116),
                    (-0.13333333333333286),
                    (-0.24099537037037067),
                    (-0.33333333333333304),
                    (-0.4487179487179489),
                    (-0.4868948412698413),
                ],
            )),
        20,
    )?;
    let kp_peak_hotday: f64 = rt::fin(
        (base_hotday
            + pmath::interp(
                ap_hotday,
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
            )),
        21,
    )?;
    let kp_mean_coldday: f64 = rt::fin(
        (base_coldday
            + pmath::interp(
                ap_coldday,
                &[2.5, 7.5, 12.5, 17.5, 25.0, 37.5, 57.5, 90.0, 255.0],
                &[
                    0.041666666666666685,
                    (-0.125),
                    (-0.09763888888888905),
                    (-0.11111111111111116),
                    (-0.13333333333333286),
                    (-0.24099537037037067),
                    (-0.33333333333333304),
                    (-0.4487179487179489),
                    (-0.4868948412698413),
                ],
            )),
        22,
    )?;
    let kp_peak_coldday: f64 = rt::fin(
        (base_coldday
            + pmath::interp(
                ap_coldday,
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
            )),
        23,
    )?;
    if (kp_peak_hotday < 0.0) {
        return Err(MethodError::Refused("Kp_peak_hotday: Kp is defined on 0 to 9; a negative index is a sign error, not a quiet sky"));
    }
    if (kp_peak_hotday > 9.0) {
        return Err(MethodError::Refused(
            "Kp_peak_hotday: Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400",
        ));
    }
    if (kp_mean_nominal < 0.0) {
        return Err(MethodError::Refused(
            "Kp_mean_nominal: Kp is defined on 0 to 9; a negative value is a sign error",
        ));
    }
    if (kp_mean_nominal > 9.0) {
        return Err(MethodError::Refused(
            "Kp_mean_nominal: Kp is defined on 0 to 9",
        ));
    }
    if (kp_mean_hotmean < 0.0) {
        return Err(MethodError::Refused(
            "Kp_mean_hotmean: Kp is defined on 0 to 9; a negative value is a sign error",
        ));
    }
    if (kp_mean_hotmean > 9.0) {
        return Err(MethodError::Refused(
            "Kp_mean_hotmean: Kp is defined on 0 to 9",
        ));
    }
    if (kp_mean_coldmean < 0.0) {
        return Err(MethodError::Refused(
            "Kp_mean_coldmean: Kp is defined on 0 to 9; a negative value is a sign error",
        ));
    }
    if (kp_mean_coldmean > 9.0) {
        return Err(MethodError::Refused(
            "Kp_mean_coldmean: Kp is defined on 0 to 9",
        ));
    }
    if (kp_mean_hotday < 0.0) {
        return Err(MethodError::Refused(
            "Kp_mean_hotday: Kp is defined on 0 to 9; a negative value is a sign error",
        ));
    }
    if (kp_mean_hotday > 9.0) {
        return Err(MethodError::Refused(
            "Kp_mean_hotday: Kp is defined on 0 to 9",
        ));
    }
    if (kp_mean_coldday < 0.0) {
        return Err(MethodError::Refused(
            "Kp_mean_coldday: Kp is defined on 0 to 9; a negative value is a sign error",
        ));
    }
    if (kp_mean_coldday > 9.0) {
        return Err(MethodError::Refused(
            "Kp_mean_coldday: Kp is defined on 0 to 9",
        ));
    }
    if (kp_peak_nominal < 0.0) {
        return Err(MethodError::Refused(
            "Kp_peak_nominal: Kp is defined on 0 to 9; a negative value is a sign error",
        ));
    }
    if (kp_peak_nominal > 9.0) {
        return Err(MethodError::Refused(
            "Kp_peak_nominal: Kp is defined on 0 to 9",
        ));
    }
    if (kp_peak_hotmean < 0.0) {
        return Err(MethodError::Refused(
            "Kp_peak_hotmean: Kp is defined on 0 to 9; a negative value is a sign error",
        ));
    }
    if (kp_peak_hotmean > 9.0) {
        return Err(MethodError::Refused(
            "Kp_peak_hotmean: Kp is defined on 0 to 9",
        ));
    }
    if (kp_peak_coldmean < 0.0) {
        return Err(MethodError::Refused(
            "Kp_peak_coldmean: Kp is defined on 0 to 9; a negative value is a sign error",
        ));
    }
    if (kp_peak_coldmean > 9.0) {
        return Err(MethodError::Refused(
            "Kp_peak_coldmean: Kp is defined on 0 to 9",
        ));
    }
    if (kp_peak_coldday < 0.0) {
        return Err(MethodError::Refused(
            "Kp_peak_coldday: Kp is defined on 0 to 9; a negative value is a sign error",
        ));
    }
    if (kp_peak_coldday > 9.0) {
        return Err(MethodError::Refused(
            "Kp_peak_coldday: Kp is defined on 0 to 9",
        ));
    }
    published[0] = rt::fin(kp_mean_nominal, 87)?; // Kp_mean_nominal
    published[1] = rt::fin(kp_mean_hotmean, 88)?; // Kp_mean_hotmean
    published[2] = rt::fin(kp_mean_coldmean, 89)?; // Kp_mean_coldmean
    published[3] = rt::fin(kp_mean_hotday, 90)?; // Kp_mean_hotday
    published[4] = rt::fin(kp_mean_coldday, 91)?; // Kp_mean_coldday
    published[5] = rt::fin(kp_peak_nominal, 92)?; // Kp_peak_nominal
    published[6] = rt::fin(kp_peak_hotmean, 93)?; // Kp_peak_hotmean
    published[7] = rt::fin(kp_peak_coldmean, 94)?; // Kp_peak_coldmean
    published[8] = rt::fin(kp_peak_coldday, 95)?; // Kp_peak_coldday
    return Ok((rt::fin(kp_peak_hotday, 96)?, published));
    Err(MethodError::Degenerate {
        line: 0,
        what: "the method ended without an answer",
    })
}
