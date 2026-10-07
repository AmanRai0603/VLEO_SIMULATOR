// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What does the spacecraft weigh with no propellant on board?
///
/// `m_dry = (sum of subsystem masses)*(1 + M_sys)`
///
/// Source: `ecss_e_st_10_02`
pub const NODE_ID: &str = "mass_dry";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x08d40cc20b54f17e;

pub fn evaluate(st: Mass, pr: Mass, pw: Mass, th: Mass, av: Mass, cm: Mass, gn: Mass, pa: Mass, mg: Ratio) -> Result<Mass, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::mass_dry. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Mass = match methods::mass_dry::evaluate(st.get(), pr.get(), pw.get(), th.get(), av.get(), cm.get(), gn.get(), pa.get(), mg.get()) {
        Ok(v) => Mass::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "m_dry")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Mass = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "m_dry", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_dry", value: answer.get(), bound: 1.0, edge: Edge::Lower, unit: Mass::UNIT, reason: "below a kilogram there is no spacecraft" });
    }
    if answer.get() > 5000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "m_dry", value: answer.get(), bound: 5000.0, edge: Edge::Upper, unit: Mass::UNIT, reason: "above 5 tonnes the vehicle is not in this class" });
    }
    Ok(answer)
}
