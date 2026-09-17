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
/// * The table is defined for the three-hourly ap and is being fed a daily mean Ap — fails when Kp(ap) is concave, so by Jensen's inequality the table run on a daily mean returns a Kp above the mean of the eight three-hourly Kp and well below the daily peak. Measured on the solar-weather record over 10,297 days with both an Ap and all eight Kp: against the 24-hour mean the table reads high by 0.083 Kp (median), against the daily peak it reads low by 1.000 Kp; on disturbed days (Ap >= 48, 131 of them) those become 0.397 high and 1.606 low. Both signs are what the concavity argument predicts. A design sized on the peak slot through this node alone is sized on a sky 1.6 Kp quieter than the record's, and that is on exactly the days a drag design is sized by. sw_kp_slot_bias now measures both offsets and publishes the peak one, +1.317 Kp at the design Ap of 158; this node's own answer still carries them uncorrected, because the correction is a separate row a consumer adds rather than something applied inside the conversion.
/// * The 28 pairs are written once, in vleo-core, and THEY USED TO BE WRITTEN TWICE — fails when a published table is hand-copied. `vleo_core::physics::env::kp_from_ap` held the same scale with Kp tabulated as decimals — 0.33, 0.67 — where this node's hole held it in exact thirds, which is how IAGA defines the index. The two disagreed by up to 0.0033 Kp everywhere between the anchors, and nothing caught it for as long as both existed, because the fixtures on this row are the published table's ANCHOR points and the anchors are precisely where two transcriptions of one table agree. The kernel's copy is now in thirds and this hole calls it, so there is one table; but the lesson is about the evidence rather than the table — a fixture set drawn only from a source's own tabulated points cannot see a transcription error in what lies between them
/// * Straight lines between the 28 tabulated points — fails when the published scale is a discrete table, so every value strictly between two anchors is this node's choice and not the source's. ap grows roughly geometrically with Kp, so interpolating linearly in ap rather than in its logarithm understates Kp inside a bin; the worst departure between the two over the whole domain is 0.017 Kp, in the 2-to-3 bin. That is the size of the arbitrariness, and it is smaller than the slot bias above by two orders of magnitude.
/// * Clipped to the table's ends: below ap 0 and above ap 400 the answer is 0 and 9, and the guard against the first of those lives in the producer, not here — fails when above ap 400 every storm returns exactly 9, so the largest storm on record and a merely severe one become the same number and any relation reading Kp stops responding. The largest daily Ap in the solar-weather record is 273, so nothing in this record reaches the clip — but a scenario multiplier applied to a disturbed day can, and it will do so silently. At the other end the clamp is worse than silent, it is plausible: ap -1, ap -1e9 and negative infinity all read back as Kp 0, the quietest possible sky, which no guard on this node can catch because 0 is a legitimate Kp. What protects a run is the PRODUCER's declared range — sw_storm_return_level publishes 20 to 230 and refuses before this node is reached — because in this repository a range travels with the variable and an [[input]] declares no range of its own. So a direct call to evaluate() with a negative ap, which is what a test does and what a future consumer with a looser range would do, returns Kp 0 rather than refusing. This was going to be fixed here with an explicit fault; generation refused the body, correctly — a hole may not construct a fault, because guards belong to declared domains where their reason is attached. The honest fix is a range on the producer, and that is where it now is.
pub const NODE_ID: &str = "sw_kp_from_ap";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x938c0600d69f62b5;

pub fn evaluate(ap: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : clamp the daily Ap to the table's domain, then read Kp off the published 28-point scale -> Ratio
    // The published scale, as IAGA prints it: 28 pairs, ap against Kp, with Kp in
    // thirds from 0 to 9. It lives in vleo-core as `env::kp_from_ap` and this
    // hole calls it rather than restating it.
    //
    // THIS HOLE USED TO HOLD ITS OWN COPY OF THE 28 PAIRS, and that is why the
    // call is here. The kernel's copy tabulated Kp as decimals — 0.33, 0.67 —
    // where this one used exact thirds, so the two disagreed by up to 0.0033 Kp
    // everywhere between the anchors. Nothing caught it for as long as both
    // existed: the fixtures below are the published table's anchor points, and
    // the anchors are precisely where two transcriptions of one table agree.
    // The kernel's copy is now in thirds and is the only one.
    //
    // kp_from_ap clamps at both ends instead of extrapolating, which is the
    // clamp the sheet declares rather than a convenience: below ap 0 and above
    // ap 400 the scale simply does not continue, and a straight line drawn past
    // either end would be this node inventing sky the source never described.
    let k: Ratio = Ratio::new(vleo_core::physics::env::kp_from_ap(ap.get()));
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
