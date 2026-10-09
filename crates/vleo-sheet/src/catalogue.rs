//! What each group publishes to the others, and what a change to it reaches.
//!
//! A group owns the rows under its heading and releases them together. What
//! crosses to another group is the contract between them: a row of one group
//! that a row of another reads, or the interface row that crosses a layer. The
//! catalogue lists exactly those, with who reads each, so a group knows what it
//! has promised and the developer taking its release in knows whom it touches.
//!
//! The impact of a change is everything downstream of the rows it changes, in
//! other groups: a reader, its readers, and so on. Taken from the derivation
//! graph — the inputs each sheet declares — never from names, because a name
//! can be renamed and an input edge cannot.

use crate::load::Tree;
use crate::text::producer_of;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// One row a group publishes: read by another group, or crossing a layer.
#[derive(Clone, Debug, PartialEq)]
pub struct Published {
    pub group: String,
    pub node: String,
    pub label: String,
    pub ty: String,
    pub unit: String,
    pub kind: String,
    pub state: String,
    /// The newest recorded version, or 0 when the row has none.
    pub version: u32,
    /// The layer it crosses to, for an interface row; empty otherwise.
    pub crosses_to: String,
    /// Every row of another group that reads it, as `(group, node)`.
    pub read_by: Vec<(String, String)>,
}

/// Every row's readers, from the inputs each sheet declares.
pub fn readers(tree: &Tree) -> BTreeMap<&str, Vec<&str>> {
    let mut out: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for sh in tree.ordered() {
        for inp in &sh.inputs {
            if let Some((k, _)) = tree.sheets.get_key_value(producer_of(&inp.var)) {
                if k.as_str() != sh.id.as_str() {
                    let v = out.entry(k.as_str()).or_default();
                    if !v.contains(&sh.id.as_str()) {
                        v.push(sh.id.as_str());
                    }
                }
            }
        }
    }
    out
}

/// What every group publishes, by group then row. A deprecated row publishes
/// nothing; a reader that is deprecated reads nothing.
pub fn catalogue(tree: &Tree) -> Vec<Published> {
    let readers = readers(tree);
    let live = |id: &str| tree.sheets.get(id).is_some_and(|s| s.state != "deprecated");
    let mut out = Vec::new();
    for sh in tree.ordered() {
        if sh.state == "deprecated" {
            continue;
        }
        let mut read_by: Vec<(String, String)> = readers
            .get(sh.id.as_str())
            .into_iter()
            .flatten()
            .filter(|r| live(r))
            .filter_map(|r| tree.sheets.get(*r))
            .filter(|r| r.parent != sh.parent)
            .map(|r| (r.parent.clone(), r.id.clone()))
            .collect();
        if read_by.is_empty() && sh.crosses_to.is_empty() {
            continue;
        }
        read_by.sort();
        out.push(Published {
            group: sh.parent.clone(),
            node: sh.id.clone(),
            label: sh.label.clone(),
            ty: sh.ty.clone(),
            unit: sh.unit.clone(),
            kind: sh.kind.clone(),
            state: sh.state.clone(),
            version: sh.versions.iter().map(|v| v.n).max().unwrap_or(0),
            crosses_to: sh.crosses_to.clone(),
            read_by,
        });
    }
    out.sort_by(|a, b| (&a.group, &a.node).cmp(&(&b.group, &b.node)));
    out
}

/// One row a change reaches: its group, and how many edges away from the
/// nearest changed row it is.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reached {
    pub group: String,
    pub node: String,
    pub depth: usize,
}

/// Everything downstream of `changed`, outside the groups the changed rows
/// belong to, nearest first. A row is reached once, at its smallest depth.
/// Rows not in the tree are ignored: the caller names what it changed.
pub fn impact(tree: &Tree, changed: &[&str]) -> Vec<Reached> {
    let readers = readers(tree);
    let own: BTreeSet<&str> = changed
        .iter()
        .filter_map(|id| tree.sheets.get(*id))
        .map(|s| s.parent.as_str())
        .collect();
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    let mut queue: VecDeque<(&str, usize)> = changed
        .iter()
        .filter(|id| tree.sheets.contains_key(**id))
        .map(|id| (*id, 0))
        .collect();
    for (id, _) in &queue {
        seen.insert(id, 0);
    }
    while let Some((id, d)) = queue.pop_front() {
        for r in readers.get(id).into_iter().flatten() {
            if !seen.contains_key(r) {
                seen.insert(r, d + 1);
                queue.push_back((r, d + 1));
            }
        }
    }
    let mut out: Vec<Reached> = seen
        .into_iter()
        .filter(|(_, d)| *d > 0)
        .filter_map(|(id, depth)| {
            let sh = tree.sheets.get(id)?;
            (!own.contains(sh.parent.as_str()) && sh.state != "deprecated").then(|| Reached {
                group: sh.parent.clone(),
                node: sh.id.clone(),
                depth,
            })
        })
        .collect();
    out.sort_by(|a, b| (a.depth, &a.group, &a.node).cmp(&(b.depth, &b.group, &b.node)));
    out
}
