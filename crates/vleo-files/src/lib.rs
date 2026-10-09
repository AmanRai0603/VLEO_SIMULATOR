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
//!   today's intake makes of its content, its methods read and run by the
//!   method language against their own cases, one test for each refusal.
//!
//! * [`folder`] — the group folder's checks, the page's group checker
//!   (`web/js/gcheck.js`) said by the library against the same pattern
//!   (`groups/SPEC.toml`), held to it finding for finding.
//!
//! * [`compare`] — any two files, block by block: two revisions, two
//!   releases, two designs, each row matched by its key and given to its
//!   block, a block matched by its uid through a rename;
//!
//! * [`convert`] — the design itself: the files in `design/`, each branch's
//!   group file, a node file per row and each case, read as the folders they
//!   were converted from, by the one reader every face reads the design
//!   through ([`convert::open`]);
//!
//! * [`n2`] — a block's N2, drawn from the files' wires, and every loop they
//!   make: the block it belongs on, and whether that block declares it;
//!
//! * [`seal`] — the seal of a group's folder from before 1.0: its sign-offs,
//!   current or stale against the fingerprint they were given for, who may
//!   sign what, and what stands between the folder and its seal, as the
//!   page's seal rules (`web/js/gseal.js`) have them.

pub mod chain;
pub mod checks;
pub mod compare;
pub mod convert;
pub mod csv;
pub mod error;
pub mod folder;
pub mod format_1;
pub mod intake;
pub mod keys;
pub mod meta;
pub mod model;
pub mod n2;
pub mod page;
pub mod rows;
pub mod seal;
#[cfg(not(target_arch = "wasm32"))]
pub mod sqlite;
pub mod upgrade;

pub use error::{Error, ErrorKind};
