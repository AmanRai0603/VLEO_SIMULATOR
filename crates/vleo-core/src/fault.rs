//! The fault taxonomy.
//!
//! A node returns `Result<_, Fault>`, never a sentinel and never a quiet NaN.
//! A wrong answer that keeps travelling is the failure this whole system exists
//! to prevent, so a function that can fail returns something the caller must
//! open before it can use.
//!
//! The enum is exhaustively matched everywhere. Adding a variant breaks every
//! reader until each one says what it does with the new case — which is change
//! control performed by the compiler rather than by a reviewer remembering.

use vleo_units::Unit;

/// Why a node did not produce a number.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Fault {
    /// A supplied or derived value fell outside a declared limit. Carries the
    /// field, the offending value, the bound it broke, and the *reason the
    /// bound exists* — because a guard whose reason is not written gets deleted
    /// by the next person.
    OutOfDomain {
        node: &'static str,
        field: &'static str,
        value: f64,
        bound: f64,
        edge: Edge,
        unit: Unit,
        reason: &'static str,
    },
    /// The mathematics is undefined at this point — a zero denominator, a
    /// negative argument to a square root, a logarithm of zero. Distinct from
    /// `OutOfDomain`, which is a declared limit rather than a singularity.
    Degenerate {
        node: &'static str,
        field: &'static str,
        reason: &'static str,
    },
    /// The node has never run and something asked for its value.
    NotRun { node: &'static str },
    /// One or more upstream nodes have never run. The missing node is named,
    /// because a disabled control that does not say why is a defect.
    Blocked {
        node: &'static str,
        missing: &'static str,
    },
    /// A reference-data bundle a node declared it needs is not in the store.
    DataMissing {
        node: &'static str,
        bundle: &'static str,
    },
    /// A bundle is present but its hash or signature does not verify. This is a
    /// refusal, never a warning: a result computed from unverifiable data is
    /// not a degraded result, it is not a result.
    DataUnverified {
        node: &'static str,
        bundle: &'static str,
    },
    /// A declared cycle did not reach its convergence criterion. Carries the
    /// iteration count and the last residual, so non-convergence is a
    /// reportable result rather than a crash.
    NotConverged {
        node: &'static str,
        iterations: u32,
        residual: f64,
        tolerance: f64,
    },
    /// The dependency graph contains a cycle nobody declared. A cycle is a
    /// property of the design, so it is declared in the case; an undeclared one
    /// is an error naming the nodes involved.
    UndeclaredCycle {
        node: &'static str,
        back_to: &'static str,
    },
    /// A node depends on another that has been retired as *wrong*. Its
    /// consumers are suspect, and retirement should not protect anyone from
    /// finding that out.
    DependsOnWithdrawn {
        node: &'static str,
        withdrawn: &'static str,
        replacement: &'static str,
    },
    /// The resolver was given a workspace too small for the graph. A caller
    /// error, reported rather than silently truncated.
    WorkspaceTooSmall { needed: usize, given: usize },
}

/// Which end of a declared range was broken.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Edge {
    /// Below the declared lower limit.
    Lower,
    /// Above the declared upper limit.
    Upper,
}

impl Fault {
    /// The node the fault is attributed to. Every fault has one, so a failure
    /// always lands somewhere on the tree rather than as a generic message.
    pub fn node(&self) -> &'static str {
        match self {
            Fault::OutOfDomain { node, .. }
            | Fault::Degenerate { node, .. }
            | Fault::NotRun { node }
            | Fault::Blocked { node, .. }
            | Fault::DataMissing { node, .. }
            | Fault::DataUnverified { node, .. }
            | Fault::NotConverged { node, .. }
            | Fault::UndeclaredCycle { node, .. }
            | Fault::DependsOnWithdrawn { node, .. } => node,
            Fault::WorkspaceTooSmall { .. } => "resolver",
        }
    }

    /// A short machine-readable kind, used by the ledger and by the front end
    /// to choose how to present the refusal.
    pub fn kind(&self) -> &'static str {
        match self {
            Fault::OutOfDomain { .. } => "out-of-domain",
            Fault::Degenerate { .. } => "degenerate",
            Fault::NotRun { .. } => "not-run",
            Fault::Blocked { .. } => "blocked",
            Fault::DataMissing { .. } => "data-missing",
            Fault::DataUnverified { .. } => "data-unverified",
            Fault::NotConverged { .. } => "not-converged",
            Fault::UndeclaredCycle { .. } => "undeclared-cycle",
            Fault::DependsOnWithdrawn { .. } => "depends-on-withdrawn",
            Fault::WorkspaceTooSmall { .. } => "workspace-too-small",
        }
    }
}

impl core::fmt::Display for Fault {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Fault::OutOfDomain { node, field, value, bound, edge, unit, reason } => write!(
                f,
                "{node}: {field} = {value} {sym} is {which} the declared {end} limit of {bound} {sym} — {reason}",
                sym = unit.symbol(),
                which = match edge { Edge::Lower => "below", Edge::Upper => "above" },
                end = match edge { Edge::Lower => "lower", Edge::Upper => "upper" },
            ),
            Fault::Degenerate { node, field, reason } => {
                write!(f, "{node}: {field} is degenerate — {reason}")
            }
            Fault::NotRun { node } => write!(f, "{node}: has not run"),
            Fault::Blocked { node, missing } => {
                write!(f, "{node}: blocked — {missing} has never run")
            }
            Fault::DataMissing { node, bundle } => {
                write!(f, "{node}: bundle '{bundle}' is not in the local store — run `vleo data sync`")
            }
            Fault::DataUnverified { node, bundle } => {
                write!(f, "{node}: bundle '{bundle}' failed hash or signature verification — refusing to run")
            }
            Fault::NotConverged { node, iterations, residual, tolerance } => write!(
                f,
                "{node}: declared cycle did not converge in {iterations} iterations — residual {residual}, tolerance {tolerance}"
            ),
            Fault::UndeclaredCycle { node, back_to } => write!(
                f,
                "{node}: the derivation graph closes a loop back to {back_to} and no case declares it"
            ),
            Fault::DependsOnWithdrawn { node, withdrawn, replacement } => write!(
                f,
                "{node}: depends on {withdrawn}, which was retired as wrong — use {replacement}"
            ),
            Fault::WorkspaceTooSmall { needed, given } => {
                write!(f, "resolver: workspace holds {given} slots, the graph needs {needed}")
            }
        }
    }
}
