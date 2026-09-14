// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! `sw_event_duration` — F10.7 burst duration
//!
//! Once F10.7 goes above the spike threshold, how long does it stay there?

#[path = "model.rs"]
pub mod model;
#[path = "contract.rs"]
pub mod contract;
#[cfg(test)]
#[path = "evidence.rs"]
mod evidence;

pub use contract::{call, INPUT_VARS, NODE_ID, OUTPUT_UNIT, OUTPUT_VARS, SHEET_HASH};
