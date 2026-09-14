// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What Kp does this daily planetary Ap mean?
///
/// `Kp = piecewise_linear(ap_28 -> kp_28, clamp(Ap, 0, 400))`
///
/// Source: `iaga_kp_ap`
///
/// The atmosphere model wants Kp; the design product carries Ap. This is that
/// conversion and only that — the published table, applied as published.
/// What flows in is the design storm from sw_storm_return_level, so what
/// flows out is the Kp of that storm and not of an average day. The bias the
/// table carries when a daily mean is fed to a three-hourly scale is measured
/// separately, in sw_kp_slot_bias, and is not corrected here.
///
/// # Assumptions
///
/// * The table is defined for the three-hourly ap and is being fed a daily mean Ap — fails when Kp(ap) is concave, so by Jensen's inequality the table run on a daily mean returns a Kp above the mean of the eight three-hourly Kp and well below the daily peak. Measured on the solar-weather record over 10,297 days with both an Ap and all eight Kp: against the 24-hour mean the table reads high by 0.083 Kp (median), against the daily peak it reads low by 1.000 Kp; on disturbed days (Ap >= 48, 131 of them) those become 0.397 high and 1.606 low. Both signs are what the concavity argument predicts. A design sized on the peak slot through this node alone is sized on a sky 1.6 Kp quieter than the record's, and that is on exactly the days a drag design is sized by. sw_kp_slot_bias measures and publishes both offsets; until it is written this node's answer carries them uncorrected.
/// * Straight lines between the 28 tabulated points — fails when the published scale is a discrete table, so every value strictly between two anchors is this node's choice and not the source's. ap grows roughly geometrically with Kp, so interpolating linearly in ap rather than in its logarithm understates Kp inside a bin; the worst departure between the two over the whole domain is 0.017 Kp, in the 2-to-3 bin. That is the size of the arbitrariness, and it is smaller than the slot bias above by two orders of magnitude.
/// * Clipped to the table's ends: below ap 0 and above ap 400 the answer is 0 and 9 — fails when above ap 400 every storm returns exactly 9, so the largest storm on record and a merely severe one become the same number and any relation reading Kp stops responding. The largest daily Ap in the solar-weather record is 273, so nothing in this record reaches the clip — but a scenario multiplier applied to a disturbed day can, and it will do so silently.
pub const NODE_ID: &str = "sw_kp_from_ap";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x938c0600d69f62b5;

pub fn evaluate(ap: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : clamp the daily Ap to the table's domain, then read Kp off the published 28-point scale -> Ratio
    // The published scale, as IAGA prints it: 28 pairs, ap against Kp, with Kp
    // in thirds from 0 to 9. Written as thirds rather than as decimals because
    // thirds are what the scale is defined in, and a decimal transcription is a
    // place for a digit to go missing.
    use vleo_core::math::Table1;
    const SCALE: Table1 = Table1 {
        x: &[
            0.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0, 12.0, 15.0, 18.0, 22.0, 27.0, 32.0, 39.0, 48.0,
            56.0, 67.0, 80.0, 94.0, 111.0, 132.0, 154.0, 179.0, 207.0, 236.0, 300.0, 400.0,
        ],
        y: &[
            0.0,
            1.0 / 3.0,
            2.0 / 3.0,
            1.0,
            4.0 / 3.0,
            5.0 / 3.0,
            2.0,
            7.0 / 3.0,
            8.0 / 3.0,
            3.0,
            10.0 / 3.0,
            11.0 / 3.0,
            4.0,
            13.0 / 3.0,
            14.0 / 3.0,
            5.0,
            16.0 / 3.0,
            17.0 / 3.0,
            6.0,
            19.0 / 3.0,
            20.0 / 3.0,
            7.0,
            22.0 / 3.0,
            23.0 / 3.0,
            8.0,
            25.0 / 3.0,
            26.0 / 3.0,
            9.0,
        ],
    };
    // Table1::at clamps at both ends instead of extrapolating, which is the
    // clamp the sheet declares rather than a convenience: below ap 0 and above
    // ap 400 the scale simply does not continue, and a straight line drawn past
    // either end would be this node inventing sky the source never described.
    let k: Ratio = Ratio::new(SCALE.at(ap.get()));
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = k;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0..9 and the table's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet day" });
    }
    if answer.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp", value: answer.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0..9 and the table's last point is Kp 9 at ap 400. This is the same guard env_kp carries, and it catches an ap value reaching a consumer that wanted Kp" });
    }
    Ok(answer)
}
