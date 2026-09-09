//! RING 1 — the kernel.
//!
//! Everything a number in this system is made of lives here: every formula,
//! the fault taxonomy, the evidence and credibility model, the graph types and
//! the resolver that walks them.
//!
//! # The four rules this crate is shaped by
//!
//! * **P1 — one implementation of every formula.** No face, renderer, binding
//!   or shader re-derives a number. If a formula appears outside this crate,
//!   `cargo xtask gate` fails the build.
//! * **P2 — inputs immutable, evaluation pure.** [`resolver::evaluate`] is a
//!   function of the case and the graph and nothing else, which is what makes
//!   staleness computable rather than remembered.
//! * **P3 — units are types.** Enforced one ring down, in `vleo-units`.
//! * **P4 — the kernel cannot draw, allocate, or read a file.** This is
//!   structural, not a review comment: the crate is `no_std`, declares no
//!   allocator, and has no renderer or filesystem crate anywhere in its
//!   dependency tree. There is exactly one dependency, and it is a leaf.
//!
//! # What is *not* here
//!
//! No node identifiers, no edges, no per-node code. Those are generated from
//! the sheets into `vleo-graph` and `vleo-mod-*`. This crate holds the types
//! they are expressed in and the relations they compose, so it can be read,
//! reviewed and tested without knowing which nodes exist.
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(clippy::float_cmp)]

pub mod credibility;
pub mod evidence;
pub mod fault;
pub mod graph;
pub mod hash;
pub mod math;
pub mod physics;
pub mod resolver;
pub mod value;

pub use credibility::{CredVec, Factor, Tier};
pub use evidence::{Provenance, Verdict};
pub use fault::Fault;
pub use graph::{Kind, NodeDef, NodeIdx, NodeTable, State, VarDef, VarIdx};
pub use resolver::{Mode, RunReport, Workspace};
pub use value::{Slot, SlotStatus, Store};

/// Re-exported so a node's generated file has exactly one crate to import.
pub use vleo_units as units;
