//! What each group publishes to the others, and what a change reaches
//! (`vleo_sheet::catalogue`), on the design as it is: `design/`, read by the
//! one reader. Kept beside the reader because the sheets the catalogue was
//! once tested on are not the design.

use std::path::Path;

use vleo_sheet::catalogue::{catalogue, impact, readers};
use vleo_sheet::load::Tree;

fn tree() -> Tree {
    vleo_files::convert::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .expect("the design loads")
        .0
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
