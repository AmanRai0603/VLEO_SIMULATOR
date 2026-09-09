// GENERATED from node.toml by `cargo xtask docs`. Do not edit outside a
// numbered HOLE block: a hand edit anywhere else is discarded by the next
// regeneration and fails the regeneration diff in the gate.
//! `aero_cp_cm_offset` — Centre of pressure to centre of mass offset
//!
//! How far behind the centre of mass does the aerodynamic force act?

#[path = "model.rs"]
pub mod model;
#[path = "contract.rs"]
pub mod contract;
#[cfg(test)]
#[path = "evidence.rs"]
mod evidence;

pub use contract::{call, INPUT_VARS, NODE_ID, OUTPUT_UNIT, OUTPUT_VARS, SHEET_HASH};
