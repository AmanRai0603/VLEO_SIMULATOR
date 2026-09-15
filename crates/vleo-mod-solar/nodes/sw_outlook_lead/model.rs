// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// At what lead is the published short-term outlook verified?
///
/// `L_short = 26 d`
///
/// Source: `noaa_swpc`
///
/// The lead the forecast-verification rows are evaluated at.
/// sw_forecast_skill and sw_forecast_bias both answer 'at what lead', and
/// this is the lead worth asking about: the far end of the published window,
/// where the outlook is weakest and a design reading it is most exposed. It
/// is the far end the record can measure, not the nominal far end of the
/// product — see the note above on the two lead conventions in the source
/// column.
///
/// # Assumptions
///
/// * The product is 27 days long and this row is 26, so it does not answer 'how far ahead does the outlook reach' — fails when somebody reads it as the window length. 899 of 1281 issues span exactly 27 days and the synodic solar rotation is about 27.3 days, so 27 is the honest answer to that question and this row is not asking it. It is asking where the outlook is weakest among the leads the record can verify, and the 27th lead cannot be verified because only 192 of 1281 issues place a row there. If a row is ever needed for the window length it is a second row, not this one.
/// * One lead, and the skill of the outlook is strongly lead-dependent — fails when measured on this record the outlook's skill against persistence rises from +0.0685 at lead 1 to a peak of +0.4375 at lead 9, falls back through +0.0177 at lead 23, and goes negative at 24, 25 and 26. So a single lead cannot characterise the product: 26 is the pessimistic end and a row evaluated there says nothing about the useful middle. It is chosen because a design wants to know how bad the far edge is, not how good the centre is.
/// * Skill is measured against persistence defined as the last observation STRICTLY BEFORE the issue date — fails when the baseline is allowed the observation on the issue date itself. That is the lead-0 target for the 719 issues that index from 0, so it hands persistence an answer the forecaster did not have, and it flatters the baseline enormously: the same arithmetic then reports the lead-1 skill as -1.3138 instead of +0.0685, and the outlook appears to lose to persistence at leads 1 through 4 when it does not. One choice of baseline, and the sign of the short-lead conclusion changes. This row's own first version carried the leaky figure and said the outlook loses at short leads; it does not.
pub const NODE_ID: &str = "sw_outlook_lead";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x0543d6ab410d48b3;

pub fn evaluate() -> Result<Time, Fault> {
    // generated · a declared value, converted from the unit it was written in
    let declared: Time = match Time::from_unit(26.0, Unit::Day) {
        Some(q) => q,
        None => return Err(Fault::Degenerate { node: NODE_ID, field: "L_short", reason: "the declared unit does not match the declared type" }),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = declared;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "L_short", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 86400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L_short", value: answer.get(), bound: 86400.0, edge: Edge::Lower, unit: Time::UNIT, reason: "a lead of less than a day is not a forecast the published outlook makes; its first verifiable row is lead 1, present for all 1281 issues" });
    }
    if answer.get() > 2246400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "L_short", value: answer.get(), bound: 2246400.0, edge: Edge::Upper, unit: Time::UNIT, reason: "26 is the last lead in the published window that the record can verify, because lead_days is indexed two ways in the same column and only the 1-based minority reaches 27. At 26 the sample is 868 issues drawn from both conventions; at 27 it is 192 drawn from one, and the answer there has a different sign from each of its neighbours. A larger value would be asking the verification rows about a cell whose contents are a property of the indexing rather than of the forecast" });
    }
    Ok(answer)
}
