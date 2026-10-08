//! Today's design is built from every group's latest sealed release that
//! passes its checks; a refused release is replaced by its group's last good
//! one and marked, by its reason; and the design says which releases it used.
//!
//! docs/OPERATING_1_0.md, section 5. The releases are the two the solar group
//! sealed in the group application, kept as test fixtures
//! (`crates/vleo-files/tests/fixtures`): 1.1 passes every check, and 1.0 has a
//! node whose results never test what it must refuse — the reason 1.1 was
//! sealed.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use vleo_server::today::{build, Today};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A drive holding the named copies of the solar releases, each as
/// `(fixture version, file name on the drive)`.
fn drive(name: &str, releases: &[(&str, &str)]) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vleo-today-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let rel = d.join("groups/l3_solar/releases");
    std::fs::create_dir_all(&rel).unwrap();
    for (v, file) in releases {
        std::fs::copy(
            root().join(format!(
                "crates/vleo-files/tests/fixtures/l3_solar-{v}.vleo"
            )),
            rel.join(file),
        )
        .unwrap();
    }
    d
}

fn today(d: &Path) -> Today {
    build(Arc::new(vleo_sheet::files::Disk), &root(), d).expect("today's design builds")
}

fn sql(file: &Path, statement: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    assert!(db.execute(statement, []).unwrap() > 0, "{statement}");
}

/// Make a copy of 1.1 into its 1.2, as its group would: the version, and the
/// version record saying what 1.2 changed and why.
fn as_1_2(file: &Path) {
    sql(file, "UPDATE meta SET value = '1.2' WHERE key = 'version'");
    sql(
        file,
        "UPDATE tbl SET csv = csv || '1.2,2026-10-06,Aman Rai,\
         The bands were right and their method said them plainly.,\
         The method was read again against its cases.,\
         A line saying what the method is helps the next reader.,\
         A first line saying what the method does.,,,\
         the four NOAA SWPC activity levels,a correction to those levels\n' \
         WHERE scope = 'group' AND path = 'versions.csv'",
    );
}

/// Seal a release again, as its subsystem engineer would after a change: the
/// fingerprint of its folder as it now is, worked out by the library the
/// group application seals with.
fn reseal(file: &Path) {
    let r = vleo_files::intake::Release::open(file).unwrap();
    let fp = vleo_files::seal::fingerprint_of(&r.folder, "group");
    sql(
        file,
        &format!("UPDATE meta SET value = '{fp}' WHERE key = 'fingerprint'"),
    );
}

#[test]
fn a_group_s_latest_release_that_passes_is_taken_and_said() {
    let d = drive(
        "latest",
        &[("1.0", "l3_solar-1.0.vleo"), ("1.1", "l3_solar-1.1.vleo")],
    );
    let t = today(&d);
    assert_eq!(t.groups.len(), 1);
    let g = &t.groups[0];
    let k = g.taken.as_ref().expect("a release is taken");
    assert_eq!(k.version, "1.1");
    assert!(g.refused.is_empty(), "{:?}", g.refused);
    assert_eq!(g.said(), "l3_solar 1.1 — its latest");
    // The design took 1.1 in through the developer's intake already, so it
    // changes no sheet: today's design from the drive is the design.
    assert!(k.changes.is_empty(), "{:?}", k.changes);
    // What is not yet checked is said, not passed over.
    assert!(k
        .notes
        .iter()
        .any(|n| n.contains("not yet checked through the chain")));
    // The design built answers as the design does: the same graph.
    let tree = vleo_sheet::load::load_all_from(&*t.files, &root()).unwrap();
    let g = vleo_modules::opened::graph(&tree).unwrap();
    let sheets = vleo_modules::opened::graph(&vleo_sheet::load_all(&root()).unwrap()).unwrap();
    assert_eq!(g.graph_hash(), sheets.graph_hash());
}

#[test]
fn a_refused_release_is_replaced_by_the_group_s_last_good_one_and_marked() {
    // A release that says it is 1.2, its files no longer the ones sealed.
    let d = drive(
        "fallback",
        &[("1.1", "l3_solar-1.1.vleo"), ("1.1", "l3_solar-1.2.vleo")],
    );
    sql(
        &d.join("groups/l3_solar/releases/l3_solar-1.2.vleo"),
        "UPDATE meta SET value = '1.2' WHERE key = 'version'",
    );
    let t = today(&d);
    let g = &t.groups[0];
    assert_eq!(g.taken.as_ref().map(|k| k.version.as_str()), Some("1.1"));
    assert_eq!(g.refused.len(), 1);
    let (v, why) = &g.refused[0];
    assert_eq!(v, "1.2");
    assert!(why.contains("its files are not the ones sealed"), "{why}");
    assert!(
        g.said().starts_with(
            "l3_solar 1.1 — its latest, 1.2, is refused: its files are not the ones sealed"
        ),
        "{}",
        g.said()
    );
}

#[test]
fn a_group_with_no_release_that_passes_keeps_the_design_s_own_sheets_and_says_why() {
    // 1.1 with a method edited after the seal; 1.0, whose content does not
    // hold: neither is taken, each for its own reason.
    let d = drive(
        "none",
        &[("1.0", "l3_solar-1.0.vleo"), ("1.1", "l3_solar-1.1.vleo")],
    );
    sql(
        &d.join("groups/l3_solar/releases/l3_solar-1.1.vleo"),
        "UPDATE doc SET body = body || '\n# edited after the seal' WHERE kind = 'pseudocode' \
         AND scope = (SELECT uid FROM node WHERE id = 'sw_activity_band')",
    );
    let t = today(&d);
    let g = &t.groups[0];
    assert!(g.taken.is_none());
    let refused: Vec<(&str, &str)> = g
        .refused
        .iter()
        .map(|(v, w)| (v.as_str(), w.as_str()))
        .collect();
    assert_eq!(refused.len(), 2, "{refused:?}");
    assert_eq!(refused[0].0, "1.1");
    assert!(
        refused[0].1.contains("its files are not the ones sealed"),
        "{}",
        refused[0].1
    );
    assert_eq!(refused[1].0, "1.0");
    assert!(
        refused[1]
            .1
            .contains("sw_activity_band: its results have no case the node must refuse"),
        "{}",
        refused[1].1
    );
    assert!(
        g.said()
            .contains("no release passes; the design's own sheets"),
        "{}",
        g.said()
    );
}

#[test]
fn a_release_in_another_group_s_folder_is_refused() {
    let d = drive("elsewhere", &[]);
    let other = d.join("groups/l3_power/releases");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::copy(
        root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo"),
        other.join("l3_power-1.0.vleo"),
    )
    .unwrap();
    let t = today(&d);
    let g = t.groups.iter().find(|g| g.id == "l3_power").unwrap();
    assert!(g.taken.is_none());
    assert!(
        g.refused[0]
            .1
            .contains("it says it is l3_solar's release, in l3_power's folder"),
        "{:?}",
        g.refused
    );
}

#[test]
fn what_is_not_a_release_is_ignored_and_said() {
    let d = drive(
        "ignored",
        &[
            ("1.1", "l3_solar-1.1.vleo"),
            ("1.1", "l3_solar-1.1 (1).vleo"),
        ],
    );
    std::fs::write(d.join("groups/l3_solar/releases/notes.txt"), "a note").unwrap();
    let t = today(&d);
    assert_eq!(t.groups[0].taken.as_ref().unwrap().version, "1.1");
    assert_eq!(t.ignored.len(), 2, "{:?}", t.ignored);
    assert!(t
        .ignored
        .iter()
        .any(|i| i.contains("notes.txt: not a release")));
    assert!(t
        .ignored
        .iter()
        .any(|i| i.contains("(1).vleo: two files where there should be one")));
}

#[test]
fn a_group_s_new_release_changes_today_s_design_and_its_answers_run() {
    // Solar 1.2: sw_activity_band's method, said again with a line of its
    // node engineer's own, sealed.
    let d = drive(
        "changed",
        &[("1.1", "l3_solar-1.1.vleo"), ("1.1", "l3_solar-1.2.vleo")],
    );
    let f = d.join("groups/l3_solar/releases/l3_solar-1.2.vleo");
    as_1_2(&f);
    sql(
        &f,
        "UPDATE doc SET body = '# 1.2: the same four bands, said again.\n' || body \
         WHERE kind = 'pseudocode' AND scope = 'sw_activity_band'",
    );
    reseal(&f);
    let t = today(&d);
    let g = &t.groups[0];
    let k = g.taken.as_ref().expect("1.2 is taken");
    assert_eq!(k.version, "1.2", "{:?}", g.refused);
    assert_eq!(k.changes, ["sw_activity_band"]);
    // The design holds the release's method, and runs it in the interpreter.
    let tree = vleo_sheet::load::load_all_from(&*t.files, &root()).unwrap();
    let sh = &tree.sheets["sw_activity_band"];
    assert!(
        sh.method
            .text
            .contains("# 1.2: the same four bands, said again."),
        "{}",
        sh.method.text
    );
    let graph = vleo_modules::opened::graph(&tree).unwrap();
    let k = graph.find("sw_activity_band").unwrap();
    assert!(matches!(graph.run.get(k as usize), Some(Some(_))));
    // A comment changes no answer: on each of its fixtures it answers as the
    // design in the repository does.
    let sheets = vleo_modules::opened::graph(&vleo_sheet::load_all(&root()).unwrap()).unwrap();
    let c = sheets.find("sw_activity_band").unwrap();
    for f in graph.nodes[k as usize].fixtures {
        assert_eq!(
            graph.probe(k, f.inputs).map(|v| v[0]),
            sheets.probe(c, f.inputs).map(|v| v[0])
        );
    }
    // The design on disk is not touched.
    let on_disk = vleo_sheet::load_all(&root()).unwrap();
    assert!(!on_disk.sheets["sw_activity_band"]
        .method
        .text
        .contains("# 1.2"));
}

#[test]
fn a_release_whose_relation_an_assistant_supplied_is_refused() {
    // Solar 1.2, sealed, saying an assistant supplied sw_activity_band's
    // relation (AGENTS.md, rule 6).
    let d = drive(
        "assistant",
        &[("1.1", "l3_solar-1.1.vleo"), ("1.1", "l3_solar-1.2.vleo")],
    );
    let f = d.join("groups/l3_solar/releases/l3_solar-1.2.vleo");
    as_1_2(&f);
    sql(
        &f,
        "UPDATE tbl SET csv = replace(csv, ',transcribed,', ',relation,') \
         WHERE scope = 'sw_activity_band' AND path = 'declaration.csv'",
    );
    sql(
        &f,
        "UPDATE doc SET body = '# 1.2\n' || body \
         WHERE kind = 'pseudocode' AND scope = 'sw_activity_band'",
    );
    reseal(&f);
    let t = today(&d);
    let g = &t.groups[0];
    assert_eq!(g.taken.as_ref().map(|k| k.version.as_str()), Some("1.1"));
    let (v, why) = &g.refused[0];
    assert_eq!(v, "1.2");
    assert!(
        why.contains("sw_activity_band") && why.contains("assistant"),
        "{why}"
    );
}
