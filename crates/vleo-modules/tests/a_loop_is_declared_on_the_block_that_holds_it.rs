//! A loop belongs to the smallest block that holds it, and is declared there
//! (docs/SYSTEM_MODEL.md, section 5). The design's one loop — power becomes
//! heat, heat sets the array's temperature, and back — is declared in
//! `design/` on the block that holds it, `sys_satellite_subsystems`. Declared
//! on no block instead, in memory, it runs exactly as it does today, and the
//! gate notes the block it belongs on; declared on a block too small to hold
//! it, it is refused by name.

#![cfg(feature = "std")]

use std::path::PathBuf;

use vleo_bus::{Case, RunMode};
use vleo_files::convert::Served;
use vleo_modules::{opened, Scratch};
use vleo_sheet::files::Files;
use vleo_sheet::gate::{validate_tree, Verdict};
use vleo_sheet::load::{load_all_from, Tree};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The block the design declares its loop on, the smallest that holds it.
const HOLDS: &str = "sys_satellite_subsystems";

/// The design's files, served as the folders they were converted from, with
/// the loop taken out of the layer file of the block that holds it: the
/// served files, that layer file's path and its text without the loop, and
/// the loop, in the words it was declared in.
fn without_the_loop() -> (Served, PathBuf, String, String) {
    let r = root();
    let (served, _) = vleo_files::convert::serve(&r).unwrap();
    let file = r.join("layers").join(format!("{HOLDS}.toml"));
    let text = served.read_to_string(&file).unwrap();
    let at = text
        .find("\n[[group.iterate]]")
        .map(|i| i + 1)
        .expect("the design declares its loop on the block that holds it");
    let the_loop = text[at..].to_string();
    // The loop is the last of the block's tables: nothing else goes with it.
    assert!(
        the_loop
            .lines()
            .filter(|l| l.starts_with('['))
            .all(|l| l.starts_with("[[group.iterate")),
        "{the_loop}"
    );
    (served, file, text[..at].to_string(), the_loop)
}

/// The design with its loop declared on no block: a layer file of its own
/// that holds the loop and no heading, in the same words.
fn declared_on_no_block() -> Tree {
    let r = root();
    let (mut served, file, rest, the_loop) = without_the_loop();
    let the_loop = the_loop
        .replace("[[group.iterate]]", "[[iterate]]")
        .replace("[[group.iterate.seed]]", "[[iterate.seed]]");
    served.replace(&file, Some(rest.into_bytes()));
    served.replace(&r.join("layers/cycles.toml"), Some(the_loop.into_bytes()));
    load_all_from(&served, &r).unwrap()
}

/// The design with its loop taken off the block that holds it and declared on
/// group `on`, in the layer file that holds that group, in the same words.
fn declared_on(on: &str, layer: &str) -> Tree {
    let r = root();
    let (mut served, file, rest, the_loop) = without_the_loop();
    served.replace(&file, Some(rest.into_bytes()));
    let file = r.join("layers").join(layer);
    let text = served.read_to_string(&file).unwrap();
    let key = format!("id = \"{on}\"");
    let start = text.find(&key).expect("the group is in its layer file");
    // After the group's own keys: at the next table header, or the end.
    let end = text[start..]
        .find("\n[")
        .map_or(text.len(), |i| start + i + 1);
    let edited = format!("{}{}\n{}", &text[..end], the_loop, &text[end..]);
    served.replace(&file, Some(edited.into_bytes()));
    load_all_from(&served, &r).unwrap()
}

fn v11b(tree: &Tree) -> Verdict {
    validate_tree(tree)
        .into_iter()
        .find(|c| c.name.starts_with("V11b"))
        .expect("the gate checks where loops are declared")
        .verdict
}

#[test]
fn the_design_s_loop_is_declared_on_the_block_that_holds_it() {
    let tree = vleo_files::convert::open(&root()).unwrap().0;
    assert_eq!(tree.cycles.len(), 1);
    assert_eq!(tree.cycles[0].on, HOLDS);
    assert_eq!(v11b(&tree), Verdict::Pass);
}

#[test]
fn declared_on_no_block_the_loop_runs_as_it_does_today_and_is_noted() {
    let tree = declared_on_no_block();
    assert_eq!(tree.cycles.len(), 1);
    assert_eq!(tree.cycles[0].on, "");
    match v11b(&tree) {
        Verdict::Note(why) => assert!(
            why.contains(&format!(
                "declared on no block; the smallest that holds it is {HOLDS}"
            )),
            "{why}"
        ),
        other => panic!("{other:?}"),
    }
    // The engine holds the same loop — its rows, what it converges on, its
    // stopping rule and its seed — as the one declared on its block.
    let g = opened::graph(&tree).unwrap();
    let today = opened::graph(&vleo_files::convert::open(&root()).unwrap().0).unwrap();
    let (a, b) = (&g.cases[0].cycles, &today.cases[0].cycles);
    assert_eq!(a.len(), 1);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.nodes, y.nodes);
        assert_eq!(x.converge_on, y.converge_on);
        assert_eq!((x.tolerance, x.max_iter), (y.tolerance, y.max_iter));
        assert_eq!(x.seeds, y.seeds);
    }
    // And the whole design answers, and refuses, as it does today: the loop's
    // rows are stated and never derived yet, so they refuse, by name, alike.
    let case = Case {
        target: "pwr_demand".into(),
        mode: RunMode::All,
        ..Default::default()
    };
    let x = g.evaluate(&case, &mut Scratch::for_graph(g)).unwrap();
    let y = today
        .evaluate(&case, &mut Scratch::for_graph(today))
        .unwrap();
    let values = |r: &vleo_bus::Results| {
        r.values
            .iter()
            .map(|v| (v.id.clone(), v.value))
            .collect::<Vec<_>>()
    };
    let blocked = |r: &vleo_bus::Results| {
        r.blocked
            .iter()
            .map(|b| (b.id.clone(), b.message.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(values(&x), values(&y));
    assert_eq!(blocked(&x), blocked(&y));
    assert!(blocked(&x).iter().any(|(id, _)| id == "pwr_demand"));
}

#[test]
fn a_loop_declared_on_a_block_too_small_to_hold_it_is_refused_by_name() {
    let tree = declared_on("l3_power", "l3_power.toml");
    match v11b(&tree) {
        Verdict::Fail(why) => assert!(
            why.contains(&format!(
                "declared on l3_power, and the smallest block that holds it is {HOLDS}"
            )),
            "{why}"
        ),
        other => panic!("{other:?}"),
    }
}
