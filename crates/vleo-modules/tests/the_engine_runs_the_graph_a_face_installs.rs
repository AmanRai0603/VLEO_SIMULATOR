//! The engine runs the graph a face installs from the design's files, and no
//! other: every function a face calls answers from it, and names what it
//! answers from it too, so a design laid out otherwise than this build is
//! answered under its own names.
//!
//! One test, run alone in its own binary, because which graph runs is the one
//! setting the engine shares across a process.

#![cfg(feature = "std")]

use std::path::PathBuf;

use vleo_bus::{Case, RunMode};
use vleo_modules::core_engine::graph::Kind;
use vleo_modules::{engine, opened, run_compiled, run_on, Scratch, COMPILED};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn alone(id: &str) -> Case {
    Case {
        target: id.to_string(),
        mode: RunMode::All,
        ..Default::default()
    }
}

#[test]
fn the_engine_runs_the_installed_graph_and_names_what_it_answers_from_it() {
    assert!(
        std::ptr::eq(engine(), &COMPILED),
        "before a face installs one"
    );

    // A row that answers, held as code by its id, in a design whose copy of it
    // has moved on: the same layout, one relation this build does not have.
    // Every relation of the design is a method now, and a method the build
    // has not seen runs in the interpreter; so the row is taken without it.
    let mut tree = vleo_sheet::load_all(&root()).unwrap();
    let id = vleo_modules::nodes()
        .iter()
        .find(|d| {
            d.kind == Kind::Computed
                && vleo_modules::evaluate(&alone(d.id), &mut Scratch::new())
                    .is_ok_and(|r| r.values.iter().any(|v| v.id == d.id))
        })
        .map(|d| d.id.to_string())
        .expect("a computed row that answers");
    let sh = tree.sheets.get_mut(&id).unwrap();
    sh.method = Default::default();
    sh.impl_hash ^= 1;
    let moved = opened::graph(&tree).unwrap();

    // Installed, every free function runs on it: the row is refused by name.
    run_on(moved);
    assert!(std::ptr::eq(engine(), moved));
    let r = vleo_modules::evaluate(&alone(&id), &mut Scratch::new()).unwrap();
    assert!(
        r.blocked.iter().any(|b| b.id == id
            && b.message
                .contains("its relation is not in this build of the engine")),
        "{id} ran on the compiled graph after a face installed another: {:#?}",
        r.values
    );
    assert!(!r.values.iter().any(|v| v.id == id));

    // Compiled again, it answers again.
    run_compiled();
    assert!(std::ptr::eq(engine(), &COMPILED));
    let r = vleo_modules::evaluate(&alone(&id), &mut Scratch::new()).unwrap();
    assert!(r.values.iter().any(|v| v.id == id));

    // A design with a row fewer is laid out otherwise than this build: it
    // runs, and every list a face reads is the design's, not the build's.
    // The last row nothing reads: a row another reads cannot go, for the
    // design would name a variable it no longer has and is refused.
    let mut short = vleo_sheet::load_all(&root()).unwrap();
    let last = unread(&short).last().unwrap().clone();
    short.sheets.remove(&last);
    let fewer = opened::graph(&short).unwrap();
    run_on(fewer);
    assert!(std::ptr::eq(engine(), fewer));
    assert_eq!(vleo_modules::nodes().len(), COMPILED.nodes.len() - 1);
    assert_eq!(vleo_modules::vars().len(), fewer.vars.len());
    assert!(
        vleo_modules::Vleo::find(&last).is_none(),
        "{last} is still named"
    );
    assert_eq!(vleo_modules::groups().len(), COMPILED.groups.len());
    assert_eq!(vleo_modules::cases().len(), COMPILED.cases.len());
    run_compiled();
    assert!(std::ptr::eq(engine(), &COMPILED));

    // The case's inputs, read from a graph whose rows have all moved: the
    // first row gone, every other one place earlier than this build has it.
    // Each input still says its own row's declared value, not its neighbour's.
    let defaults = || -> std::collections::BTreeMap<&'static str, u64> {
        vleo_modules::inputs::inputs(&vleo_modules::cases()[0])
            .iter()
            .map(|i| (i.id, i.default.to_bits()))
            .collect()
    };
    let built = defaults();
    let mut shifted = vleo_sheet::load_all(&root()).unwrap();
    let first = unread(&shifted).first().unwrap().clone();
    shifted.sheets.remove(&first);
    run_on(opened::graph(&shifted).unwrap());
    let moved = defaults();
    run_compiled();
    assert!(moved.len() > 100, "only {} inputs", moved.len());
    for (id, v) in &moved {
        assert_eq!(
            Some(v),
            built.get(id),
            "{id}'s declared value is not its own"
        );
    }
}

/// The rows nothing in the design names, in the design's order: no row reads
/// them and no case supplies, conditions on or loops through them.
fn unread(tree: &vleo_sheet::load::Tree) -> Vec<String> {
    let mut named: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for sh in tree.sheets.values() {
        for i in &sh.inputs {
            named.insert(i.var.split('.').next().unwrap_or(&i.var));
        }
    }
    for c in tree.cases.values() {
        named.extend(c.supply.iter().map(|(k, _)| k.as_str()));
        named.extend(c.conditions.iter().map(String::as_str));
        for cy in &c.cycles {
            named.extend(cy.nodes.iter().map(String::as_str));
            named.insert(cy.converge_on.as_str());
            named.extend(cy.seeds.iter().map(|(k, _)| k.as_str()));
        }
    }
    tree.ordered()
        .into_iter()
        .filter(|sh| !named.contains(sh.id.as_str()))
        .map(|sh| sh.id.clone())
        .collect()
}

#[test]
fn the_graph_read_from_the_design_carries_its_headings_and_their_relations() {
    // A face draws the tree from the graph that runs, so the graph read from
    // the design's files carries every heading and relation the build does,
    // in the build's order.
    let read = opened::graph(&vleo_sheet::load_all(&root()).unwrap()).unwrap();
    let heading = |g: &vleo_modules::GroupDef| {
        (
            g.id, g.label, g.parent, g.owner, g.layer, g.order, g.is_box, g.tone, g.cases,
        )
    };
    assert_eq!(read.groups.len(), COMPILED.groups.len());
    for (a, b) in read.groups.iter().zip(COMPILED.groups) {
        assert_eq!(heading(a), heading(b));
    }
    assert_eq!(read.relations, COMPILED.relations);
}
