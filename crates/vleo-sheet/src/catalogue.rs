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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn tree() -> Tree {
        crate::load_all(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    }

    #[test]
    fn a_published_row_is_read_by_another_group_or_crosses_a_layer() {
        let t = tree();
        let cat = catalogue(&t);
        assert!(!cat.is_empty());
        for p in &cat {
            assert!(
                !p.crosses_to.is_empty() || p.read_by.iter().any(|(g, _)| g != &p.group),
                "{} is listed and nobody outside {} reads it",
                p.node,
                p.group
            );
        }
        // Every edge between two groups is in the catalogue, from its producer.
        let readers = readers(&t);
        for (from, rs) in &readers {
            let f = &t.sheets[*from];
            for r in rs {
                let r = &t.sheets[*r];
                if r.parent != f.parent && f.state != "deprecated" && r.state != "deprecated" {
                    let p = cat.iter().find(|p| p.node == f.id).unwrap_or_else(|| {
                        panic!(
                            "{} is read by {} of another group and is not listed",
                            f.id, r.id
                        )
                    });
                    assert!(p.read_by.contains(&(r.parent.clone(), r.id.clone())));
                }
            }
        }
    }

    #[test]
    fn the_impact_of_a_change_is_everything_downstream_in_other_groups() {
        let t = tree();
        let cat = catalogue(&t);
        let p = cat
            .iter()
            .find(|p| !p.read_by.is_empty())
            .expect("no row is read across groups");
        let reached = impact(&t, &[p.node.as_str()]);
        for (g, n) in &p.read_by {
            assert!(
                reached
                    .iter()
                    .any(|r| &r.node == n && &r.group == g && r.depth == 1),
                "{n} reads {} directly and is not reached at depth 1",
                p.node
            );
        }
        assert!(reached.iter().all(|r| r.group != p.group));
        // It follows readers of readers: some change in the tree reaches a
        // row two or more edges away, and every row it reaches at depth one
        // reads a changed row directly.
        let deepest = cat
            .iter()
            .map(|p| impact(&t, &[p.node.as_str()]))
            .max_by_key(|r| r.iter().map(|x| x.depth).max().unwrap_or(0))
            .unwrap();
        assert!(
            deepest.iter().any(|r| r.depth > 1),
            "no change reaches past its direct readers"
        );
        let rs = readers(&t);
        for r in reached.iter().filter(|r| r.depth == 1) {
            assert!(rs[p.node.as_str()].contains(&r.node.as_str()));
        }
        // A row that nobody reads reaches nothing.
        let leaf = t
            .ordered()
            .into_iter()
            .find(|s| !readers(&t).contains_key(s.id.as_str()))
            .unwrap();
        assert!(impact(&t, &[leaf.id.as_str()]).is_empty());
    }
}
