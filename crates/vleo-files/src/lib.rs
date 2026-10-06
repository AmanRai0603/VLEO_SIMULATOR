//! The one library that reads, writes and checks every design file.
//!
//! Every file a person keeps — a node, a group, a release, a released design,
//! a case, a result, a key — is read, written and checked here, and only here
//! (docs/OPERATING_1_0.md, section 10). The installed application links it and
//! the page runs it compiled to WebAssembly, so a check that refuses a file in
//! one refuses it in the other, by the same code.
//!
//! It is built in parts (docs/PLAN_1_0.md, phase C). This is the first: the
//! keys of section 14 — a key made, locked by its owner's passphrase, a
//! signature made with it, and a signature checked. The schema every file
//! shares, the chain a signature checks through and the checks a release must
//! pass come next, on top of it.

pub mod error;
pub mod keys;

pub use error::{Error, ErrorKind};
