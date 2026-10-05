// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What daily planetary Ap must the design survive as a sustained level over the mission window?
///
/// `Ap_long = Ap_central + 1.28 * sigma_ap`
///
/// Source: `noaa_swpc`
///
/// The Ap twin of sw_f107_design_long, and NOT the same question as
/// sw_ap_design. That row asks what extreme recurs once per mission; this
/// asks what level the window sits at. A design needs both and they are
/// different numbers by a factor of several.
pub const NODE_ID: &str = "sw_ap_design_long";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0xc8eba9d10fc181e9;

pub fn evaluate(central: Ratio, spread: Ratio) -> Result<Ratio, Fault> {
    // generated · from the node's method, translated by rule into
    // vleo_core::physics::methods::sw_ap_design_long. No hole: the method is the
    // implementation, and the author's cases in evidence.rs test it.
    let method_answer: Ratio = match methods::sw_ap_design_long::evaluate(central.get(), spread.get()) {
        Ok(v) => Ratio::new(v),
        Err(e) => return Err(method::fault(e, NODE_ID, "Ap_long")),
    };

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = method_answer;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "Ap_long", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 0.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_long", value: answer.get(), bound: 0.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "Ap floors at zero — a perfectly quiet day is Ap 0 — so a design level below it means a spread has been subtracted rather than added" });
    }
    if answer.get() > 300.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "Ap_long", value: answer.get(), bound: 300.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 300 the level exceeds the largest daily Ap in the record, 273, so a SUSTAINED level there is not a window this tool can model" });
    }
    Ok(answer)
}
