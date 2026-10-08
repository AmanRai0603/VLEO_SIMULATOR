//! A loop belongs to the smallest block that holds it, and is declared there
//! (docs/SYSTEM_MODEL.md, section 5). The design's one loop — power becomes
//! heat, heat sets the array's temperature, and back — is declared today in
//! `layers/cycles.toml`, on no block; the gate notes the block it belongs on.
//! Declared on that block instead, in memory, it runs exactly as it does
//! today; declared on a block too small to hold it, it is refused by name.

#![cfg(feature = "std")]

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use vleo_bus::{Case, RunMode};
use vleo_modules::{opened, Scratch};
use vleo_sheet::files::{Disk, Files};
use vleo_sheet::gate::{validate_tree, Verdict};
use vleo_sheet::load::{load_all_from, Tree};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The design's files with some replaced, as text.
struct Edited(BTreeMap<PathBuf, String>);

impl Files for Edited {
    fn read(&self, p: &Path) -> io::Result<Vec<u8>> {
        match self.0.get(p) {
            Some(t) => Ok(t.clone().into_bytes()),
            None => Disk.read(p),
        }
    }
    fn entries(&self, p: &Path) -> io::Result<Vec<PathBuf>> {
        Disk.entries(p)
    }
    fn is_dir(&self, p: &Path) -> bool {
        Disk.is_dir(p)
    }
    fn is_file(&self, p: &Path) -> bool {
        Disk.is_file(p)
    }
}

/// The design with its loop taken out of `layers/cycles.toml` and declared on
/// group `on`, in the layer file that holds that group, in the same words.
fn declared_on(on: &str, layer: &str) -> Tree {
    let r = root();
    let cycles = std::fs::read_to_string(r.join("layers/cycles.toml")).unwrap();
    let at = cycles
        .find("\n[[iterate]]")
        .map(|i| i + 1)
        .expect("the design declares its loop");
    let the_loop = cycles[at..]
        .replace("[[iterate]]", "[[group.iterate]]")
        .replace("[[iterate.seed]]", "[[group.iterate.seed]]");
    let file = r.join("layers").join(layer);
    let text = std::fs::read_to_string(&file).unwrap();
    let key = format!("id = \"{on}\"");
    let start = text.find(&key).expect("the group is in its layer file");
    // After the group's own keys: at the next table header, or the end.
    let end = text[start..]
        .find("\n[")
        .map_or(text.len(), |i| start + i + 1);
    let edited = format!("{}{}\n{}", &text[..end], the_loop, &text[end..]);
    let mut files = BTreeMap::new();
    files.insert(r.join("layers/cycles.toml"), cycles[..at].to_string());
    files.insert(file, edited);
    load_all_from(&Edited(files), &r).unwrap()
}

fn v11b(tree: &Tree) -> Verdict {
    validate_tree(tree)
        .into_iter()
        .find(|c| c.name.starts_with("V11b"))
        .expect("the gate checks where loops are declared")
        .verdict
}

#[test]
fn the_design_s_loop_is_noted_with_the_block_it_belongs_on() {
    let tree = vleo_sheet::load_all(&root()).unwrap();
    assert_eq!(tree.cycles.len(), 1);
    assert_eq!(tree.cycles[0].on, "");
    match v11b(&tree) {
        Verdict::Note(why) => assert!(
            why.contains("declared on no block; the smallest that holds it is root"),
            "{why}"
        ),
        other => panic!("{other:?}"),
    }
}

#[test]
fn declared_on_its_block_the_loop_runs_as_it_does_today() {
    let tree = declared_on("root", "root.toml");
    assert_eq!(tree.cycles.len(), 1);
    assert_eq!(tree.cycles[0].on, "root");
    assert_eq!(v11b(&tree), Verdict::Pass);
    // The engine holds the same loop — its rows, what it converges on, its
    // stopping rule and its seed — as the one declared on no block.
    let g = opened::graph(&tree).unwrap();
    let today = opened::graph(&vleo_sheet::load_all(&root()).unwrap()).unwrap();
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
            why.contains("declared on l3_power, and the smallest block that holds it is root"),
            "{why}"
        ),
        other => panic!("{other:?}"),
    }
}
