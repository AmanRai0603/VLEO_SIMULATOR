//! The design's files, `design/`, hold together: every branch, row and case
//! is a file that checks and reads back as written, every block proposed is
//! one the system model proposes, and what a file says is what the design
//! reads (docs/PLAN_1_0.md, phase E; `src/convert.rs`).
//!
//! The design on disk is read and never written: an edit is made to the files
//! in memory, and a file written is written to a scratch folder.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use vleo_files::convert::{self, proposed_id, Served, PROGRAMME, PROPOSED, SYSTEMS};
use vleo_files::meta::Kind;
use vleo_files::model::File;
use vleo_sheet::load::{self, Tree};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tree() -> Tree {
    convert::open(&root()).expect("the design loads").0
}

/// The design's files, read once for every test here.
fn converted() -> &'static [(String, File)] {
    static FILES: OnceLock<Vec<(String, File)>> = OnceLock::new();
    FILES.get_or_init(|| {
        convert::read_folder(&root().join("design"))
            .expect("design/ reads")
            .0
    })
}

/// Where each group mounts, as docs/SYSTEM_MODEL.md, section 8, says.
fn mounts_in_the_model() -> BTreeMap<String, String> {
    let doc = std::fs::read_to_string(root().join("docs/SYSTEM_MODEL.md")).unwrap();
    let at = doc.find("### Where each group mounts").unwrap();
    let mut out = BTreeMap::new();
    for line in doc[at..].lines().skip_while(|l| !l.starts_with("| l3_")) {
        let Some(row) = line.strip_prefix("| ") else {
            break;
        };
        let cells: Vec<&str> = row.split(" | ").collect();
        let group = cells[0].split(" · ").next().unwrap().trim();
        let on = cells[1]
            .trim_end_matches(" |")
            .split(',')
            .next()
            .unwrap()
            .trim();
        out.insert(group.to_string(), on.to_string());
    }
    out
}

#[test]
fn every_branch_every_row_and_every_case_is_a_file_that_holds_together() {
    let t = tree();
    let files = converted();
    let of = |k: Kind| files.iter().filter(move |(_, f)| f.kind().unwrap() == k);
    // The programme's, the systems' and the eighteen subsystem groups'.
    let groups: BTreeSet<&str> = of(Kind::Group)
        .map(|(_, f)| f.meta["group_id"].as_str())
        .collect();
    let model = mounts_in_the_model();
    let mut expected: BTreeSet<&str> = model.keys().map(String::as_str).collect();
    expected.insert(PROGRAMME);
    expected.insert(SYSTEMS);
    assert_eq!(groups, expected);
    assert_eq!(groups.len(), 20);
    // A node file for every row, the blocks proposed among them.
    let proposed: usize = PROPOSED.iter().map(|(_, n)| n.len()).sum();
    assert_eq!(of(Kind::Node).count(), t.sheets.len());
    assert!(PROPOSED.iter().all(|(g, names)| names
        .iter()
        .all(|(n, _)| t.sheets.contains_key(&proposed_id(g, n)))));
    assert_eq!(proposed, 5);
    // The one case, and the two kept as CSV beside it.
    assert_eq!(of(Kind::Case).count(), 3);
    for (path, f) in files {
        f.check_meta().unwrap_or_else(|e| panic!("{path}: {e}"));
        f.check_values().unwrap_or_else(|e| panic!("{path}: {e}"));
    }
    // Each is a file on disk, and reads back as it was written.
    let scratch = std::env::temp_dir().join(format!("vleo-converted-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    for (path, f) in files.iter().filter(|(p, _)| {
        p.ends_with(".vgroup") || p.contains("/l3_solar/") || p.starts_with("cases/")
    }) {
        let p = scratch.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        vleo_files::sqlite::write(f, &p).unwrap();
        assert_eq!(&vleo_files::sqlite::read(&p).unwrap(), f, "{path}");
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

#[test]
fn every_block_proposed_is_one_the_system_model_proposes_and_the_breakdown_has_not() {
    // Each is in the system model's proposed next level, under its group.
    let doc = std::fs::read_to_string(root().join("docs/SYSTEM_MODEL.md")).unwrap();
    let at = doc.find("### Proposed: one level deeper").unwrap();
    let mut in_doc: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for line in doc[at..]
        .lines()
        .skip_while(|l| !l.starts_with("| Propulsion"))
        .take_while(|l| l.starts_with('|'))
    {
        let cells: Vec<&str> = line.trim_matches('|').split(" | ").collect();
        in_doc.insert(
            cells[0].trim().to_lowercase(),
            cells[1]
                .split(" · ")
                .map(|c| c.trim().to_lowercase())
                .collect(),
        );
    }
    let t = tree();
    for (group, names) in PROPOSED {
        let heading = t.groups[*group].label.to_lowercase();
        let row = in_doc
            .iter()
            .find(|(g, _)| heading.starts_with(g.as_str()) || g.starts_with(heading.as_str()))
            .map(|(_, c)| c)
            .unwrap_or_else(|| panic!("{group} ({heading}) has no row in the model's table"));
        for (name, why) in *names {
            assert!(row.contains(&name.to_lowercase()), "{group}: {name}");
            assert!(
                why.starts_with("Proposed when the design was converted"),
                "{name}"
            );
            // And no row but the block itself has its name.
            let id = proposed_id(group, name);
            assert!(
                t.sheets
                    .values()
                    .filter(|s| s.id != id)
                    .all(|s| !s.label.to_lowercase().contains(&name.to_lowercase())),
                "{group} already has {name}"
            );
        }
    }
}

#[test]
fn what_a_file_says_is_what_the_design_reads() {
    // A bound changed in the node file is the bound the loader reads: the
    // rows are the design, not a copy of it.
    let mut files = converted().to_vec();
    let (_, f) = files
        .iter_mut()
        .find(|(p, _)| p.ends_with("/sw_mean_band_spread.vnode"))
        .unwrap();
    let port = f.ports.iter_mut().find(|p| p.direction == "out").unwrap();
    port.upper = "44.5".into();
    // And so is a case's expected answer.
    let (_, g) = files
        .iter_mut()
        .find(|(p, _)| p.ends_with("/sw_f107_design_long.vnode"))
        .unwrap();
    g.cases[0].expected = "1.5".into();
    let s = Served::new(&root(), &files).unwrap();
    let t = load::load_all_from(&s, &root()).unwrap();
    assert_eq!(t.sheets["sw_mean_band_spread"].upper, 44.5);
    assert_eq!(t.sheets["sw_f107_design_long"].fixtures[0].expect, 1.5);

    // A block whose content makes it something else than it says is refused,
    // by name.
    let mut files = converted().to_vec();
    let (_, f) = files
        .iter_mut()
        .find(|(p, _)| p.ends_with("/sw_f107_design_long.vnode"))
        .unwrap();
    f.texts.retain(|t| t.kind != "method");
    let e = Served::new(&root(), &files).err().expect("refused");
    assert!(
        e.message()
            .contains("its block says it is method, and what it holds makes it built-in"),
        "{e}"
    );
}
