//! RING 3 — mission performance and the KPI closures.
//!
//! Coverage, revisit, latency, availability — and the required-against-achieved comparison, identical at every boundary.
//!
//! # Why this is its own crate
//!
//! Inside a single crate `use crate::prop::..` from `power` compiles, and the
//! isolation rule is unenforced. A separate crate makes it a manifest line:
//! a sibling this crate did not declare will not compile. The faces still take
//! one dependency on the facade, and the compiler still sees twelve units, so
//! they build in parallel.
//!
//! # What is in here
//!
//! Nothing but generated node modules. Every formula lives one ring down in
//! `vleo-core`; a node's `model.rs` composes them and holds no formula of its
//! own, and `cargo xtask gate` fails the build if one appears here.
#![forbid(unsafe_code)]

/// The nodes this subsystem owns. One module per folder, wired at build time.
pub mod nodes {
    include!(concat!(env!("OUT_DIR"), "/nodes.rs"));
}
