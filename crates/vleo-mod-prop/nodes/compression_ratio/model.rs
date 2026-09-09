// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// By how much does the intake concentrate the gas before it reaches the thruster?
///
/// `CR = 4*V*A_in*eta_geo/(v_bar*(A_out + beta*A_in))`
///
/// Source: `romano2021`
pub const NODE_ID: &str = "prop_compression_ratio";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x11c51ef5b8338a0c;

pub fn evaluate(a_in: Area, a_out: Area, eta_geo: Ratio, beta: Ratio, t_c: Temperature, m: MolarMass, n: NumberDensity, v: Velocity) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : take the chamber-to-free-stream density ratio from the same flux balance -> Ratio
    let r: Ratio = prop::intake_balance(n, v, a_in, a_out, eta_geo, beta, t_c, m).compression_ratio;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = r;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "CR", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 1.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "CR", value: answer.get(), bound: 1.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "an intake cannot dilute the flow; a compression ratio below one means the geometry is inverted" });
    }
    if answer.get() > 100000.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "CR", value: answer.get(), bound: 100000.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 1e5 the chamber gas becomes collisional and the free-molecular balance no longer holds" });
    }
    Ok(answer)
}
