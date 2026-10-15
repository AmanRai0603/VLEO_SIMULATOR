//! A node's revision is saved in place in the working copy, and a second
//! writer is stopped with nothing written (docs/briefs/F3.md, F3-a).
//!
//! The node is the design's own `sw_ap_design`, copied into a scratch copy
//! beside an invented group file that assigns it to Test Nia, whose deputy is
//! Test Dee. Test Ivo is assigned nothing, and Test Zed is Ivo's deputy.

use std::path::{Path, PathBuf};
use vleo_files::chain::Registry;
use vleo_files::copied::digest;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::save::{save, Saved};
use vleo_files::sqlite;

const NODE: &str = "groups/l3_solar/nodes/sw_ap_design.vnode";
const APP: &str = "vleo 1.0.0 (test)";

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
    for (name, deputy_for) in [
        ("Test Nia", ""),
        ("Test Dee", "Test Nia"),
        ("Test Ivo", ""),
        ("Test Zed", "Test Ivo"),
    ] {
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
    match save(&c, NODE, &edited(&f), &opened, "Test Nia", APP, &g).unwrap() {
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
    let beside: Vec<_> = std::fs::read_dir(c.join("groups/l3_solar/nodes"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        beside,
        ["sw_ap_design.vnode"],
        "the file beside it is gone once renamed over the old one"
    );
    assert!(
        !c.with_file_name(format!(
            ".{}.lock",
            c.file_name().unwrap().to_string_lossy()
        ))
        .exists(),
        "the copy's lock is let go"
    );
    assert_eq!(
        now.meta.get("written_by_app").map(String::as_str),
        Some(APP),
        "the program that saved it says so, not the edit"
    );
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_second_writer_is_stopped_and_nothing_is_written() {
    let (c, g) = copy("second");
    let (f, opened) = open(&c);
    // Someone else saves first: Test Dee, for Nia, on the same copy.
    let (theirs, _) = open(&c);
    match save(&c, NODE, &edited(&theirs), &opened, "Test Dee", APP, &g).unwrap() {
        Saved::Written { revision: 2, .. } => {}
        other => panic!("the first save did not write: {other:?}"),
    }
    let before = std::fs::read(c.join(NODE)).unwrap();
    match save(&c, NODE, &edited(&f), &opened, "Test Nia", APP, &g).unwrap() {
        Saved::SecondWriter {
            writer,
            revision,
            now,
            bytes,
            digest: d,
        } => {
            assert_eq!((writer.as_str(), revision), ("Test Dee", 2));
            assert_eq!(bytes, before, "theirs, as the bytes compared");
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
        save(&c, NODE, &edited(&f), &opened, "Test Nia", APP, &g).unwrap(),
        Saved::Written { revision: 2, .. }
    ));
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn only_its_assigned_writer_or_their_deputy_writes_it() {
    let (c, g) = copy("who");
    let (f, opened) = open(&c);
    let before = std::fs::read(c.join(NODE)).unwrap();
    for who in ["Test Ivo", "Test Zed"] {
        let e = save(&c, NODE, &edited(&f), &opened, who, APP, &g).unwrap_err();
        assert!(
            e.message()
                .contains(&format!("{who} is not assigned sw_ap_design")),
            "{}",
            e.message()
        );
    }
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    assert!(matches!(
        save(&c, NODE, &edited(&f), &opened, "Test Dee", APP, &g).unwrap(),
        Saved::Written { .. }
    ));
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_save_does_not_move_a_node_or_reach_outside_the_copy() {
    let (c, g) = copy("moved");
    let (f, opened) = open(&c);
    let before = std::fs::read(c.join(NODE)).unwrap();
    for (key, value) in [
        ("block_uid", "sw_ap_other"),
        ("group_id", "l3_power"),
        ("file_kind", "group"),
    ] {
        let mut moved = edited(&f);
        moved.meta.insert(s(key), s(value));
        let e = save(&c, NODE, &moved, &opened, "Test Nia", APP, &g).unwrap_err();
        assert!(
            e.message().contains(&format!("is {key}"))
                && e.message().contains("a save does not move a node"),
            "{key}: {}",
            e.message()
        );
    }
    for (what, change) in [
        (
            "id",
            (|b: &mut Block| b.id = s("sw_ap_other")) as fn(&mut Block),
        ),
        ("parent", |b: &mut Block| b.parent_uid = s("l3_power")),
        ("contract", |b: &mut Block| b.contract_version += 1),
    ] {
        let mut moved = edited(&f);
        change(&mut moved.blocks[0]);
        let e = save(&c, NODE, &moved, &opened, "Test Nia", APP, &g).unwrap_err();
        assert!(
            e.message().contains("a save does not move a node"),
            "{what}: {}",
            e.message()
        );
    }
    for path in ["../elsewhere.vnode", "/etc/passwd", "groups/../x.vnode"] {
        let e = save(&c, path, &edited(&f), &opened, "Test Nia", APP, &g).unwrap_err();
        assert!(
            e.message().contains("not a path inside the copy"),
            "{path}: {}",
            e.message()
        );
    }
    let mut other = g.clone();
    other.file.meta.insert(s("group_id"), s("l3_power"));
    let e = save(&c, NODE, &edited(&f), &opened, "Test Nia", APP, &other).unwrap_err();
    assert!(
        e.message().contains("the group file given is l3_power's"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_save_changes_the_node_s_own_rows_and_nothing_else_in_its_file() {
    let (c, g) = copy("outside");
    let (f, opened) = open(&c);
    let before = std::fs::read(c.join(NODE)).unwrap();
    let signature = SignatureRow {
        scope: s("sw_ap_design"),
        revision: s("2"),
        signer: s("Test Nia"),
        public_key: s("00"),
        digest: s("00"),
        signature: s("00"),
        signed_at: s("2026-10-15T12:00:00Z"),
        verdict: s("ok"),
        note: s(""),
    };
    type Edit = Box<dyn Fn(&mut File)>;
    let edits: Vec<(&str, Edit)> = vec![
        (
            "signature",
            Box::new(move |e| e.signatures.push(signature.clone())),
        ),
        (
            "block",
            Box::new(|e| {
                let mut other = e.blocks[0].clone();
                other.uid = s("sw_other");
                other.id = s("sw_other");
                e.blocks.push(other);
            }),
        ),
        (
            "mount",
            Box::new(|e| {
                e.mounts.push(Mount {
                    block_uid: s("sw_ap_design"),
                    group_id: s("l3_power"),
                    release: s(""),
                })
            }),
        ),
        (
            "person",
            Box::new(|e| {
                e.people.push(Person {
                    name: s("Test Mal"),
                    role: s("node engineer"),
                    deputy_for: s(""),
                })
            }),
        ),
        (
            "port",
            Box::new(|e| {
                let mut p = e.ports[0].clone();
                p.block_uid = s("sw_other");
                e.ports.push(p);
            }),
        ),
        (
            "meta",
            Box::new(|e| {
                e.meta.insert(s("based_on"), s("elsewhere"));
            }),
        ),
    ];
    for (what, change) in edits {
        let mut e = edited(&f);
        change(&mut e);
        let refused = save(&c, NODE, &e, &opened, "Test Nia", APP, &g).unwrap_err();
        assert!(
            refused
                .message()
                .contains("not sw_ap_design's own to change"),
            "{what}: {}",
            refused.message()
        );
    }
    let mut gone = edited(&f);
    gone.blocks.clear();
    let e = save(&c, NODE, &gone, &opened, "Test Nia", APP, &g).unwrap_err();
    assert!(e.message().contains("does not hold"), "{}", e.message());
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_revision_that_is_not_a_count_is_not_saved_over_and_a_revision_always_rises() {
    let (c, g) = copy("revision");
    let (mut f, _) = open(&c);
    f.meta.insert(s("revision"), s("two"));
    sqlite::write(&f, &c.join(NODE)).unwrap();
    let (f, opened) = open(&c);
    let e = save(&c, NODE, &edited(&f), &opened, "Test Nia", APP, &g).unwrap_err();
    assert!(e.message().contains("not a count"), "{}", e.message());
    // The block is ahead of the file: the next revision is past both.
    let mut f = f;
    f.meta.insert(s("revision"), s("1"));
    f.blocks[0].revision = 4;
    sqlite::write(&f, &c.join(NODE)).unwrap();
    let (f, opened) = open(&c);
    assert!(matches!(
        save(&c, NODE, &edited(&f), &opened, "Test Nia", APP, &g).unwrap(),
        Saved::Written { revision: 5, .. }
    ));
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn an_opened_that_is_not_a_digest_is_refused_as_that_and_not_as_a_writer() {
    let (c, g) = copy("opened");
    let (f, _) = open(&c);
    for opened in ["", "abc", &"G".repeat(64)] {
        let e = save(&c, NODE, &edited(&f), opened, "Test Nia", APP, &g).unwrap_err();
        assert!(
            e.message()
                .contains("is not the SHA-256 of the node as it was opened"),
            "{opened:?}: {}",
            e.message()
        );
    }
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_node_file_is_kept_in_its_group_s_folder_under_its_id() {
    let (c, g) = copy("place");
    let planted = "groups/l3_other/nodes/sw_ap_design.vnode";
    std::fs::create_dir_all(c.join("groups/l3_other/nodes")).unwrap();
    std::fs::copy(c.join(NODE), c.join(planted)).unwrap();
    let bytes = std::fs::read(c.join(planted)).unwrap();
    let f = sqlite::read(&c.join(planted)).unwrap();
    let e = save(
        &c,
        planted,
        &edited(&f),
        &digest(&bytes),
        "Test Nia",
        APP,
        &g,
    )
    .unwrap_err();
    assert!(
        e.message()
            .contains("is not where the node it holds is kept"),
        "{}",
        e.message()
    );
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn nobody_writes_a_block_assigned_to_nobody() {
    let (c, mut g) = copy("nobody");
    g.file.assignments[0].person = s("");
    let (f, opened) = open(&c);
    for who in ["Test Nia", "Test Ivo"] {
        let e = save(&c, NODE, &edited(&f), &opened, who, APP, &g).unwrap_err();
        assert!(
            e.message().contains("is not assigned"),
            "{who}: {}",
            e.message()
        );
    }
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_copy_another_program_is_writing_is_not_saved_to() {
    let (c, g) = copy("locked");
    let (f, opened) = open(&c);
    let held = vleo_files::save::lock(&c).unwrap();
    let e = save(&c, NODE, &edited(&f), &opened, "Test Nia", APP, &g).unwrap_err();
    assert!(
        e.message()
            .contains("another program is copying the database or writing your copy"),
        "{}",
        e.message()
    );
    drop(held);
    assert!(matches!(
        save(&c, NODE, &edited(&f), &opened, "Test Nia", APP, &g).unwrap(),
        Saved::Written { .. }
    ));
    let _ = std::fs::remove_dir_all(&c);
}
