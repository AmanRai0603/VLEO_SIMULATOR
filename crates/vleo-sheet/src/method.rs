//! THE METHOD LANGUAGE — the pseudocode a node's engineer writes.
//!
//! A node's relation arrives in two independent forms: the author's own code,
//! which produced their test cases, and this method, which says the same thing
//! in a small fixed language and is what the engine runs. Each checks the
//! other, so the language has to be small enough that every construct has
//! exactly one meaning, and strict enough that a unit mistake is caught before
//! the method runs.
//!
//! ```text
//! # The speed of a circular orbit at altitude h.
//! let r = R_EARTH + h
//! if h < 0 [m] then
//!   refuse "the altitude is below the ground"
//! end
//! let v : Velocity = sqrt(MU_EARTH / r)
//! return v
//! ```
//!
//! Every value is a number in the SI base unit of its dimension, carrying that
//! dimension. A literal says its unit in brackets (`6371 [km]`) and is
//! converted on reading, so `h < 0 [m]` and `h < 0 [km]` are the same test.
//! The node's declared inputs arrive as variables of their quantity's
//! dimension; the kernel constants below are available by name. The checker
//! refuses a sum of unlike dimensions, a transcendental function of a
//! dimensioned value, a return of the wrong dimension, and a path that
//! neither returns nor refuses.
//!
//! One implementation: the gate, intake, the pages that check a method and
//! the engine that runs it all read this module, so a page and the gate cannot
//! disagree about what a method means. `docs/PSEUDOCODE.md` is written from
//! [`FUNCTIONS`], [`KERNEL_CONSTANTS`] and [`STATEMENTS`] by `xtask docs`.

use crate::{Error, ErrorKind};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use vleo_units::method_rt as rt;
use vleo_units::unit::Dim;
use vleo_units::{constants as k, pmath, Unit};

/// The language version. A method is read by the version it was written for;
/// raising this is a reviewed change to every node that has one.
///
/// 2 added `while … at most N times` and the kernel functions; 3 added
/// `publish`, for a node that publishes several values; 4 added lists — a
/// `const` list written out, read by entry, by `for … in`, by `len` and by
/// `interp`. Every method of an earlier version reads the same in a later one.
pub const LANGUAGE_VERSION: u32 = 4;

/// How many loop iterations a method may run in all. A method is a relation,
/// not a simulation: a loop exists for a short series or a fixed-point
/// refinement, and a method that needs more than this is a node that should be
/// split.
pub const MAX_STEPS: u64 = 100_000;

// ---------------------------------------------------------------------------
// diagnostics

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    /// Worth saying, and not a reason to refuse: an input the method never
    /// reads, for instance.
    Warning,
}

/// One finding, on one line of the method.
#[derive(Clone, Debug, PartialEq)]
pub struct Diag {
    /// 1-based. 0 means the method as a whole.
    pub line: usize,
    pub severity: Severity,
    pub msg: String,
}

impl Diag {
    fn err(line: usize, msg: impl Into<String>) -> Diag {
        Diag {
            line,
            severity: Severity::Error,
            msg: msg.into(),
        }
    }
    fn warn(line: usize, msg: impl Into<String>) -> Diag {
        Diag {
            line,
            severity: Severity::Warning,
            msg: msg.into(),
        }
    }
}

impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.severity {
            Severity::Error => "",
            Severity::Warning => "note: ",
        };
        if self.line == 0 {
            write!(f, "{kind}{}", self.msg)
        } else {
            write!(f, "line {}: {kind}{}", self.line, self.msg)
        }
    }
}

// ---------------------------------------------------------------------------
// the parts, one file each, under src/method/

mod ast;
mod cases;
mod check;
mod kernel;
mod node;
mod parse;
mod reference;
mod run;
mod spec;
#[cfg(test)]
mod tests;
mod units;

pub use ast::*;
pub use cases::*;
pub use check::*;
pub use kernel::*;
pub use node::*;
pub use parse::*;
pub use reference::*;
pub use run::*;
pub use spec::*;
pub use units::*;
