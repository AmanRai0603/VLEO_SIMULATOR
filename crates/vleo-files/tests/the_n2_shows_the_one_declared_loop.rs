//! The N2 of the converted design, drawn from its files' wires, shows one
//! loop, and it is the one the design declares, on the block that holds it
//! (docs/PLAN_1_0.md, phase E: "its N2 shows the one declared loop";
//! docs/SYSTEM_MODEL.md, section 5).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use vleo_files::convert;
use vleo_files::model::File;
use vleo_files::n2::Design;
use vleo_sheet::files::Disk;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn converted() -> &'static [(String, File)] {
    static FILES: OnceLock<Vec<(String, File)>> = OnceLock::new();
    FILES.get_or_init(|| {
        let tree = vleo_sheet::load::load_all(&root()).expect("the design loads");
        convert::convert(&tree, &Disk, "vleo test").expect("the design converts")
    })
}

#[test]
fn the_wires_make_one_loop_and_it_is_the_one_declared_on_its_block() {
    let d = Design::of(converted()).unwrap();
    let loops = d.loops();
    assert_eq!(loops.len(), 1, "{loops:?}");
    let l = &loops[0];
    let declared = converted()
        .iter()
        .flat_map(|(_, f)| f.loops.iter())
        .collect::<Vec<_>>();
    assert_eq!(declared.len(), 1);
    let members: BTreeSet<String> = declared[0].members.split(',').map(str::to_string).collect();
    assert_eq!(l.rows, members);
    assert_eq!(l.belongs_on, declared[0].block_uid);
    assert!(l.declared);

    // On its block's N2, the loop is marks both ways between the children
    // that hold its rows: one feeds the other and is fed back.
    let n2 = d.n2(&l.belongs_on);
    assert!(n2.children.len() > 1, "{n2:?}");
    let both_ways = n2
        .marks
        .iter()
        .filter(|(i, j)| n2.marks.contains(&(*j, *i)))
        .count();
    assert!(both_ways > 0, "{:?}", n2.marks);
}

#[test]
fn an_undeclared_loop_is_found_and_said_to_be_undeclared() {
    // The declared loop taken away: the wires still make it, and the N2 says
    // where it belongs and that nothing declares it there.
    let mut files = converted().to_vec();
    for (_, f) in files.iter_mut() {
        f.loops.clear();
    }
    let loops = Design::of(&files).unwrap().loops();
    assert_eq!(loops.len(), 1);
    assert!(!loops[0].declared);
    assert!(!loops[0].belongs_on.is_empty());
}
