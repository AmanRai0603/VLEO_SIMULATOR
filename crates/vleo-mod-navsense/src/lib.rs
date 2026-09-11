//! LAYER 3 - Navigation sensing
//!
//! One subsystem layer. It answers its targets from the system layer, and is reached through its interface node rather than by reaching into it.
//!
//! # Why this is its own crate
//!
//! Inside one crate `use crate::prop::..` from `power` compiles and the
//! isolation rule is unenforced. A separate crate makes it a manifest line: a
//! sibling this crate did not declare will not build. The faces still take one
//! dependency on the facade, so the compiler still sees independent units and
//! still builds them in parallel.
//!
//! # What is in here
//!
//! Nothing but generated node modules, and for most layers not even those yet.
//! Every formula lives one ring down in `vleo-core`; a node composes them in
//! its `model.rs` and holds no formula of its own, and `cargo xtask gate`
//! fails the build if one appears here.
#![forbid(unsafe_code)]

/// The nodes this layer owns. One module per folder, wired at build time.
pub mod nodes {
    include!(concat!(env!("OUT_DIR"), "/nodes.rs"));
}
