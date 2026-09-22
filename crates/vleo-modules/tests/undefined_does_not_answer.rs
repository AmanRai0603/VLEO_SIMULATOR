//! A function nobody derived does not answer.
//!
//! Checked over the whole assembled tree rather than over a hand-built table,
//! so it keeps being true as sheets are written: the day somebody derives
//! `orbit_mission_duration`, the row moves from one side of these assertions to
//! the other and nothing here needs editing.
//!
//! The rule these protect, in one line: **an input may carry a default, a
//! function may not.** A default is a decision somebody made and the face lets
//! it be edited. A relation stated and never derived is a number the design has
//! never accounted for, and the resolver refuses it rather than printing it
//! beside the ones it has.

use vleo_bus::{Case, RunMode};
use vleo_modules::{Scratch, NODES};

fn run(node: &str, mode: RunMode) -> Result<vleo_bus::Results, vleo_core::fault::Fault> {
    let case = Case {
        base: "c1".into(),
        supply: Vec::new(),
        target: node.into(),
        mode,
        data: Vec::new(),
        data_versions: Vec::new(),
    };
    let mut scratch = Scratch::new();
    vleo_modules::evaluate(&case, &mut scratch)
}

/// Asked on its own, with nothing else in the order. The strongest form of the
/// claim — not "it was blocked because its inputs were missing" but "it was
/// asked directly and refused".
fn alone(node: &str) -> vleo_bus::Results {
    run(node, RunMode::Alone).expect("a run of one node cannot fail before it starts")
}

/// The tree must actually contain both sides of the distinction, or every
/// assertion below is vacuously true and this file is decoration.
#[test]
fn the_tree_has_functions_on_both_sides_of_the_rule() {
    let runnable = || NODES.iter().filter(|d| d.state.runnable());
    let derived = runnable().filter(|d| d.is_function() && d.derived).count();
    let undefined = runnable().filter(|d| d.is_function() && !d.derived).count();
    let inputs = runnable().filter(|d| !d.is_function()).count();
    assert!(
        derived > 0 && undefined > 0 && inputs > 0,
        "the rule is only tested if the tree has all three: {derived} derived function(s), \
         {undefined} undefined, {inputs} input(s)"
    );
}

/// An undefined function refuses, under its own name, with the reason.
///
/// Every one of them, not a sample: a rule that holds for the row somebody
/// happened to write a test for is not a rule.
#[test]
fn every_undefined_function_refuses_by_name() {
    for def in NODES.iter().filter(|d| d.state.runnable()) {
        if !def.is_function() || def.derived {
            continue;
        }
        let r = alone(def.id);
        assert!(
            !r.values.iter().any(|v| v.id == def.id),
            "{} has no derivation and answered anyway with {:?}",
            def.id,
            r.values.iter().find(|v| v.id == def.id).map(|v| v.value)
        );
        let b = r
            .blocked
            .iter()
            .find(|b| b.id == def.id)
            .unwrap_or_else(|| panic!("{} did not answer and is not in the blocked list — a refusal that names nobody is a silent one", def.id));
        assert_eq!(
            b.kind, "undefined",
            "{} refused as '{}' rather than as its own reason",
            def.id, b.kind
        );
    }
}

/// An input answers from its default. This is the half of the rule that is easy
/// to break while fixing the other half — refuse too widely and the tree has no
/// values at all, which passes the test above perfectly.
#[test]
fn an_input_answers_from_its_default() {
    let mut checked = 0usize;
    for def in NODES.iter().filter(|d| d.state.runnable()) {
        if def.is_function() || !def.inputs.is_empty() {
            continue;
        }
        let r = alone(def.id);
        assert!(
            r.values.iter().any(|v| v.id == def.id),
            "{} is an input carrying a default and did not answer: {:?}",
            def.id,
            r.blocked
                .iter()
                .find(|b| b.id == def.id)
                .map(|b| &b.message)
        );
        checked += 1;
    }
    assert!(checked > 100, "only {checked} inputs were reached");
}

/// A consumer of an undefined row blocks, and names a row rather than the
/// general situation. "Not active" without the name is a dead end for whoever
/// has to fix it.
#[test]
fn a_consumer_blocks_on_a_named_row() {
    // The largest consumer of undefined work in the tree, found rather than
    // written down — whichever row it is, its refusal must name somebody.
    // A branch run, because this is about the closure. A declared cycle the
    // case does not carry is a different refusal and not this test's business,
    // so those are skipped rather than asserted about.
    let target = NODES
        .iter()
        .filter(|d| d.state.runnable() && d.is_function() && d.derived)
        .filter_map(|d| run(d.id, RunMode::Branch).ok().map(|r| (d.id, r)))
        .find(|(id, r)| r.blocked.iter().any(|b| b.id != *id));
    let Some((id, r)) = target else {
        return; // every derived function's closure is clean; nothing to check
    };
    let own = r
        .blocked
        .iter()
        .find(|b| b.id == id)
        .expect("the target did not answer, so it is blocked");
    assert_eq!(
        own.kind, "blocked",
        "{id} blamed itself for an upstream gap"
    );
    let named = own.message.split_whitespace().any(|w| {
        NODES
            .iter()
            .any(|d| d.id == w.trim_matches(|c: char| !c.is_alphanumeric() && c != '_'))
    });
    assert!(named, "{id} blocked without naming a row: {}", own.message);
}

/// A derived relation nobody has read against its source answers, and says so.
///
/// The two facts are separate on purpose and the separation is the point. The
/// derivation is the sheet author's own account of where the relation came
/// from, and without it there is nothing to review — so the row is silent.
/// `[maths] confirmed_by` is a second person having read that account against
/// the source, and its absence is a number worth less, not a number withheld.
///
/// Before this, the mathematics factor scored 4 for any row with an expression
/// and a citation. A citation says a paper exists. It cannot say the relation
/// came out of it, and an invented relation carries one just as convincingly.
///
/// Scored straight off the definitions rather than off a run: the confirmed
/// relations in this tree all read a reference-data bundle, and whether a
/// bundle is on the disk is a question about the machine, not about this rule.
#[test]
fn mathematics_is_scored_on_the_attribution_not_the_citation() {
    use vleo_core::credibility::{score, CredVec, Factor};
    // Everything except the factor under test is held at its best, upstream
    // included. The chain is folded in factor by factor at the end of `score`,
    // so an upstream of nothing-assessed would drive every factor to zero and
    // this test would agree with any implementation at all.
    let mut best = CredVec::ZERO;
    for f in Factor::ALL {
        best.set(f, 4);
    }
    let m = |def| score(def, true, true, true, best).get(Factor::Mathematics);
    let (mut unconfirmed, mut confirmed) = (0usize, 0usize);
    for def in NODES
        .iter()
        .filter(|d| d.state.runnable() && d.is_function() && d.derived)
    {
        if def.relation_by.is_empty() {
            assert_eq!(
                m(def),
                1,
                "{} is derived and unconfirmed and scored {} for mathematics",
                def.id,
                m(def)
            );
            unconfirmed += 1;
        } else {
            assert_eq!(
                m(def),
                4,
                "{} has a name against its relation and scored {} for mathematics",
                def.id,
                m(def)
            );
            confirmed += 1;
        }
    }
    assert!(
        unconfirmed > 0 && confirmed > 0,
        "only tested if both exist: {unconfirmed} unconfirmed, {confirmed} confirmed"
    );
}

/// And the unconfirmed ones still answer. The whole design is that the two
/// levels do different things — silence for no derivation, a lower number for
/// no attribution — so a change that collapsed them would have to fail here.
#[test]
fn an_unconfirmed_relation_is_not_withheld() {
    let answered = NODES
        .iter()
        .filter(|d| d.state.runnable() && d.is_function() && d.derived && d.relation_by.is_empty())
        .filter(|d| {
            run(d.id, RunMode::Branch)
                .map(|r| r.values.iter().any(|v| v.id == d.id))
                .unwrap_or(false)
        })
        .count();
    assert!(
        answered > 0,
        "no derived-but-unconfirmed row answered, so 'unconfirmed is not withheld' is untested"
    );
}
