//! The gap pass.
//!
//! What each sheet promised and nothing yet covers, per node and for the
//! tree. It never reads another node, and its output is byte-stable: sorted
//! iteration everywhere, so two runs on the same design say the same thing.

use crate::load::Tree;
use crate::model::*;

// ---------------------------------------------------------------------------
// the gap pass
// ---------------------------------------------------------------------------

/// What the sheet promised and nothing yet covers.
///
/// The gate asks whether what exists passes. It does not ask whether anything
/// the sheet promised is *absent*, and the dominant way this class of work fails
/// is not that it cannot make progress — it is that something plausible gets
/// built, the checks it chose for itself pass, a confident summary is written,
/// and it stops while real requirements are still unmet.
///
/// Here the requirement is a schema rather than prose, so the difference
/// between the sheet and the artefacts is a diff and not a judgement. It costs
/// nothing and it runs on every node, every night. Its output is a list, not a
/// verdict: a node with an open gap cannot enter the second human review.
pub fn gap_pass(sh: &Sheet) -> Vec<String> {
    let mut g = Vec::new();
    if sh.is_seeded() {
        g.push(
            "seeded, not yet specified — it needs a question, a relation cited to a page, \
             an interface with units, a domain with a reason per bound, a numbered algorithm \
             and a known-good value from outside this code"
                .into(),
        );
        return g;
    }
    // The explanation standard and the de-risking record, docs/EXPLAINING.md
    // and docs/DERISKING.md. Both are open gaps rather than failures: a row
    // that computes correctly and cannot yet be explained plainly, or has not
    // said what it rests on, is unfinished rather than broken.
    if sh.explain.simply.trim().is_empty() || sh.explain.breaks.trim().is_empty() {
        g.push(
            "not said simply, or not where the simple version breaks — a reader who cannot \
             follow the relation has nothing to hold on to (E2)"
                .into(),
        );
    }
    if sh.versions.is_empty() {
        g.push(
            "no belief recorded — what does this row rest on, and what would break it? Its \
             next change will need to say which belief broke (D1)"
                .into(),
        );
    }
    if sh.question.trim().is_empty() {
        g.push(
            "no question stated — an equation with no question gets reused for the wrong thing"
                .into(),
        );
    }
    if sh.expression.trim().is_empty() {
        g.push("no relation stated".into());
    }
    if sh.source.trim().is_empty() {
        g.push("no source cited — this is the claim everything else rests on".into());
    }
    if sh.owner.trim().is_empty() {
        g.push("no owner — reviews on this node route nowhere".into());
    }
    if !sh.expression.trim().is_empty() && sh.relation_by.trim().is_empty() {
        g.push(
            "the relation has nobody's name against it — an assistant may never supply \
             mathematics, and without an attribution nothing can tell whether one did"
                .into(),
        );
    }
    // The method and its author's cases: the node's relation in a form the
    // tool can check and run, and the evidence from the author's own
    // code. A computed row with one answer and none of this is unfinished, and
    // says so; a set row and a declared row are not asked.
    if !sh.is_declared() && sh.publishes.is_empty() && sh.method.text.trim().is_empty() {
        g.push(
            "no method yet — the relation in the method language, with the author's own code \
             and test cases, so the tool can check it three ways (docs/PSEUDOCODE.md)"
                .into(),
        );
    }
    if sh.criticality == "significant"
        && !sh.dir.join("parity.csv").is_file()
        && sh.fixtures.len() < 2
    {
        g.push(
            "significant, and one fixture or none — a significant node is the one that gets a \
             second independent check, which is the whole reason for the word"
                .into(),
        );
    }
    if sh.migrated_from.trim().is_empty() && sh.dir.join("parity.csv").is_file() {
        g.push(
            "a parity.csv with no migrated_from — the grid is some other implementation's \
             numbers and nothing says whose, which makes a disagreement unattributable"
                .into(),
        );
    }
    if !sh.migrated_from.trim().is_empty() && !sh.dir.join("parity.csv").is_file() {
        g.push(format!(
            "migrated from {} and no parity.csv — the prior implementation is a liability until \
             its numbers sit beside this one",
            sh.migrated_from
        ));
    }
    if sh.symbol.trim().is_empty() || sh.ty.trim().is_empty() || sh.unit.trim().is_empty() {
        g.push("the output is not fully declared: symbol, type and unit are all required".into());
    }
    if sh.reason_lower.trim().is_empty() || sh.reason_upper.trim().is_empty() {
        g.push("a declared limit has no reason — a guard whose reason is not written gets deleted by the next person".into());
    }
    if !sh.is_declared() && sh.inputs.is_empty() {
        g.push(
            "a computed node with no declared input claims to compute something from nothing"
                .into(),
        );
    }
    if sh.is_declared() && sh.value.is_none() {
        g.push("a declared value with no number".into());
    }
    if sh.is_declared() && sh.confirmed_by.trim().is_empty() {
        g.push("a declared value with nobody's confirmation against it".into());
    }
    // A row's steps are computed by its method, and by nothing else.
    let from_method = crate::method::node_program(sh).is_some();
    for st in sh.steps.iter().filter(|_| !from_method) {
        g.push(format!("step {} is in no method: {}", st.number, st.text));
        if st.binds.trim().is_empty() || st.ty.trim().is_empty() {
            g.push(format!(
                "step {} does not say what it binds or at what type",
                st.number
            ));
        }
    }
    if !sh.is_declared() && sh.fixtures.is_empty() {
        g.push(
            "no fixture: nothing outside this code has ever agreed with what it computes".into(),
        );
    }
    for fx in &sh.fixtures {
        if fx.provenance == "self-snapshot" || fx.provenance == "agent-generated" {
            g.push(format!(
                "fixture '{}' has provenance '{}' — an expected value may not come from the code under test",
                fx.label, fx.provenance
            ));
        }
        if fx.source.trim().is_empty() {
            g.push(format!("fixture '{}' cites no source", fx.label));
        }
        for i in &sh.inputs {
            if !fx.inputs.iter().any(|(k, _)| *k == i.binding) {
                g.push(format!(
                    "fixture '{}' does not supply the input '{}'",
                    fx.label, i.binding
                ));
            }
        }
    }
    if sh.assumptions.is_empty() && !sh.is_declared() && sh.steps.len() > 1 {
        g.push("no assumption stated — a multi-step relation always has at least one".into());
    }
    // A relation nobody explained. On a function this is not only a review
    // problem any more: the resolver refuses the row, so the gap and the
    // silence are the same fact and the message says which one it is. On a
    // declared value there is no algorithm to derive and the row still answers,
    // so it stays what it was — a gap against the second review, where somebody
    // has to agree the relation is the right one and cannot do it from a single
    // line of algebra.
    if sh.theory.is_empty() && !sh.expression.trim().is_empty() {
        g.push(if sh.steps.is_empty() {
            "no theory: the relation is stated but not derived, so a reviewer can check what it \
             computes and not whether it is the right thing to compute"
                .into()
        } else {
            "no theory: the relation is stated but not derived — THE ROW DOES NOT ANSWER. A \
             function is defined by its derivation, not by its expression and not by its \
             citation, and until one is written every reader of this row is blocked on it \
             by name"
                .to_string()
        });
    }
    g
}

/// The whole tree's gaps, for the nightly pass.
pub fn gap_report(tree: &Tree) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for sh in tree.ordered() {
        let g = gap_pass(sh);
        if !g.is_empty() {
            out.push((sh.id.clone(), g));
        }
    }
    out
}
