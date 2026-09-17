// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the solar-weather subsystem conclude, for a system reader who will not open it?
///
/// `driver_set = {nominal, hotmean, coldmean, hotday, coldday} x {f107, f107bar, ap}, each member relayed from the row that computed it`
///
/// Source: `noaa_swpc`
///
/// One row, one conclusion, no reaching in. Everything the subsystem
/// establishes — the record, the cycle, the storms, the slot bias, the two
/// spreads, both tails of each — arrives at five design scenarios, and this
/// carries them across to sys_space_environment as one thing. Five scenarios
/// because that is what the study's product is and what a design reads:
/// nominal is where the mission sits, the two *mean scenarios are what it
/// SUSTAINS for months, and the two *day scenarios are what one day inside
/// that reaches. An array is sized on a sustained level and a thermal
/// transient on a day. A tool that published one of the five would have
/// decided for the reader which of those their problem is.
///
/// # Assumptions
///
/// * The Kp columns do not cross, and the study's driver set has two of them — fails when a consumer needs Kp. The study publishes kp_mean and kp_peak per scenario, formed from that scenario's Ap by the published ap-to-Kp scale plus a measured slot bias. This tree has all three relations — sw_kp_from_ap, sw_kp_mean_bias, sw_kp_slot_bias — and cannot use them here, because the bus passes a node's VALUE and not its RELATION: each of those rows answers at one Ap, and a driver set needs them at five. Three ways out, none of them free: those three rows each publish a set of five, keyed to the scenarios; or the ap-to-Kp scale and both bias tables move into vleo-core as named functions this hole can call, at the cost of putting measured data in the kernel; or ten more rows exist, one per scenario per slot. Until one is chosen the Ap column crosses and the Kp columns do not, and a consumer that needs Kp must convert it itself — which is the duplication this row exists to prevent
/// * It relays and does not compute, and the f107bar column is the edge of that claim — fails when somebody calls the f107bar mapping a calculation. No value is combined with another and no constant appears; what happens is that one input is published under two names, because the study's hotday scenario carries hotmean's 81-day mean beneath it. If that is computation then a crossing cannot carry a set at all, and §20.3's decision needs revisiting rather than this hole
/// * Every member inherits every limitation of the row beneath it, and a system reader sees none of them — fails when this is the ordinary cost of a seam and it is worth stating where the seam is. The two *mean scenarios are 1.28-sigma bands, which is the 90th percentile and not the 95 per cent the run is labelled; the two *day scenarios stack a second one-sided percentile on top, which is nearer a 1-in-100 day than a 1-in-20; and the centre beneath all five is a cycle analogue that beyond one cycle past cycle 25's maximum is scaled by the mean amplitude of two completed cycles, whose peaks differ by 41 per cent. A reader at layer 2 sees fifteen numbers and a credibility vector, and would have to open five rows to learn any of that. The credibility travels; the assumptions do not
/// * One refused member refuses the whole set — fails when this is read as a defect in the seam. It is the seam's job: a driver set with one member the record cannot support is not a driver set, and publishing four members and a hole would put the judgement about which was which above the seam. It does mean this row's availability is the AND of ten rows, so the seam is the least available thing in the subsystem by construction
pub const NODE_ID: &str = "l3_solar_interface";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xab82ebb6e9826449;

/// The set this node publishes. One field per published variable, named
/// by the sheet's own symbol, in the order `OUTPUT_VARS` declares: this
/// node's own answer first, then each `[[publishes]]` block. The field
/// NAMES are what a hole body assigns, so the order cannot be got wrong
/// by hand, and every member is guarded against its own declared domain.
#[allow(non_snake_case)]
pub struct Answer {
    /// Solar weather — subsystem interface — this node's own answer.
    pub F107_hotmean: Ratio,
    /// F10.7 the mission is expected to sit at — published as `l3_solar_interface.f107_nominal`.
    pub F107_nominal: Ratio,
    /// F10.7 sustained on the cold side — published as `l3_solar_interface.f107_coldmean`.
    pub F107_coldmean: Ratio,
    /// F10.7 on the worst single day — published as `l3_solar_interface.f107_hotday`.
    pub F107_hotday: Ratio,
    /// F10.7 on the quietest single day — published as `l3_solar_interface.f107_coldday`.
    pub F107_coldday: Ratio,
    /// 81-day mean F10.7 beneath the nominal scenario — published as `l3_solar_interface.f107bar_nominal`.
    pub F107bar_nominal: Ratio,
    /// 81-day mean F10.7 beneath the hot sustained scenario — published as `l3_solar_interface.f107bar_hotmean`.
    pub F107bar_hotmean: Ratio,
    /// 81-day mean F10.7 beneath the cold sustained scenario — published as `l3_solar_interface.f107bar_coldmean`.
    pub F107bar_coldmean: Ratio,
    /// 81-day mean F10.7 the hot day rides on — published as `l3_solar_interface.f107bar_hotday`.
    pub F107bar_hotday: Ratio,
    /// 81-day mean F10.7 the cold day rides on — published as `l3_solar_interface.f107bar_coldday`.
    pub F107bar_coldday: Ratio,
    /// Ap the mission is expected to sit at — published as `l3_solar_interface.ap_nominal`.
    pub Ap_nominal: Ratio,
    /// Ap sustained on the disturbed side — published as `l3_solar_interface.ap_hotmean`.
    pub Ap_hotmean: Ratio,
    /// Ap sustained on the quiet side — published as `l3_solar_interface.ap_coldmean`.
    pub Ap_coldmean: Ratio,
    /// Ap on the most disturbed single day — published as `l3_solar_interface.ap_hotday`.
    pub Ap_hotday: Ratio,
    /// Ap on the quietest single day — published as `l3_solar_interface.ap_coldday`.
    pub Ap_coldday: Ratio,
    /// Kp, mean slot, nominal scenario — published as `l3_solar_interface.kp_mean_nominal`.
    pub Kp_mean_nominal: Ratio,
    /// Kp, mean slot, sustained disturbed scenario — published as `l3_solar_interface.kp_mean_hotmean`.
    pub Kp_mean_hotmean: Ratio,
    /// Kp, mean slot, sustained quiet scenario — published as `l3_solar_interface.kp_mean_coldmean`.
    pub Kp_mean_coldmean: Ratio,
    /// Kp, mean slot, disturbed single day — published as `l3_solar_interface.kp_mean_hotday`.
    pub Kp_mean_hotday: Ratio,
    /// Kp, mean slot, quietest single day — published as `l3_solar_interface.kp_mean_coldday`.
    pub Kp_mean_coldday: Ratio,
    /// Kp, peak slot, nominal scenario — published as `l3_solar_interface.kp_peak_nominal`.
    pub Kp_peak_nominal: Ratio,
    /// Kp, peak slot, sustained disturbed scenario — published as `l3_solar_interface.kp_peak_hotmean`.
    pub Kp_peak_hotmean: Ratio,
    /// Kp, peak slot, sustained quiet scenario — published as `l3_solar_interface.kp_peak_coldmean`.
    pub Kp_peak_coldmean: Ratio,
    /// Kp, peak slot, disturbed single day — published as `l3_solar_interface.kp_peak_hotday`.
    pub Kp_peak_hotday: Ratio,
    /// Kp, peak slot, quietest single day — published as `l3_solar_interface.kp_peak_coldday`.
    pub Kp_peak_coldday: Ratio,
}

pub fn evaluate(f107_centre: Ratio, f107_hot_long: Ratio, f107_cold_long: Ratio, f107_hot_day: Ratio, f107_cold_day: Ratio, ap_centre: Ratio, ap_hot_long: Ratio, ap_cold_long: Ratio, ap_hot_day: Ratio, ap_cold_day: Ratio, kp_mean_nominal: Ratio, kp_mean_hotmean: Ratio, kp_mean_coldmean: Ratio, kp_mean_hotday: Ratio, kp_mean_coldday: Ratio, kp_peak_nominal: Ratio, kp_peak_hotmean: Ratio, kp_peak_coldmean: Ratio, kp_peak_hotday: Ratio, kp_peak_coldday: Ratio) -> Result<Answer, Fault> {
    // ---- HOLE 1 : assemble the five scenarios from the ten rows that computed them, and carry them across the seam unchanged -> Answer
    // A crossing carries; it does not compute. The one thing that can go wrong
    // here is that the seam alters what it is handed — a stray factor, an
    // unasked-for unit conversion, a clamp inherited from the wrong row — and
    // both sides would still look plausible. So every member below is one input,
    // named, with no arithmetic anywhere in the block.
    //
    // Written out member by member rather than looped, because the mapping from
    // scenario to producing row IS the content of this row and a reader has to
    // be able to check it against the study's own table. Fifteen assignments is
    // the price of that being checkable.
    let set: Answer = Answer {
        // The three *mean scenarios: f107 == f107bar, because they ARE the
        // window mean. The study's own table shows the same.
        F107_hotmean: f107_hot_long,
        F107bar_hotmean: f107_hot_long,

        F107_nominal: f107_centre,
        F107bar_nominal: f107_centre,

        F107_coldmean: f107_cold_long,
        F107bar_coldmean: f107_cold_long,

        // The two *day scenarios: f107 is the day, f107bar is the SUSTAINED
        // level it rides on. This is the one place the row publishes an input
        // under a second name, and it is selection rather than arithmetic.
        F107_hotday: f107_hot_day,
        F107bar_hotday: f107_hot_long,

        F107_coldday: f107_cold_day,
        F107bar_coldday: f107_cold_long,

        // Ap has no 81-day companion in the study's driver set, so five members
        // rather than ten. The Kp columns are the ones missing from this set and
        // the sheet's first assumption says why.
        Ap_nominal: ap_centre,
        Ap_hotmean: ap_hot_long,
        Ap_coldmean: ap_cold_long,
        Ap_hotday: ap_hot_day,
        Ap_coldday: ap_cold_day,

        // The two Kp columns, relayed from the one row that evaluates the
        // published scale and both measured slot offsets at all five Ap. The
        // arithmetic is there and not here, because a crossing relays: putting
        // kp_from_ap(ap) + bias in this block would be subsystem work done where
        // no subsystem reviewer reads it.
        Kp_mean_nominal: kp_mean_nominal,
        Kp_mean_hotmean: kp_mean_hotmean,
        Kp_mean_coldmean: kp_mean_coldmean,
        Kp_mean_hotday: kp_mean_hotday,
        Kp_mean_coldday: kp_mean_coldday,

        Kp_peak_nominal: kp_peak_nominal,
        Kp_peak_hotmean: kp_peak_hotmean,
        Kp_peak_coldmean: kp_peak_coldmean,
        Kp_peak_hotday: kp_peak_hotday,
        Kp_peak_coldday: kp_peak_coldday,
    };
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Answer = set;

    // generated · every published member carries its own declared domain.
    // A set whose primary is in range and whose fifth member is not is not a
    // usable set, and the member a consumer reads may be any of them.
    if !answer.F107_hotmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_hotmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.F107_hotmean.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_hotmean", value: answer.F107_hotmean.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as env_f107 and every design row beneath this one: below 60 sfu has never been observed and every relation reading F10.7 has no support there. A crossing that narrowed or widened the range it carries would be changing the answer, so it declares the producer's own bounds" });
    }
    if answer.F107_hotmean.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_hotmean", value: answer.F107_hotmean.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as env_f107 and every design row beneath this one: above 400 sfu the exospheric temperature relation is extrapolated past the largest recorded daily value. Restating it here means a system reader sees the limit without opening the subsystem" });
    }
    if !answer.F107_nominal.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_nominal", reason: "the computation produced a value that is not a number" });
    }
    if answer.F107_nominal.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_nominal", value: answer.F107_nominal.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the producer's own floor, restated: below 60 sfu has never been observed" });
    }
    if answer.F107_nominal.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_nominal", value: answer.F107_nominal.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the producer's own ceiling, restated: above 400 sfu the temperature relation is extrapolated past the record" });
    }
    if !answer.F107_coldmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_coldmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.F107_coldmean.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_coldmean", value: answer.F107_coldmean.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the producer's own floor, restated. On the cold scenarios it is load-bearing rather than decorative: a band wider than the sky reaches through it" });
    }
    if answer.F107_coldmean.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_coldmean", value: answer.F107_coldmean.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the producer's own ceiling, restated. A COLD level near it means a spread has been added rather than subtracted somewhere beneath this row" });
    }
    if !answer.F107_hotday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_hotday", reason: "the computation produced a value that is not a number" });
    }
    if answer.F107_hotday.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_hotday", value: answer.F107_hotday.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the producer's own floor, restated: below 60 sfu has never been observed" });
    }
    if answer.F107_hotday.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_hotday", value: answer.F107_hotday.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the producer's own ceiling, restated. This is the member most likely to reach it — a sustained level with a daily excursion stacked on top — which is why the guard is on every member and not only the primary" });
    }
    if !answer.F107_coldday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107_coldday", reason: "the computation produced a value that is not a number" });
    }
    if answer.F107_coldday.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_coldday", value: answer.F107_coldday.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the producer's own floor, restated, and THIS IS THE MEMBER THAT REFUSES on the nominal case: sw_f107_cold_short reaches 38.36 sfu from a centre of 86.85, and 38 sfu has never been observed. The guard is beneath this row and fires there; this restates the same limit so a system reader sees it without opening the subsystem" });
    }
    if answer.F107_coldday.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107_coldday", value: answer.F107_coldday.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the producer's own ceiling, restated. On the QUIETEST of the five scenarios a value near it means a sign is wrong somewhere beneath this row" });
    }
    if !answer.F107bar_nominal.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107bar_nominal", reason: "the computation produced a value that is not a number" });
    }
    if answer.F107bar_nominal.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_nominal", value: answer.F107bar_nominal.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as the daily value it equals. An 81-day mean cannot sit below a floor every day of it respects" });
    }
    if answer.F107bar_nominal.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_nominal", value: answer.F107bar_nominal.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as the daily value it equals" });
    }
    if !answer.F107bar_hotmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107bar_hotmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.F107bar_hotmean.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_hotmean", value: answer.F107bar_hotmean.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as the daily value it equals: a *mean scenario IS its own 81-day mean" });
    }
    if answer.F107bar_hotmean.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_hotmean", value: answer.F107bar_hotmean.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as the daily value it equals" });
    }
    if !answer.F107bar_coldmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107bar_coldmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.F107bar_coldmean.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_coldmean", value: answer.F107bar_coldmean.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the same floor as the daily value it equals: a *mean scenario IS its own 81-day mean" });
    }
    if answer.F107bar_coldmean.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_coldmean", value: answer.F107bar_coldmean.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the same ceiling as the daily value it equals" });
    }
    if !answer.F107bar_hotday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107bar_hotday", reason: "the computation produced a value that is not a number" });
    }
    if answer.F107bar_hotday.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_hotday", value: answer.F107bar_hotday.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the hotmean level's own floor, because that is what this member is: a single day rides on the sustained level beneath it, and its 81-day mean is that level rather than the day's own value" });
    }
    if answer.F107bar_hotday.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_hotday", value: answer.F107bar_hotday.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the hotmean level's own ceiling. A *day scenario whose f107bar equalled its f107 would be claiming eighty-one consecutive days of the worst one, which the record does not support and the study does not publish" });
    }
    if !answer.F107bar_coldday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107bar_coldday", reason: "the computation produced a value that is not a number" });
    }
    if answer.F107bar_coldday.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_coldday", value: answer.F107bar_coldday.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the coldmean level's own floor, because that is what this member is: the quietest day still rides on the sustained cold level beneath it" });
    }
    if answer.F107bar_coldday.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107bar_coldday", value: answer.F107bar_coldday.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the coldmean level's own ceiling" });
    }
    if !answer.Ap_nominal.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_nominal", reason: "the computation produced a value that is not a number" });
    }
    if answer.Ap_nominal.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_nominal", value: answer.Ap_nominal.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero — a perfectly quiet day is Ap 0 — and there is nothing below it" });
    }
    if answer.Ap_nominal.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_nominal", value: answer.Ap_nominal.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the top of the Ap index itself; a value above it is not a geomagnetic index at all" });
    }
    if !answer.Ap_hotmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_hotmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.Ap_hotmean.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_hotmean", value: answer.Ap_hotmean.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero, so a sustained level below it means a spread has been subtracted rather than added beneath this row" });
    }
    if answer.Ap_hotmean.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_hotmean", value: answer.Ap_hotmean.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the top of the Ap index itself" });
    }
    if !answer.Ap_coldmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_coldmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.Ap_coldmean.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_coldmean", value: answer.Ap_coldmean.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero. On the quiet scenarios the floor is close: the producer's band subtracted from a centre below about 4.6 reaches through it" });
    }
    if answer.Ap_coldmean.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_coldmean", value: answer.Ap_coldmean.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the top of the Ap index itself" });
    }
    if !answer.Ap_hotday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_hotday", reason: "the computation produced a value that is not a number" });
    }
    if answer.Ap_hotday.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_hotday", value: answer.Ap_hotday.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero" });
    }
    if answer.Ap_hotday.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_hotday", value: answer.Ap_hotday.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the top of the Ap index itself, and this member — a sustained level with a daily excursion stacked on top — is the one in the set most likely to reach for it" });
    }
    if !answer.Ap_coldday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_coldday", reason: "the computation produced a value that is not a number" });
    }
    if answer.Ap_coldday.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_coldday", value: answer.Ap_coldday.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero, and this is the member in the whole subsystem closest to a declared bound: at the declared window it crosses at 6.91, seven units clear. Its producer's own sheet says a centre below 15.2 would put it through the floor" });
    }
    if answer.Ap_coldday.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_coldday", value: answer.Ap_coldday.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "400 is the top of the Ap index itself. On the QUIETEST scenario a value anywhere near it means a sign is wrong beneath this row" });
    }
    if !answer.Kp_mean_nominal.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_mean_nominal", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_mean_nominal.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_nominal", value: answer.Kp_mean_nominal.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_mean_nominal.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_nominal", value: answer.Kp_mean_nominal.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400" });
    }
    if !answer.Kp_mean_hotmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_mean_hotmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_mean_hotmean.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_hotmean", value: answer.Kp_mean_hotmean.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_mean_hotmean.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_hotmean", value: answer.Kp_mean_hotmean.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400" });
    }
    if !answer.Kp_mean_coldmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_mean_coldmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_mean_coldmean.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_coldmean", value: answer.Kp_mean_coldmean.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_mean_coldmean.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_coldmean", value: answer.Kp_mean_coldmean.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400" });
    }
    if !answer.Kp_mean_hotday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_mean_hotday", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_mean_hotday.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_hotday", value: answer.Kp_mean_hotday.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_mean_hotday.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_hotday", value: answer.Kp_mean_hotday.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400" });
    }
    if !answer.Kp_mean_coldday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_mean_coldday", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_mean_coldday.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_coldday", value: answer.Kp_mean_coldday.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_mean_coldday.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_mean_coldday", value: answer.Kp_mean_coldday.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400" });
    }
    if !answer.Kp_peak_nominal.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_peak_nominal", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_peak_nominal.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_nominal", value: answer.Kp_peak_nominal.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_peak_nominal.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_nominal", value: answer.Kp_peak_nominal.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400" });
    }
    if !answer.Kp_peak_hotmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_peak_hotmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_peak_hotmean.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_hotmean", value: answer.Kp_peak_hotmean.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_peak_hotmean.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_hotmean", value: answer.Kp_peak_hotmean.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400" });
    }
    if !answer.Kp_peak_coldmean.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_peak_coldmean", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_peak_coldmean.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_coldmean", value: answer.Kp_peak_coldmean.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_peak_coldmean.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_coldmean", value: answer.Kp_peak_coldmean.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400" });
    }
    if !answer.Kp_peak_hotday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_peak_hotday", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_peak_hotday.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_hotday", value: answer.Kp_peak_hotday.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_peak_hotday.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_hotday", value: answer.Kp_peak_hotday.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400" });
    }
    if !answer.Kp_peak_coldday.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Kp_peak_coldday", reason: "the computation produced a value that is not a number" });
    }
    if answer.Kp_peak_coldday.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_coldday", value: answer.Kp_peak_coldday.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's first point is Kp 0 at ap 0; a negative index is a sign error, not a quiet sky" });
    }
    if answer.Kp_peak_coldday.get() > 9.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Kp_peak_coldday", value: answer.Kp_peak_coldday.get(), bound: 9.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "Kp is defined on 0 to 9 and the scale's last point is Kp 9 at ap 400" });
    }
    Ok(answer)
}
