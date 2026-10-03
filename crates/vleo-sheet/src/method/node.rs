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

/// A node's method, when it has one the tool can generate code from: written,
/// checking without error, on a row with one computed answer. Anything else
/// keeps its hand-written holes — and the gate says why.
pub fn node_program(sh: &crate::model::Sheet) -> Option<Program> {
    if sh.method.text.trim().is_empty() || sh.is_declared() {
        return None;
    }
    compile(&sh.method.text, &node_signature(sh)?).ok()
}

/// The node's kernel function, `vleo_core::physics::methods::<module>`, when
/// it has a method to translate.
pub fn node_rust(sh: &crate::model::Sheet) -> Option<String> {
    let p = node_program(sh)?;
    let inputs: Vec<String> = sh.inputs.iter().map(|i| i.binding.clone()).collect();
    let publishes: Vec<String> = sh.publishes.iter().map(|p| p.symbol.clone()).collect();
    Some(to_rust_publishing(
        &p,
        &sh.id,
        &sh.source,
        &sh.method.text,
        &inputs,
        &publishes,
    ))
}
