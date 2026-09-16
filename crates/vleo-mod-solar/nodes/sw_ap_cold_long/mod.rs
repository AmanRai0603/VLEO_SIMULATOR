// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! `sw_ap_cold_long` — Sustained cold Ap to design to
//!
//! What Ap must the design still work at as a sustained level over the mission window, on the low side?

#[path = "model.rs"]
pub mod model;
#[path = "contract.rs"]
pub mod contract;
#[cfg(test)]
#[path = "evidence.rs"]
mod evidence;

pub use contract::{call, INPUT_VARS, NODE_ID, OUTPUT_UNIT, OUTPUT_VARS, SHEET_HASH};
