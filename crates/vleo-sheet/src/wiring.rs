//! Every name the design wires by, and whether each resolves.
//!
//! THE TABLES ONCE GUESSED. An input naming a variable that does not exist was
//! wired to variable 0; an unknown cycle member, seed, supply or condition was
//! left out; a fixture that did not bind an input fed it 0.0. Each compiled,
//! and the engine it built answered a different question than the sheets ask.
//! The gate caught most of them, but a plain `cargo build` did not.
//!
//! So this is asked before any graph is made of the design: by the build,
//! before it compiles the tables (`vleo-modules/build.rs`), and by the reader,
//! before it builds the graph a face runs from the design's files
//! (`vleo_modules::opened`). A name that resolves to nothing stops both, by
//! name; nothing runs on a guess.

use crate::load::Tree;

/// Every name the design would have to resolve and cannot, one sentence each.
pub fn errors(tree: &Tree) -> Vec<String> {
    let mut vars: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for sh in tree.sheets.values() {
        vars.insert(sh.id.clone());
        for pb in &sh.publishes {
            vars.insert(format!("{}.{}", sh.id, pb.id));
        }
    }
    let known = |v: &str| vars.contains(v);
    let mut e = Vec::new();
    for sh in tree.ordered() {
        for i in &sh.inputs {
            if !known(&i.var) {
                e.push(format!(
                    "{}: input `{}` reads `{}`, which no row publishes",
                    sh.id, i.binding, i.var
                ));
            }
        }
        for f in &sh.fixtures {
            for i in &sh.inputs {
                if !f.inputs.iter().any(|(k, _)| *k == i.binding) {
                    e.push(format!(
                        "{}: fixture '{}' gives no value for input `{}`",
                        sh.id, f.label, i.binding
                    ));
                }
            }
            // A fixture names the variable it checks by a published id, or the
            // row's own answer by its symbol or id; anything else is a typo.
            let own = f.variable == sh.symbol || f.variable == sh.id;
            if !f.variable.is_empty() && !own && !sh.publishes.iter().any(|pb| pb.id == f.variable)
            {
                e.push(format!(
                    "{}: fixture '{}' checks `{}`, which this row does not publish",
                    sh.id, f.label, f.variable
                ));
            }
        }
    }
    for c in tree.cases.values() {
        for (k, _) in &c.supply {
            if !known(k) {
                e.push(format!("case {}: supplies `{k}`, which is not a row", c.id));
            }
        }
        for k in &c.conditions {
            if !known(k) {
                e.push(format!("case {}: condition `{k}` is not a row", c.id));
            }
        }
        for cy in &c.cycles {
            for n in cy.nodes.iter().chain([&cy.converge_on]) {
                if !known(n) {
                    e.push(format!(
                        "case {}: cycle names `{n}`, which is not a row",
                        c.id
                    ));
                }
            }
            for (k, _) in &cy.seeds {
                if !known(k) {
                    e.push(format!(
                        "case {}: cycle seeds `{k}`, which is not a row",
                        c.id
                    ));
                }
            }
        }
    }
    e
}
