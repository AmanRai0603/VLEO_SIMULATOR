//! LAYER 3 — the fifteen subsystem layers — one team each, reached only through an interface node.
//!
//! # Why a crate with nothing in it
//!
//! The rows are here. Every one has a folder, a sheet, a place on the tree and
//! eight tabs that open and say what goes in each of them. What is missing is
//! the content, and nothing is generated from a sheet nobody has specified —
//! a file full of `todo!()` would hide that behind something that looks like
//! work.
//!
//! The crate exists so that the moment a row is specified there is somewhere
//! for its implementation to go, and so the module list that wires them in is
//! already being built.
#![forbid(unsafe_code)]

/// The nodes this layer owns. Empty until the first row is specified.
pub mod nodes {
    include!(concat!(env!("OUT_DIR"), "/nodes.rs"));
}
