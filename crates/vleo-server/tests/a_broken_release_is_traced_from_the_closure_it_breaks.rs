//! A group seals a release whose method is wrong where its own cases do not
//! look; today's design takes it, as it takes every release that passes its
//! checks; and the health map traces the closure it breaks to that node, by
//! name, with the release that changed it, its group and owner. A method
//! wrong where its cases do look never reaches today's design.
//!
//! docs/OPERATING_1_0.md, sections 5 and 6, end to end: the drive, today's
//! design, the graph read from its files, the map and the trace. The release
//! is the solar group's 1.1, kept as a test fixture
//! (`crates/vleo-files/tests/fixtures`), made into a 1.2 and sealed again as
//! its subsystem engineer would.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use vleo_bus::Case;
use vleo_modules::health::{health, trace, trace_since, State};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sql(file: &Path, statement: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    assert!(db.execute(statement, []).unwrap() > 0, "{statement}");
}

/// The case with the reference data, verified into a store of the test's
/// own.
fn with_data(scratch: &Path) -> Case {
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    let mut store = vleo_data::Store::open(&scratch.join("data"));
    store
        .sync(&vleo_data::Source::Shipped(root().join("bundles")))
        .expect("the shipped bundles verify");
    Case {
        data: store.verified_names(),
        ..Default::default()
    }
}

/// A drive holding solar 1.1 as its group sealed it, and a 1.2 made from it
/// as its group would: `sw_f107_design_long`'s method with `from` replaced by
/// `to`, the version record saying why, sealed again.
fn drive(name: &str, from: &str, to: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vleo-health-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let rel = d.join("groups/l3_solar/releases");
    std::fs::create_dir_all(&rel).unwrap();
    let fixture = root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo");
    std::fs::copy(&fixture, rel.join("l3_solar-1.1.vleo")).unwrap();
    let f = rel.join("l3_solar-1.2.vleo");
    std::fs::copy(&fixture, &f).unwrap();
    sql(&f, "UPDATE meta SET value = '1.2' WHERE key = 'version'");
    sql(
        &f,
        "UPDATE tbl SET csv = csv || '1.2,2026-10-06,Aman Rai,\
         The band needed a wider margin when the cycle is low.,\
         The multiplier was read again.,\
         A wider band is safer.,\
         The multiplier of the hot edge.,,,\
         the one-sided 90th percentile,a multiplier from another table\n' \
         WHERE scope = 'group' AND path = 'versions.csv'",
    );
    sql(
        &f,
        &format!(
            "UPDATE doc SET body = replace(body, '{from}', '{to}') \
             WHERE kind = 'pseudocode' AND scope = 'sw_f107_design_long'"
        ),
    );
    let r = vleo_files::intake::Release::open(&f).unwrap();
    let fp = vleo_files::seal::fingerprint_of(&r.folder, "group");
    sql(
        &f,
        &format!("UPDATE meta SET value = '{fp}' WHERE key = 'fingerprint'"),
    );
    d
}

fn today(d: &Path) -> vleo_server::today::Today {
    vleo_server::today::build(Arc::new(vleo_sheet::files::Disk), &root(), d)
        .expect("today's design builds")
}

#[test]
fn a_method_wrong_where_its_cases_look_never_reaches_today_s_design() {
    // The multiplier from another table, everywhere: its own cases see it.
    let d = drive("seen", "const z = 1.28 [1]", "const z = 14 [1]");
    let t = today(&d);
    let g = &t.groups[0];
    assert_eq!(g.taken.as_ref().map(|k| k.version.as_str()), Some("1.1"));
    let (v, why) = &g.refused[0];
    assert_eq!(v, "1.2");
    assert!(
        why.contains("sw_f107_design_long: its method does not reproduce"),
        "{why}"
    );
}

#[test]
fn a_method_wrong_where_its_cases_do_not_look_is_traced_by_name_with_its_release() {
    // The multiplier from another table, only below a centre of 95 sfu —
    // where none of the node's own cases is, and where the design's central
    // expectation is.
    let d = drive(
        "unseen",
        "return sustained",
        "if central < 95 then\n  return central + spread * 14\nend\nreturn sustained",
    );
    let t = today(&d);
    let k = t.groups[0].taken.as_ref().expect("a release is taken");
    assert_eq!(k.version, "1.2", "{:?}", t.groups[0].refused);
    assert_eq!(k.changes, ["sw_f107_design_long"]);

    // The graph read from today's design runs the release's method.
    let tree = vleo_sheet::load::load_all_from(&*t.files, &root()).unwrap();
    let g = vleo_modules::opened::graph(&tree).unwrap();
    let case = with_data(&d);
    let map = health(g, &case);
    let closure = g.find("l3_solar_ach_01").unwrap();
    assert_eq!(
        map.node(closure).state,
        State::Fails,
        "{}",
        map.node(closure).why
    );
    assert_eq!(map.spacecraft, State::Fails);
    // It reproduces its own cases: nothing on the map alone marks it out from
    // every other row the closure reads.
    let node = g.find("sw_f107_design_long").unwrap();
    assert_eq!(map.node(node).state, State::Fails);
    assert!(trace(g, &case, &map, closure).causes.is_empty());

    // What does is that it changed: traced with what changed since the
    // design's own sheets, it is the cause, by name, with its release, group
    // and owner.
    let changed: Vec<(u16, String)> = t
        .changed()
        .into_iter()
        .filter_map(|(id, said)| g.find(&id).map(|k| (k, said)))
        .collect();
    let tr = trace_since(g, &case, &map, closure, &changed);
    let c = tr.causes.first().expect("the closure has a cause");
    assert_eq!(g.nodes[c.node as usize].id, "sw_f107_design_long");
    assert!(c.why.starts_with("changed by l3_solar 1.2"), "{}", c.why);
    assert_eq!((c.group, c.owner), ("l3_solar", "environment"));

    // Its last good release, 1.1, closes: the break is the release's.
    std::fs::remove_file(d.join("groups/l3_solar/releases/l3_solar-1.2.vleo")).unwrap();
    let t = today(&d);
    assert!(t.changed().is_empty());
    let tree = vleo_sheet::load::load_all_from(&*t.files, &root()).unwrap();
    let g = vleo_modules::opened::graph(&tree).unwrap();
    let map = health(g, &case);
    assert_eq!(
        map.node(g.find("l3_solar_ach_01").unwrap()).state,
        State::Closes
    );
}
