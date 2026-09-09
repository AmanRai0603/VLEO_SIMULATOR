//! The sheet, and everything derived from it.
//!
//! One file per node is written by hand — `node.toml` — and eleven artefacts
//! are printed from it: the implementation scaffold, the contract, the module
//! wiring, the documentation fragment, the test harness, the metadata, the gap
//! report, the registry, the graph, the index and the assembled document.
//!
//! This crate is the single implementation of what a sheet *means*. Both the
//! generators in `xtask` and the build scripts that compile the tables read it,
//! so a build and a `cargo xtask docs` can never disagree about the graph. Two
//! implementations of that would be two standards.
//!
//! # Two rules the generators are held to
//!
//! * **The six per-node generators never read another node.** That is what
//!   makes 250 nodes 250 independent acts: five engineers can add a node on the
//!   same afternoon and no file is touched twice.
//! * **The assembly generators never decide anything.** They combine and they
//!   refuse. A decision taken during assembly is a decision nobody reviewed,
//!   because assembly has no diff.

pub mod emit;
pub mod gate;
pub mod load;
pub mod model;
pub mod page;

pub use load::{load_all, Tree};
pub use model::{Assumption, Fixture, Sheet, Step, View};

/// FNV-1a, the same function the kernel uses, so a hash written by a generator
/// and a hash computed by the engine are the same number.
pub fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

pub fn short_hex(h: u64) -> String {
    format!("{:06x}", h & 0xff_ffff)
}
