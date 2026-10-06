//! The design, converted to its files and read back, is the design: every
//! row, heading, wire, case and source as the repository holds it, with only
//! what the conversion is for changed, each named (docs/PLAN_1_0.md, phase E;
//! `src/convert.rs`).
//!
//! The design on disk is read and never written: the files are made in
//! memory and in a scratch folder.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use vleo_files::convert::{self, proposed_id, Served, PROGRAMME, PROPOSED, SYSTEMS};
use vleo_files::meta::Kind;
use vleo_files::model::File;
use vleo_sheet::files::Disk;
use vleo_sheet::load::{self, Tree};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tree() -> Tree {
    load::load_all(&root()).expect("the design loads")
}

/// The design converted once, for every test here.
fn converted() -> &'static [(String, File)] {
    static FILES: OnceLock<Vec<(String, File)>> = OnceLock::new();
    FILES
        .get_or_init(|| convert::convert(&tree(), &Disk, "vleo test").expect("the design converts"))
}

fn served() -> Tree {
    let s = Served::new(&root(), converted(), Arc::new(Disk)).expect("the files are served");
    load::load_all_from(&s, &root()).expect("the files load as the design")
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
    // A node file for every row, and for every block proposed.
    let proposed: usize = PROPOSED.iter().map(|(_, n)| n.len()).sum();
    assert_eq!(of(Kind::Node).count(), t.sheets.len() + proposed);
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
fn the_files_read_back_as_the_design_with_only_what_the_conversion_is_for() {
    let mut expected = tree();
    let back = served();

    // 1. Each subsystem group hangs from the block it mounts on, as the
    //    system model says, not from the root.
    let model = mounts_in_the_model();
    for (group, on) in &model {
        assert_eq!(back.groups[group].parent, *on, "{group}");
        assert_eq!(expected.groups[group].parent, "root", "{group}");
        expected.groups.get_mut(group).unwrap().parent = on.clone();
    }

    // 2. The architecture's loop is declared on the smallest block that holds
    //    every row it runs through.
    assert_eq!(back.cycles.len(), expected.cycles.len());
    for (b, e) in back.cycles.iter().zip(expected.cycles.iter_mut()) {
        assert!(e.on.is_empty() && !b.on.is_empty(), "{:?}", b.on);
        let holds = |heading: &str, row: &str| {
            let mut at = back.sheets[row].parent.clone();
            loop {
                if at == heading {
                    return true;
                }
                match back.groups.get(&at) {
                    Some(g) if !g.parent.is_empty() => at = g.parent.clone(),
                    _ => return false,
                }
            }
        };
        assert!(
            b.nodes.iter().all(|n| holds(&b.on, n)),
            "{} holds them all",
            b.on
        );
        for child in back.groups.values().filter(|g| g.parent == b.on) {
            assert!(
                !b.nodes.iter().all(|n| holds(&child.id, n)),
                "{} is smaller, and holds them all",
                child.id
            );
        }
        e.on = b.on.clone();
    }
    for c in expected.cases.values_mut() {
        for (cy, b) in c.cycles.iter_mut().zip(&back.cycles) {
            cy.on = b.on.clone();
        }
    }

    // 3. Every stated value is a parameter of the level whose branch states it.
    let level = |id: &str| {
        let mut g = back.sheets[id].parent.clone();
        loop {
            let h = &back.groups[&g];
            match back.groups.get(&h.parent) {
                Some(p) if p.layer == h.layer => g = p.id.clone(),
                _ => {
                    return match h.layer {
                        0 | 1 => "programme",
                        2 => "system",
                        _ => "subsystem",
                    }
                }
            }
        }
    };
    let mut stated = 0;
    for sh in expected.sheets.values_mut() {
        if sh.is_declared() && !sh.is_seeded() {
            assert_eq!(sh.port.parameter, "", "{}", sh.id);
            sh.port.parameter = level(&sh.id).to_string();
            stated += 1;
        }
    }
    assert!(stated > 0);

    // 4. The blocks the breakdown does not hold yet are there, open, under
    //    their group, each saying why it was proposed.
    let mut back_sheets = back.sheets.clone();
    for (group, names) in PROPOSED {
        for (name, why) in *names {
            let id = proposed_id(group, name);
            let sh = back_sheets.remove(&id).unwrap_or_else(|| panic!("{id}"));
            assert_eq!((sh.behaviour(), sh.parent.as_str()), ("open", *group));
            assert_eq!((sh.label.as_str(), sh.layer), (*name, 3));
            assert_eq!(sh.note, *why, "{id}");
        }
    }

    // And nothing else: every row, heading, relation, case and source.
    assert_eq!(
        back_sheets.keys().collect::<Vec<_>>(),
        expected.sheets.keys().collect::<Vec<_>>()
    );
    for (id, sh) in &expected.sheets {
        assert_eq!(format!("{:?}", back_sheets[id]), format!("{sh:?}"), "{id}");
    }
    assert_eq!(
        format!("{:?}", back.groups),
        format!("{:?}", expected.groups)
    );
    assert_eq!(
        format!("{:?}", back.relations),
        format!("{:?}", expected.relations)
    );
    assert_eq!(
        format!("{:?}", back.cycles),
        format!("{:?}", expected.cycles)
    );
    assert_eq!(format!("{:?}", back.cases), format!("{:?}", expected.cases));
    assert_eq!(
        format!("{:?}", back.sources),
        format!("{:?}", expected.sources)
    );
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
            // And no row of the group already has its name.
            assert!(
                t.sheets
                    .values()
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
    let s = Served::new(&root(), &files, Arc::new(Disk)).unwrap();
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
    let e = Served::new(&root(), &files, Arc::new(Disk))
        .err()
        .expect("refused");
    assert!(
        e.message()
            .contains("its block says it is method, and what it holds makes it built-in"),
        "{e}"
    );
}
