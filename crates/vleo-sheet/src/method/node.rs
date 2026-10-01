//! The method language: a node's method.

use super::*;

/// What a node's method is checked against: its inputs in declared order, and
/// its answer. `None` when a quantity is not one the tool has.
pub fn node_signature(sh: &crate::model::Sheet) -> Option<Signature> {
    let mut inputs = Vec::new();
    for i in &sh.inputs {
        inputs.push((i.binding.clone(), quantity_dim(&i.ty)?));
    }
    Some(Signature {
        inputs,
        output: quantity_dim(&sh.ty)?,
    })
}

/// A node's method, when it has one the tool can generate code from: written,
/// checking without error, on a row with one computed answer. Anything else
/// keeps its hand-written holes — and the gate says why.
pub fn node_program(sh: &crate::model::Sheet) -> Option<Program> {
    if sh.method.text.trim().is_empty() || sh.is_declared() || !sh.publishes.is_empty() {
        return None;
    }
    compile(&sh.method.text, &node_signature(sh)?).ok()
}

/// The node's kernel function, `vleo_core::physics::methods::<module>`, when
/// it has a method to translate.
pub fn node_rust(sh: &crate::model::Sheet) -> Option<String> {
    let p = node_program(sh)?;
    let inputs: Vec<String> = sh.inputs.iter().map(|i| i.binding.clone()).collect();
    Some(to_rust(&p, &sh.id, &sh.source, &sh.method.text, &inputs))
}
