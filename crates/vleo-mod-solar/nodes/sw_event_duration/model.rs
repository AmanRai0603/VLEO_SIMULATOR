// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Once F10.7 goes above the spike threshold, how long does it stay there?
///
/// `D_burst = mean length of a run of consecutive days with F107/F107A >= S_thr = 3.3115 d`
///
/// Source: `noaa_swpc`
///
/// The dwell that turns a threshold into an event. A design cares about
/// duration rather than about the crossing, because a single elevated day and
/// a week of them are different loads on a drag budget and on a power one.
///
/// # Assumptions
///
/// * A mean over a distribution that is not remotely symmetric — fails when the mean is used as a typical event. The 202 spike days fall into 61 runs: 24 of them are a single day, and the tail runs to 10. The distribution is 24 ones, 3 twos, 10 threes, 7 fours, 4 fives, 7 sixes, 2 sevens, 2 nines and 2 tens — so the modal event is one day and the mean is 3.31 because a handful of long bursts pull it up. The median is 3. A design that sizes on 3.31 days is sizing on neither the common case nor the bad one, and the bad one is what matters
/// * It is defined entirely by sw_spike_threshold and moves when that moves — fails when this row is quoted without the threshold beside it. At the declared 1.3049 the record gives 61 events averaging 3.31 days; a lower threshold merges neighbouring runs into longer ones and a higher one splits them. The two rows are one definition in two places, and a change to either without the other makes the pair incoherent
/// * A one-day gap ends an event, and that is a choice with no physics behind it — fails when the sky dips below the threshold for a day and comes back. Runs are broken on the first day that fails the test, so a two-week episode with one quiet day in the middle is counted as two events rather than one, shortening the mean. Allowing a one-day bridge would be as defensible and would give a different answer; nothing in the record says which is right, and this row states the rule rather than pretending the number is unique
/// * The 273 absent days can neither start nor end a run — fails when an event straddled 2017. observed_daily.csv is missing 2017-01-01 to 2017-09-30, so a burst in those nine months is absent, and a burst that ran into 2017-01-01 or out of 2017-09-30 is truncated at the gap rather than followed. With 61 events over the record, one or two truncations would move the mean by a tenth of a day, which is the order of the effect and is not corrected for
pub const NODE_ID: &str = "sw_event_duration";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xb9465a7d1124ed52;

pub fn evaluate() -> Result<Time, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Time = match Time::from_unit(3.3114754098, Unit::Day) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "D_burst", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "D_burst", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 86400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D_burst", value: answer.get(), bound: 86400.0, edge: Edge::Lower, unit: Time::UNIT, reason: "a run is at least one day by construction, because a run of zero days is not an event. A value below 1 means the run-finding is broken rather than that events are short" });
    }
    if answer.get() > 864000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "D_burst", value: answer.get(), bound: 864000.0, edge: Edge::Upper, unit: Time::UNIT, reason: "the longest run in 29 years is 10 days. A mean above it would exceed every single event the record contains, which no averaging can produce" });
    }
    Ok(answer)
}
