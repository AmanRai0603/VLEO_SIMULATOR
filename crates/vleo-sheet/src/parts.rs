//! A node's six parts, each done, missing or failing.
//!
//! docs/SYSTEM_MODEL.md, section 3, says what a block holds: its identity, its
//! ports, its behaviour, its evidence, its explanation and its history. The
//! Node workspace shows them down its left side, each marked (docs/
//! OPERATING_1_0.md, section 4), so a node engineer sees at a glance what is
//! left to write and what is wrong with what is written.
//!
//! **Nothing here is a new rule.** A part fails when one of the gate's own
//! checks of it fails ([`part_of`] gives each check its part), and is missing
//! when the sheet holds nothing of it yet. A part the gate passes and the sheet
//! holds is done. So the left side of the workspace and the gate cannot
//! disagree: they are the same checks, read two ways.

use crate::gate::{self, Verdict};
use crate::model::Sheet;
use crate::Tree;

/// The six parts, in the order the system model gives them.
pub const PARTS: [&str; 6] = [
    "identity",
    "ports",
    "behaviour",
    "evidence",
    "explanation",
    "history",
];

/// How a part stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Done,
    Missing,
    Failing,
}

impl State {
    pub fn name(self) -> &'static str {
        match self {
            State::Done => "done",
            State::Missing => "missing",
            State::Failing => "failing",
        }
    }
}

/// One part of a node, and why it stands as it does.
#[derive(Clone, Debug)]
pub struct Part {
    pub name: &'static str,
    pub state: State,
    /// Each reason, a sentence: the gate's words for a failing check, or what
    /// the sheet does not hold yet.
    pub why: Vec<String>,
}

/// The part each of the gate's checks is about. Every check the gate makes of
/// a node has one, and a test holds the two lists to each other: a check
/// added to the gate with no part would fail nowhere on the workspace.
pub fn part_of(check: &str) -> Option<&'static str> {
    Some(match check {
        "schema" | "parent" | "seeded" | "criticality" | "gap-pass" => "identity",
        "inputs" | "port" | "contract" | "domain" | "sense" | "sense-applied" | "publishes" => {
            "ports"
        }
        "method" | "lookup" | "children" | "declared-value" | "no-identity" | "flight" => {
            "behaviour"
        }
        "cases" | "provenance" | "parity" => "evidence",
        "lesson" | "sources" => "explanation",
        "versions" => "history",
        _ => return None,
    })
}

/// What the sheet holds of `part`, or what it is still missing.
fn missing(sh: &Sheet, part: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut need = |empty: bool, what: &str| {
        if empty {
            out.push(what.to_string());
        }
    };
    match part {
        "identity" => {
            need(sh.question.trim().is_empty(), "the question it answers");
            need(sh.owner.trim().is_empty(), "who owns it");
        }
        "ports" => {
            need(sh.symbol.trim().is_empty(), "the output it answers with");
        }
        "behaviour" => match sh.behaviour() {
            "open" => need(
                true,
                "how it answers: a method, a stated value, a table or its children",
            ),
            "stated" => need(sh.value.is_none(), "the value it states"),
            "method" => need(sh.method.text.trim().is_empty(), "its method"),
            _ => {}
        },
        "evidence" => need(
            sh.cases.is_empty() && sh.fixtures.is_empty(),
            "a case: an input, an expected answer, a tolerance and where the answer came from",
        ),
        "explanation" => need(sh.explain.simply.trim().is_empty(), "the node said simply"),
        "history" => need(
            sh.versions.is_empty(),
            "a version saying which belief it rests on",
        ),
        _ => {}
    }
    out
}

/// The six parts of `sh`, as the gate and the sheet say they stand.
pub fn parts(sh: &Sheet, tree: &Tree) -> Vec<Part> {
    let checks = gate::gate_node(sh, tree);
    PARTS
        .iter()
        .map(|&name| {
            let failing: Vec<String> = checks
                .iter()
                .filter(|c| part_of(c.name) == Some(name))
                .filter_map(|c| match &c.verdict {
                    Verdict::Fail(why) => Some(format!("{}: {why}", c.name)),
                    _ => None,
                })
                .collect();
            if !failing.is_empty() {
                return Part {
                    name,
                    state: State::Failing,
                    why: failing,
                };
            }
            let gaps = missing(sh, name);
            Part {
                name,
                state: if gaps.is_empty() {
                    State::Done
                } else {
                    State::Missing
                },
                why: gaps,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every check the gate makes of a node, by the name it gives it: each
    /// `Check::pass`, `Check::fail` and `Check::note` in gate.rs whose name is
    /// not one of the whole tree's (those start with V and a number).
    fn node_checks() -> Vec<String> {
        let src = include_str!("gate.rs");
        let mut out = Vec::new();
        for opener in ["Check::pass(\"", "Check::fail(\"", "Check::note(\""] {
            for (at, _) in src.match_indices(opener) {
                let rest = &src[at + opener.len()..];
                let name = &rest[..rest.find('"').unwrap()];
                let tree_wide =
                    name.starts_with('V') && name[1..].starts_with(|c: char| c.is_ascii_digit());
                if !tree_wide && !out.iter().any(|n| n == name) {
                    out.push(name.to_string());
                }
            }
        }
        out
    }

    #[test]
    fn every_check_the_gate_makes_of_a_node_has_its_part() {
        let names = node_checks();
        assert!(
            names.len() > 20,
            "the gate's checks were not found: {names:?}"
        );
        let without: Vec<&String> = names.iter().filter(|n| part_of(n).is_none()).collect();
        assert!(
            without.is_empty(),
            "checks the gate makes of a node with no part on the workspace: {without:?}"
        );
        for n in &names {
            assert!(PARTS.contains(&part_of(n).unwrap()), "{n}");
        }
    }
}
