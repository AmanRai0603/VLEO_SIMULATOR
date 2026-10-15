//! A node's revision is saved in place in the working copy, and a second
//! writer is stopped with nothing written (docs/briefs/F3.md, F3-a).
//!
//! The node is the design's own `sw_ap_design`, copied into a scratch copy
//! beside an invented group file that assigns it to Test Nia, whose deputy is
//! Test Dee. Test Ivo is assigned nothing.

use std::path::{Path, PathBuf};
use vleo_files::chain::Registry;
use vleo_files::copied::digest;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::save::{save, Saved};
use vleo_files::sqlite;

const NODE: &str = "groups/l3_solar/nodes/sw_ap_design.vnode";

fn s(v: &str) -> String {
    v.to_string()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A copy holding the node, and the group file that assigns it.
fn copy(name: &str) -> (PathBuf, Registry) {
    let c = std::env::temp_dir().join(format!("vleo-save-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&c);
    std::fs::create_dir_all(c.join("groups/l3_solar/nodes")).unwrap();
    std::fs::copy(root().join("design").join(NODE), c.join(NODE)).unwrap();
    let mut g = File::new(Kind::Group, "vleo 1.0.0 (test)");
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("writer", "Test Ola"),
        ("based_on", ""),
    ] {
        g.meta.insert(s(k), s(v));
    }
    for (name, deputy_for) in [("Test Nia", ""), ("Test Dee", "Test Nia"), ("Test Ivo", "")] {
        g.people.push(Person {
            name: s(name),
            role: s("node engineer"),
            deputy_for: s(deputy_for),
        });
    }
    g.assignments.push(Assignment {
        block_uid: s("sw_ap_design"),
        person: s("Test Nia"),
        contract_version: 1,
    });
    (
        c,
        Registry {
            from: s("l3_solar's group file"),
            file: g,
        },
    )
}

/// The node as the person opened it: the file, and the digest of its bytes.
fn open(c: &Path) -> (File, String) {
    let bytes = std::fs::read(c.join(NODE)).unwrap();
    (sqlite::read(&c.join(NODE)).unwrap(), digest(&bytes))
}

/// An edit a node engineer makes: a line of their own on the question.
fn edited(f: &File) -> File {
    let mut e = f.clone();
    e.texts.push(Text {
        scope: s("sw_ap_design"),
        kind: s("note"),
        body: s("A line of Test Nia's own."),
    });
    e
}

#[test]
fn a_revision_is_saved_whole_as_the_next_one_by_its_writer() {
    let (c, g) = copy("saved");
    let (f, opened) = open(&c);
    match save(&c, NODE, &edited(&f), &opened, "Test Nia", &g).unwrap() {
        Saved::Written {
            revision,
            digest: d,
            ..
        } => {
            assert_eq!(revision, 2);
            assert_eq!(d, digest(&std::fs::read(c.join(NODE)).unwrap()));
        }
        other => panic!("not written: {other:?}"),
    }
    let now = sqlite::read(&c.join(NODE)).unwrap();
    assert_eq!(now.meta.get("revision").map(String::as_str), Some("2"));
    assert_eq!(now.meta.get("writer").map(String::as_str), Some("Test Nia"));
    assert_eq!(now.blocks[0].revision, 2);
    assert!(now
        .texts
        .iter()
        .any(|t| t.body == "A line of Test Nia's own."));
    assert!(
        !c.join("groups/l3_solar/nodes/sw_ap_design.partial")
            .exists(),
        "the file beside it is gone once renamed over the old one"
    );
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_second_writer_is_stopped_and_nothing_is_written() {
    let (c, g) = copy("second");
    let (f, opened) = open(&c);
    // Someone else saves first: Test Dee, for Nia, on the same copy.
    let (theirs, _) = open(&c);
    match save(&c, NODE, &edited(&theirs), &opened, "Test Dee", &g).unwrap() {
        Saved::Written { revision: 2, .. } => {}
        other => panic!("the first save did not write: {other:?}"),
    }
    let before = std::fs::read(c.join(NODE)).unwrap();
    match save(&c, NODE, &edited(&f), &opened, "Test Nia", &g).unwrap() {
        Saved::SecondWriter {
            writer,
            revision,
            now,
            digest: d,
        } => {
            assert_eq!((writer.as_str(), revision), ("Test Dee", 2));
            assert_eq!(now.meta.get("writer").map(String::as_str), Some("Test Dee"));
            assert_eq!(d, digest(&before));
        }
        other => panic!("the second writer was not stopped: {other:?}"),
    }
    assert_eq!(
        std::fs::read(c.join(NODE)).unwrap(),
        before,
        "the file is the same, byte for byte"
    );
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_file_touched_and_not_changed_is_not_a_second_writer() {
    let (c, g) = copy("touched");
    let (f, opened) = open(&c);
    let bytes = std::fs::read(c.join(NODE)).unwrap();
    std::fs::write(c.join(NODE), &bytes).unwrap();
    assert!(matches!(
        save(&c, NODE, &edited(&f), &opened, "Test Nia", &g).unwrap(),
        Saved::Written { revision: 2, .. }
    ));
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn only_its_assigned_writer_or_their_deputy_writes_it() {
    let (c, g) = copy("who");
    let (f, opened) = open(&c);
    let before = std::fs::read(c.join(NODE)).unwrap();
    let e = save(&c, NODE, &edited(&f), &opened, "Test Ivo", &g).unwrap_err();
    assert!(
        e.message()
            .contains("Test Ivo is not assigned sw_ap_design"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    assert!(matches!(
        save(&c, NODE, &edited(&f), &opened, "Test Dee", &g).unwrap(),
        Saved::Written { .. }
    ));
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_save_does_not_move_a_node_or_reach_outside_the_copy() {
    let (c, g) = copy("moved");
    let (f, opened) = open(&c);
    let before = std::fs::read(c.join(NODE)).unwrap();
    let mut moved = edited(&f);
    moved.meta.insert(s("block_uid"), s("sw_ap_other"));
    let e = save(&c, NODE, &moved, &opened, "Test Nia", &g).unwrap_err();
    assert!(
        e.message().contains("a save does not move a node"),
        "{}",
        e.message()
    );
    for path in ["../elsewhere.vnode", "/etc/passwd", "groups/../x.vnode"] {
        let e = save(&c, path, &edited(&f), &opened, "Test Nia", &g).unwrap_err();
        assert!(
            e.message().contains("not a path inside the copy"),
            "{path}: {}",
            e.message()
        );
    }
    let mut other = g.clone();
    other.file.meta.insert(s("group_id"), s("l3_power"));
    let e = save(&c, NODE, &edited(&f), &opened, "Test Nia", &other).unwrap_err();
    assert!(
        e.message().contains("the group file given is l3_power's"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&c);
}
