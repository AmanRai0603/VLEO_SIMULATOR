// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
#![allow(unused_imports, unused_variables, unused_parens, clippy::let_and_return, clippy::approx_constant, clippy::too_many_arguments)]

use vleo_core::fault::{Edge, Fault};
use vleo_core::physics::*;
use vleo_core::units::pmath;
use vleo_core::units::*;

/// What daily 10.7 cm radio flux is the design sized to?
///
/// `F107 = F107_sys_daily`
///
/// Source: `orbitt_case_c1`
///
/// The number the density chain is built on. It is no longer declared here:
/// the solar-weather subsystem computes it over the mission window and
/// publishes it through l3_solar_interface, layer 2 carries it as
/// sys_space_environment_f10_7, and this row carries it into the subsystems
/// that read it. It is a SUSTAINED level over the window, not the flux on a
/// day. What the Sun is doing now is a different question and a different row
/// — sw_f107_observed, which is what the subsystem's own forecast starts
/// from.
pub const NODE_ID: &str = "env_f107";
/// Hash of the sheet this file was generated from. A face carrying a
/// different one refuses to run rather than showing a stale page.
pub const SHEET_HASH: u64 = 0x4be050189834fcca;

pub fn evaluate(from_system: Ratio) -> Result<Ratio, Fault> {
    // ---- HOLE 1 : carry the system layer's daily flux through unchanged -> Ratio
    let flux: Ratio = from_system;
    // ---- end HOLE 1

    // generated · the declared domain of this node's own answer. The
    // reason travels with the guard, because a guard whose reason is not
    // written down gets deleted by the next person who finds it awkward.
    let answer: Ratio = flux;
    if !answer.is_finite() {
        return Err(Fault::Degenerate { node: NODE_ID, field: "F107", reason: "the computation produced a value that is not a number" });
    }
    if answer.get() < 60.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107", value: answer.get(), bound: 60.0, edge: Edge::Lower, unit: Ratio::UNIT, reason: "below 60 sfu has never been observed; the fit has no support there" });
    }
    if answer.get() > 400.0 {
        return Err(Fault::OutOfDomain { node: NODE_ID, field: "F107", value: answer.get(), bound: 400.0, edge: Edge::Upper, unit: Ratio::UNIT, reason: "above 400 sfu is beyond the largest recorded daily value, so the temperature relation is extrapolated" });
    }
    Ok(answer)
}
