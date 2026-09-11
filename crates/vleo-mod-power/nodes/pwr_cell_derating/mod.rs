// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! `pwr_cell_derating` — Array temperature derating factor
//!
//! How much output does the array lose because it is hotter than its rating?

#[path = "model.rs"]
pub mod model;
#[path = "contract.rs"]
pub mod contract;
#[cfg(test)]
#[path = "evidence.rs"]
mod evidence;

pub use contract::{call, INPUT_VARS, NODE_ID, OUTPUT_UNIT, OUTPUT_VARS, SHEET_HASH};
