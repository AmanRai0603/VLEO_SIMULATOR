//! The method language: a node's method.

use super::*;

/// What a node's method is checked against: its inputs in declared order, and
/// its answer. `None` when a quantity is not one the tool has.
pub fn node_signature(sh: &crate::model::Sheet) -> Option<Signature> {
    let mut inputs = Vec::new();
    for i in &sh.inputs {
        inputs.push((i.binding.clone(), quantity_dim(&i.ty)?));
    }
    let mut publishes = Vec::new();
    for p in &sh.publishes {
        publishes.push((p.symbol.clone(), quantity_dim(&p.ty)?));
    }
    Some(Signature {
        inputs,
        output: quantity_dim(&sh.ty)?,
        publishes,
    })
}

/// A node's method, when it has one the engine can run: written, checking
/// without error, on a row that computes its answer. Anything else answers
/// nothing until it has one — and the gate says why.
pub fn node_program(sh: &crate::model::Sheet) -> Option<Program> {
    if sh.method.text.trim().is_empty() || sh.is_declared() {
        return None;
    }
    compile(&sh.method.text, &node_signature(sh)?).ok()
}
