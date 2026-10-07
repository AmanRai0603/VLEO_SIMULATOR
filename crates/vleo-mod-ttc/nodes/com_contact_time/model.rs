// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How long is one ground station pass?
///
/// `t = T_orb*lambda/pi`
///
/// Source: `larson_wertz`
///
/// # Assumptions
///
/// * A pass directly overhead — fails when the mean over many passes is roughly 60% of this, so a downlink volume computed from it will not be achieved
pub const NODE_ID: &str = "com_contact_time";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x7db1404e9b5c3e38;

pub fn evaluate(lam: Angle, p: Time) -> Result<Time, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::com_contact_time. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Time = match methods::com_contact_time::evaluate(lam.get(), p.get()) {
        Ok(v) => Time::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "t_con")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Time = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "t_con", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_con", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Time::UNIT, reason: "a contact time cannot be negative" });
    }
    if answer.get() > 3600.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "t_con", value: answer.get(), bound: 3600.0, edge: Edge::Upper, unit: Time::UNIT, reason: "above an hour is not a low-orbit pass" });
    }
    Ok(answer)
}
