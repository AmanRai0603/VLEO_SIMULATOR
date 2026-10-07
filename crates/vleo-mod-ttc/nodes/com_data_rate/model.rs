// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// How fast can data come down while keeping the required margin?
///
/// `R = 10^((C/N0 - EbN0_req - L_imp - M)/10)`
///
/// Source: `larson_wertz`
pub const NODE_ID: &str = "com_data_rate";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x93298b4ccd59e1b5;

pub fn evaluate(c: Ratio, er: Ratio, li: Ratio, m: Ratio) -> Result<DataRate, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::com_data_rate. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: DataRate = match methods::com_data_rate::evaluate(c.get(), er.get(), li.get(), m.get()) {
        Ok(v) => DataRate::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "R_b")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: DataRate = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "R_b", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_b", value: answer.get(), bound: 1000.0, edge: Edge::Lower, unit: DataRate::UNIT, reason: "below 1 kbit/s the downlink cannot even carry housekeeping" });
    }
    if answer.get() > 100000000000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "R_b", value: answer.get(), bound: 100000000000.0, edge: Edge::Upper, unit: DataRate::UNIT, reason: "above 100 Gbit/s no spacecraft modem in this class runs" });
    }
    Ok(answer)
}
