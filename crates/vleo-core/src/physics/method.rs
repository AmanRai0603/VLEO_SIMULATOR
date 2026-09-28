//! Where a node's method meets the fault taxonomy.
//!
//! A node's method is translated by rule into `methods::<node>`, a function of
//! SI numbers that answers or says why it will not — `vleo_units::method_rt`
//! holds the operations it is built from, shared with the interpreter that
//! checked it. This turns its "why not" into the `Fault` every node returns.

use crate::fault::Fault;
pub use vleo_units::method_rt::MethodError;

/// The fault a method's refusal or degenerate value becomes, under the node's
/// own name.
pub fn fault(e: MethodError, node: &'static str, field: &'static str) -> Fault {
    match e {
        MethodError::Refused(reason) => Fault::Refused { node, reason },
        MethodError::Degenerate { what, .. } => Fault::Degenerate {
            node,
            field,
            reason: what,
        },
    }
}
