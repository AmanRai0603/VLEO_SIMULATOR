// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// Is the published outlook better than assuming today's F10.7 continues?
///
/// `S_f107(L) = 1 - MSE_outlook(L) / MSE_persistence(L)`
///
/// Source: `noaa_swpc`
///
/// A skill score against persistence: 1 is perfect, 0 is no better than the
/// naive baseline, negative is worse than doing nothing. It answers whether
/// reading the outlook is worth anything at the lead sw_outlook_lead
/// declares, which is the far end of the window where the answer is worst.
///
/// # Assumptions
///
/// * Persistence is the last observation STRICTLY BEFORE the issue date, and this choice decides the answer — fails when the baseline is allowed the observation on the issue date itself. 719 of the 1281 issues index their rows from lead 0, so the issue date IS a forecast target for most of the record, and handing it to the baseline gives persistence a number the forecaster did not have. The whole short-lead conclusion turns on it: the same arithmetic then reports -1.314 at lead 1 instead of +0.069, and the outlook appears to lose to persistence through lead 4 when it does not. A skill score is a statement about a baseline, so the baseline is declared here rather than left to whoever reads the number
/// * It goes negative at the far end of the window, and that is the answer, not a defect — fails when the last three verifiable leads are read as noise. Skill is positive from lead 1 through lead 23, peaks at +0.438 at lead 9, and is negative at leads 24, 25 and 26 — -0.036, -0.032 and -0.022 against samples of 866 to 868 pairs each. At the far end of its own published window the outlook is very slightly worse than assuming nothing changes. A design keying off the end of the outlook is paying attention to a forecast that has stopped carrying information
/// * Skill against persistence is not accuracy — fails when a positive score is read as the forecast being good. The outlook's own RMS error grows from 10.3 sfu at lead 1 to 24.9 sfu at lead 26; what improves through the middle leads is only its ratio to a baseline that degrades faster. Peak skill of +0.438 at lead 9 sits on an RMS error of 21.2 sfu, which is 18% of a typical F10.7. The forecast is never accurate in the window; it is merely better than nothing for most of it
/// * One score over 29 years, pooled across cycles — fails when the skill is activity-dependent, which it will be: persistence is a strong baseline in a quiet Sun and a weak one in a rising cycle, so pooling cycles 23, 24 and the rise of 25 averages over regimes where the comparison means different things. The sample is 866 to 1257 pairs per lead and is not conditioned on phase. A phase-conditioned skill would be a separate row and would need the epoch, which now exists
pub const NODE_ID: &str = "sw_forecast_skill";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc050bee28e39a8fc;

pub fn evaluate(lead: Time) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::sw_forecast_skill. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::sw_forecast_skill::evaluate(lead.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "S_f107")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "S_f107", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < -0.1 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "S_f107", value: answer.get(), bound: -0.1, edge: Edge::Lower, unit: Ratio::UNIT, reason: "the worst measured skill is -0.036, at lead 24. A bound at -0.1 leaves room for the three negative leads and refuses anything that would say the published outlook is substantially worse than doing nothing, which the record does not support" });
    }
    if answer.get() > 0.5 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "S_f107", value: answer.get(), bound: 0.5, edge: Edge::Upper, unit: Ratio::UNIT, reason: "the best measured skill is +0.438, at lead 9. A skill above 0.5 against persistence would mean the outlook halves the baseline's mean squared error, and nothing in this record comes close; an answer there means the table was misread or the bundle changed underneath it" });
    }
    Ok(answer)
}
