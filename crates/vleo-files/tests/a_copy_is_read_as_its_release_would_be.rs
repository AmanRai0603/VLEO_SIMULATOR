//! A node file of a working copy, read as the release its group would seal
//! it in (docs/briefs/F3.md, F3-g2b; `vleo_files::intake::copy_form`), for
//! the page of the copy's revision over today's design.
//!
//! - Solar 1.1's `sw_activity_band`, in a node file of its own, is laid out
//!   as the release lays it out, `nodes/<id>/…`, and nothing of another
//!   scope is; it says it is the copy's revision, not sealed.
//! - Taken by today's design's own intake, it is the change the release
//!   itself is: the same method, cases, declaration and author.
//! - Planned against the design, it changes nothing the design does not
//!   already hold.
//! - Its method written anew, with its declaration and its record, as the
//!   Node workspace writes it (`vleo_files::edit`), it is planned as a change
//!   of the method, recorded with that record, dated by it, and said to come
//!   from a working copy.
//! - A file that holds two blocks, or a block it does not say it is, and a
//!   file of another node than the sheet's, are refused, by name.

use std::path::{Path, PathBuf};
use vleo_files::edit::{self, Declared, Edit, Edits};
use vleo_files::intake;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_sheet::template::{self, Derisk, Verdict};
use vleo_sheet::Tree;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-17T09:00:00Z";
const BAND: &str = "sw_activity_band";
const PATH: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";

fn s(v: &str) -> String {
    v.to_string()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/l3_solar-1.1.vleo")
}

/// The design, read from `design/` once for this test binary.
fn design() -> &'static Tree {
    static DESIGN: std::sync::OnceLock<Tree> = std::sync::OnceLock::new();
    DESIGN.get_or_init(|| vleo_files::convert::open(&root()).unwrap().0)
}

/// `uid` from solar 1.1's release, in a node file of its own.
fn from_release(uid: &str) -> File {
    let how = vleo_files::upgrade::Upgrade { app: APP, at: AT };
    let rel = vleo_files::sqlite::open(&fixture(), &how).unwrap().file;
    let mut n = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("block_uid", uid),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Nia Node"),
    ] {
        n.meta.insert(s(k), s(v));
    }
    n.blocks = vec![rel.blocks.iter().find(|b| b.uid == uid).unwrap().clone()];
    n.blocks[0].revision = 1;
    n.ports = rel
        .ports
        .iter()
        .filter(|p| p.block_uid == uid)
        .cloned()
        .collect();
    n.wires = rel
        .wires
        .iter()
        .filter(|w| w.to_block == uid)
        .cloned()
        .collect();
    n.texts = rel
        .texts
        .iter()
        .filter(|t| t.scope == uid)
        .cloned()
        .collect();
    n.tables = rel
        .tables
        .iter()
        .filter(|t| t.scope == uid)
        .cloned()
        .collect();
    n
}

/// The band's sheet in the design, and the hash its plan is made against.
fn sheet() -> (&'static vleo_sheet::model::Sheet, String) {
    let sh = &design().sheets[BAND];
    (sh, vleo_sheet::form::file_hash(&sh.text))
}

#[test]
fn the_copy_is_laid_out_as_its_release_is() {
    let mut f = from_release(BAND);
    // A text of the file's own scope, and one of another node: neither is
    // the node's.
    f.texts.push(Text {
        scope: s("file"),
        kind: s("explanation"),
        body: s("not the band's"),
    });
    f.texts.push(Text {
        scope: s("sw_ap_design"),
        kind: s("pseudocode"),
        body: s("not the band's either"),
    });
    let (r, derisk, id) = intake::copy_release(&f).unwrap();
    assert_eq!(id, BAND);
    assert_eq!(
        (r.group.as_str(), r.version.as_str(), r.sealed.as_str()),
        ("l3_solar", "revision 1", "")
    );
    assert!(r.check_seal().is_err(), "a copy is not sealed, and says so");
    assert!(
        r.folder
            .keys()
            .all(|p| p.starts_with("nodes/sw_activity_band/")),
        "{:?}",
        r.folder.keys().collect::<Vec<_>>()
    );
    for p in [
        "pseudocode.txt",
        "declaration.csv",
        "results/isolation.csv",
        "explanation.md",
    ] {
        assert!(
            r.folder
                .contains_key(&format!("nodes/sw_activity_band/{p}")),
            "{p}: {:?}",
            r.folder.keys().collect::<Vec<_>>()
        );
    }
    assert!(!r
        .folder
        .values()
        .any(|b| b.as_slice() == b"not the band's either" || b.as_slice() == b"not the band's"));
    assert_eq!(derisk, Derisk::default(), "no record is read from nowhere");

    // Taken by today's design's own intake, the copy is the change its
    // release is.
    let (sh, base) = sheet();
    let release = intake::Release::open(&fixture()).unwrap();
    let (theirs, _) = intake::node_form(
        &root(),
        &release,
        sh,
        base.clone(),
        &release.latest_version(),
    )
    .unwrap();
    let (ours, not_taken) = intake::copy_form(&root(), &f, sh, base).unwrap();
    assert_eq!(not_taken, None);
    assert_eq!(ours.filled.fields, theirs.filled.fields);
    assert_eq!(ours.filled.arrays, theirs.filled.arrays);
    assert_eq!(
        (ours.name.as_str(), ours.ai.as_str()),
        (theirs.name.as_str(), theirs.ai.as_str())
    );
    assert!(
        ours.notes.contains("working copy, revision 1: not sealed"),
        "{}",
        ours.notes
    );
    // And the design already holds it: nothing to change.
    let plan = template::plan_form_against(design(), ours).unwrap();
    assert_eq!(plan.applicable(), 0, "{:?}", plan.items);
}

#[test]
fn a_method_written_in_the_copy_is_planned_as_its_change() {
    let f = from_release(BAND);
    let method = f
        .texts
        .iter()
        .find(|t| t.kind == "pseudocode")
        .unwrap()
        .body
        .clone();
    let why = Derisk {
        believed: s("The method said nothing of where its edges come from."),
        tested: s("Read beside NOAA's table of the four levels."),
        learned: s("Its edges are NOAA's, and nothing said so."),
        changed: s("A line says where the edges come from."),
        rests_on: s("NOAA's four F10.7 levels."),
        breaks_if: s("NOAA moves an edge."),
        ..Derisk::default()
    };
    let e = Edits {
        edits: vec![Edit::Method(format!(
            "# Nia: the edges are NOAA's.\n{method}"
        ))],
        declared: Some(Declared {
            author: s("Nia Node"),
            ai: s("none"),
            source: s(""),
            checked_by: s(""),
        }),
        why: Some(why.clone()),
    };
    let out = edit::apply(&f, PATH, &e, "Nia Node", AT).unwrap().file;
    let (_, derisk, _) = intake::copy_release(&out).unwrap();
    assert_eq!(
        derisk.changed, why.changed,
        "the record is the node's own last row"
    );
    let (sh, base) = sheet();
    let (form, _) = intake::copy_form(&root(), &out, sh, base).unwrap();
    assert_eq!(form.date, "2026-10-17", "dated by its record");
    assert_eq!(form.name, "Nia Node");
    let plan = template::plan_form_against(design(), form).unwrap();
    let applied: Vec<&str> = plan
        .items
        .iter()
        .filter(|i| i.verdict == Verdict::Apply)
        .map(|i| i.what.as_str())
        .collect();
    assert!(applied.contains(&"method_text"), "{:?}", plan.items);
    assert!(
        applied
            .iter()
            .any(|w| w.starts_with("de-risking · version")),
        "{:?}",
        plan.items
    );
    let text = plan.text.unwrap();
    assert!(text.contains("# Nia: the edges are NOAA's."), "{text}");
    assert!(
        text.contains("Its edges are NOAA's, and nothing said so."),
        "{text}"
    );
}

#[test]
fn a_file_that_is_not_one_node_of_the_sheet_is_refused() {
    let mut two = from_release(BAND);
    let other = two.blocks[0].clone();
    two.blocks.push(Block {
        uid: s("another"),
        ..other
    });
    let e = intake::copy_release(&two).unwrap_err();
    assert!(e.contains("holds 2 blocks: a node file holds one"), "{e}");

    let mut wrong = from_release(BAND);
    wrong.meta.insert(s("block_uid"), s("sw_ap_design"));
    let e = intake::copy_release(&wrong).unwrap_err();
    assert!(e.contains("says it is \"sw_ap_design\""), "{e}");

    let (_, base) = sheet();
    let ap = &design().sheets["sw_ap_design"];
    let e = intake::copy_form(&root(), &from_release(BAND), ap, base).unwrap_err();
    assert!(
        e.contains("holds sw_activity_band, not sw_ap_design: it is not drawn over another node"),
        "{e}"
    );
}
