// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What 81-day mean 10.7 cm radio flux is the design sized to?
///
/// `F107A = F107bar_sys`
///
/// Source: `orbitt_case_c1`
///
/// The slow solar term the thermosphere relation weights 3.24 — the
/// dominant one. Computed by the solar-weather subsystem over the mission
/// window and carried here through layer 2, for the same reason as env_f107:
/// one row owns the answer and the rest read it.
pub const NODE_ID: &str = "env_f107a";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x6474789c1e46c8fd;

pub fn evaluate(from_system: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : carry the system layer's 81-day mean flux through unchanged -> Ratio
    let mean_flux: Ratio = from_system;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = mean_flux;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107A", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107A", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "same support limit as the daily value" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107A", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "same support limit as the daily value" });
    }
    Ok(answer)
}
