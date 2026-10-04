// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What Kp does each of the five driver scenarios carry, in the mean slot and in the peak slot?
///
/// `Kp_slot(s) = kp_from_ap(Ap_s) + dKp_slot(Ap_s), for each of the five scenarios and each of the two slots`
///
/// Source: `iaga_kp_ap`
///
/// Ten numbers, and they are the last two columns of the study's driver
/// product. Every relation behind them already existed on its own row in this
/// subsystem; what did not exist was a way to evaluate them at five Ap values
/// at once, because a node publishes a value and a consumer cannot ask it for
/// its relation. The two slots are different questions about the same day. Ap
/// is a daily mean, Kp is reported in eight three-hourly slots, and Kp(ap) is
/// concave — so the published conversion run on a daily mean lands ABOVE
/// the mean of the eight slots and well BELOW the peak. A design sized on the
/// conversion alone is sized on a sky quieter than the record's, and on
/// exactly the days a drag design is sized by.
///
/// # Assumptions
///
/// * The offsets are medians over nine bins of Ap and nothing else — fails when the day is unusual in a way Ap does not capture. The correction knows the daily mean and the bin it falls in; it does not know whether the day was one long storm or eight quiet slots and one severe one, and those have the same Ap and very different peaks. The median is the middle of that spread, so half of the days in any bin exceed the peak this row publishes
/// * The tables are clamped at both ends rather than extrapolated — fails when a scenario's Ap falls outside them. The scale runs to ap 400 and the bias bins to a centre of 255, so a hot day above that reads the last bin's offset. This tree's hot Ap day is 90.55, inside both; the study's driver sets never exceeded 42. A scenario multiplier applied to a disturbed day would reach the clamp silently
/// * Both slot corrections are measured on the record and belong to a bundle version — fails when the bundle moves. They are medians over bundles/solar-weather@2026.09.14 and must be re-measured when it does. They live in vleo-core, which is the second and third pieces of measured data in a kernel whose own comment used to say SOLAR_CYCLE_SHAPE was the only one — that claim is now wrong and the kernel says so
/// * This row composes and does not measure, so its errors are its constituents' errors — fails when a reader looks here for the physics. The scale is sw_kp_from_ap's, both offsets are sw_kp_mean_bias's and sw_kp_slot_bias's, and the five Ap values are the design rows'. What this row owns is the composition and the choice of five points, and its parity grid is the only place all three are checked together
pub const NODE_ID: &str = "sw_kp_scenarios";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x50102f9afbc92977;

/// The set this node publishes. One field per published variable, named
/// by the sheet's own symbol, in the order `OUTPUT_VARS` declares: this
/// node's own answer first, then each `[[publishes]]` block. The field
/// NAMES are what a hole body assigns, so the order cannot be got wrong
/// by hand, and every member is guarded against its own declared domain.
#[allow(non_snake_case)]
pub struct Answer {
    /// Kp in both slots, for all five scenarios — this node's own answer.
    pub Kp_peak_hotday: Ratio,
    /// Kp, mean slot, nominal scenario — published as `sw_kp_scenarios.kp_mean_nominal`.
    pub Kp_mean_nominal: Ratio,
    /// Kp, mean slot, sustained disturbed scenario — published as `sw_kp_scenarios.kp_mean_hotmean`.
    pub Kp_mean_hotmean: Ratio,
    /// Kp, mean slot, sustained quiet scenario — published as `sw_kp_scenarios.kp_mean_coldmean`.
    pub Kp_mean_coldmean: Ratio,
    /// Kp, mean slot, disturbed single day — published as `sw_kp_scenarios.kp_mean_hotday`.
    pub Kp_mean_hotday: Ratio,
    /// Kp, mean slot, quietest single day — published as `sw_kp_scenarios.kp_mean_coldday`.
    pub Kp_mean_coldday: Ratio,
    /// Kp, peak slot, nominal scenario — published as `sw_kp_scenarios.kp_peak_nominal`.
    pub Kp_peak_nominal: Ratio,
    /// Kp, peak slot, sustained disturbed scenario — published as `sw_kp_scenarios.kp_peak_hotmean`.
    pub Kp_peak_hotmean: Ratio,
    /// Kp, peak slot, sustained quiet scenario — published as `sw_kp_scenarios.kp_peak_coldmean`.
    pub Kp_peak_coldmean: Ratio,
    /// Kp, peak slot, quietest single day — published as `sw_kp_scenarios.kp_peak_coldday`.
    pub Kp_peak_coldday: Ratio,
}

pub fn evaluate(ap_nominal: Ratio, ap_hotmean: Ratio, ap_coldmean: Ratio, ap_hotday: Ratio, ap_coldday: Ratio) -> Result<Answer, Fault> {
    // ---- HOLE 1 : convert each scenario's Ap to Kp on the published scale and add the measured slot offset, for both slots -> Answer
    // Three relations from vleo-core, composed at five points. None of the three
    // is written here: the scale is env::kp_from_ap, the two offsets are
    // env::kp_mean_slot_bias and env::kp_peak_slot_bias, and each is read by
    // this row and by the row that owns it. A table copied into both would
    // drift from itself without anything noticing, which is what happened to
    // the ap-to-Kp scale for as long as it had two copies.
    use vleo_core::physics::env::{kp_from_ap, kp_mean_slot_bias, kp_peak_slot_bias};

    // One closure per slot, so the composition is written once rather than ten
    // times. Ten hand-written lines of `kp_from_ap(x) + bias(x)` is ten chances
    // to pair the wrong scenario with the wrong table.
    let mean = |ap: Ratio| -> Ratio { Ratio::new(kp_from_ap(ap.get()) + kp_mean_slot_bias(ap.get())) };
    let peak = |ap: Ratio| -> Ratio { Ratio::new(kp_from_ap(ap.get()) + kp_peak_slot_bias(ap.get())) };

    let set: Answer = Answer {
        // The primary: the worst slot of the worst day, which is what a design
        // sized against geomagnetic activity reads.
        Kp_peak_hotday: peak(ap_hotday),

        Kp_mean_nominal: mean(ap_nominal),
        Kp_mean_hotmean: mean(ap_hotmean),
        Kp_mean_coldmean: mean(ap_coldmean),
        Kp_mean_hotday: mean(ap_hotday),
        Kp_mean_coldday: mean(ap_coldday),

        Kp_peak_nominal: peak(ap_nominal),
        Kp_peak_hotmean: peak(ap_hotmean),
        Kp_peak_coldmean: peak(ap_coldmean),
        Kp_peak_coldday: peak(ap_coldday),
    };
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Answer = set;

    // generated · every published member carries its own declared domain.
    // A set whose primary is in range and whose fifth member is not is not a
    // usable set, and the member a consumer reads may be any of them.
    if !answer.Kp_peak_hotday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_peak_hotday", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_peak_hotday.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_hotday", value: answer.Kp_peak_hotday.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_peak_hotday.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_hotday", value: answer.Kp_peak_hotday.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400. This member is the worst slot of the worst day, so it is the one of the ten most likely to reach it" });
    }
    if !answer.Kp_mean_nominal.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_mean_nominal", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_mean_nominal.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_nominal", value: answer.Kp_mean_nominal.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9; a negative value is a sign error" });
    }
    if answer.Kp_mean_nominal.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_nominal", value: answer.Kp_mean_nominal.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9" });
    }
    if !answer.Kp_mean_hotmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_mean_hotmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_mean_hotmean.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_hotmean", value: answer.Kp_mean_hotmean.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9; a negative value is a sign error" });
    }
    if answer.Kp_mean_hotmean.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_hotmean", value: answer.Kp_mean_hotmean.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9" });
    }
    if !answer.Kp_mean_coldmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_mean_coldmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_mean_coldmean.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_coldmean", value: answer.Kp_mean_coldmean.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9; a negative value is a sign error, and the quiet scenarios are the ones that approach the floor" });
    }
    if answer.Kp_mean_coldmean.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_coldmean", value: answer.Kp_mean_coldmean.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9" });
    }
    if !answer.Kp_mean_hotday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_mean_hotday", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_mean_hotday.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_hotday", value: answer.Kp_mean_hotday.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9; a negative value is a sign error" });
    }
    if answer.Kp_mean_hotday.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_hotday", value: answer.Kp_mean_hotday.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9" });
    }
    if !answer.Kp_mean_coldday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_mean_coldday", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_mean_coldday.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_coldday", value: answer.Kp_mean_coldday.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9. This is the lowest of the ten and the one nearest the floor: at the declared window it is 1.27" });
    }
    if answer.Kp_mean_coldday.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_coldday", value: answer.Kp_mean_coldday.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9" });
    }
    if !answer.Kp_peak_nominal.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_peak_nominal", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_peak_nominal.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_nominal", value: answer.Kp_peak_nominal.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9; a negative value is a sign error" });
    }
    if answer.Kp_peak_nominal.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_nominal", value: answer.Kp_peak_nominal.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9" });
    }
    if !answer.Kp_peak_hotmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_peak_hotmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_peak_hotmean.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_hotmean", value: answer.Kp_peak_hotmean.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9; a negative value is a sign error" });
    }
    if answer.Kp_peak_hotmean.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_hotmean", value: answer.Kp_peak_hotmean.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9" });
    }
    if !answer.Kp_peak_coldmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_peak_coldmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_peak_coldmean.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_coldmean", value: answer.Kp_peak_coldmean.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9; a negative value is a sign error" });
    }
    if answer.Kp_peak_coldmean.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_coldmean", value: answer.Kp_peak_coldmean.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9" });
    }
    if !answer.Kp_peak_coldday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_peak_coldday", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_peak_coldday.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_coldday", value: answer.Kp_peak_coldday.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9; a negative value is a sign error" });
    }
    if answer.Kp_peak_coldday.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_coldday", value: answer.Kp_peak_coldday.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9" });
    }
    Ok(answer)
}
