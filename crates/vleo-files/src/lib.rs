//! The one library that reads, writes and checks every design file.
//!
//! Every file a person keeps — a node, a group, a release, a released design,
//! a case, a result, a key — is read, written and checked here, and only here
//! (docs/OPERATING_1_0.md, section 10). The installed application links it and
//! the page runs it compiled to WebAssembly, so a check that refuses a file in
//! one refuses it in the other, by the same code.
//!
//! It is built in parts (docs/PLAN_1_0.md, phase C):
//!
//! * [`keys`] — section 14: a key made, locked by its owner's passphrase, a
//!   signature made with it, and a signature checked;
//! * [`model`] and [`meta`] — sections 10 and 13: the one schema every file
//!   is (`schema.sql`), a file as its rows, and what each kind of file must
//!   say about itself, its version and what it was based on;
//! * `sqlite` — the file on disk, for the installed application; the page
//!   hands the same rows from its own SQLite;
//! * [`format_1`] and [`upgrade`] — section 16: a group's file from before 1.0
//!   read as it is, its folder and the fingerprint its sign-offs were given
//!   for, and upgraded to this format with nothing dropped; a release from
//!   before 1.0 anchored by the programme manager, by its fingerprint;
//! * [`chain`] — section 14: what a signature signs, and the chain it checks
//!   through, from the anchored programme manager to every node;
//! * [`rows`] and [`page`] — a file's rows as bytes, and everything the page
//!   asks the library, answered by the same functions the native tests run;
//! * [`csv`] — the group folder's CSV, written byte for byte as the page
//!   writes it.
//!
//! * [`checks`] — what a release must hold before it is taken: the checks
//!   today's intake makes of its content, one test for each refusal.
//!
//! The method checks — a method that does not reproduce its own cases — and
//! the rest of the page's folder checks come next, on top of these.

pub mod chain;
pub mod checks;
pub mod csv;
pub mod error;
pub mod format_1;
pub mod keys;
pub mod meta;
pub mod model;
pub mod page;
pub mod rows;
#[cfg(not(target_arch = "wasm32"))]
pub mod sqlite;
pub mod upgrade;

pub use error::{Error, ErrorKind};
