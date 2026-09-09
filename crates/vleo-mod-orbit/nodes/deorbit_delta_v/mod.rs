// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! `orbit_deorbit_delta_v` — Disposal delta-v
//!
//! What does the end-of-life disposal manoeuvre cost, as required by the debris mitigation standard?

#[path = "model.rs"]
pub mod model;
#[path = "contract.rs"]
pub mod contract;
#[cfg(test)]
#[path = "evidence.rs"]
mod evidence;

pub use contract::{call, INPUT_VARS, NODE_ID, OUTPUT_UNIT, OUTPUT_VARS, SHEET_HASH};
