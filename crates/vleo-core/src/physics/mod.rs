//! The physics.
//!
//! Every relation in the system is here and nowhere else. A node's generated
//! implementation composes these into its own answer; it holds no formula of
//! its own, and `cargo xtask gate` fails the build if one appears outside this
//! module.
//!
//! The split is by what the quantity is about, not by which team owns it:
//! ownership is a path rule in the repository, and a formula that moved because
//! a team reorganised is a formula whose history was lost.

pub mod aero;
pub mod comms;
pub mod cost;
pub mod env;
pub mod gnc;
pub mod mass;
pub mod mission;
pub mod orbit;
pub mod payload;
pub mod power;
pub mod prop;
pub mod thermal;
