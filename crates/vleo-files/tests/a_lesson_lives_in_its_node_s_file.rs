//! A row's lesson has its place in the row's node file: a `text` of kind
//! `lesson` on its block, served to the loader as the `lesson.toml` its folder
//! held, so everything that reads a lesson reads it as it did; checked with the
//! node, and signed with it (docs/briefs/F3.md, F3-e; docs/LESSONS.md).
//!
//! The lesson is the example the documentation keeps
//! (`docs/examples/orbit_velocity.lesson.toml`), placed in the design's own
//! `orbit_velocity` node file, in memory: nothing in `design/` is written.

use std::path::{Path, PathBuf};
use vleo_files::chain;
use vleo_files::checks;
use vleo_files::convert::{self, Served, LESSON};
use vleo_files::model::*;
use vleo_sheet::files::Files;

const NODE: &str = "groups/l3_x_envorbit/nodes/orbit_velocity.vnode";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn example() -> String {
    std::fs::read_to_string(root().join("docs/examples/orbit_velocity.lesson.toml")).unwrap()
}

/// The design's files, with `lesson` placed in orbit_velocity's node file.
fn with_lesson(lesson: Option<&str>) -> (Vec<(String, File)>, String) {
    let (mut files, _) = convert::read_folder(&root().join("design")).unwrap();
    let (_, f) = files
        .iter_mut()
        .find(|(p, _)| p.ends_with(NODE) || p == NODE)
        .expect("orbit_velocity's node file");
    let uid = f.blocks[0].uid.clone();
    f.texts.retain(|t| !(t.scope == uid && t.kind == LESSON));
    if let Some(l) = lesson {
        f.texts.push(Text {
            scope: uid.clone(),
            kind: LESSON.to_string(),
            body: l.to_string(),
        });
    }
    (files, uid)
}

fn node_file(files: &[(String, File)]) -> &File {
    &files
        .iter()
        .find(|(p, _)| p.ends_with(NODE) || p == NODE)
        .unwrap()
        .1
}

#[test]
fn a_node_file_s_lesson_is_the_lesson_the_design_reads() {
    // Today no node file holds one, and the design reads none.
    let (files, _) = with_lesson(None);
    let tree =
        vleo_sheet::load::load_all_from(&Served::new(&root(), &files).unwrap(), &root()).unwrap();
    assert!(tree.sheets.values().all(|sh| sh.lesson.is_none()));

    let (files, uid) = with_lesson(Some(&example()));
    let served = Served::new(&root(), &files).unwrap();
    let sh_dir = {
        let tree = vleo_sheet::load::load_all_from(&served, &root()).unwrap();
        let sh = &tree.sheets["orbit_velocity"];
        // Read by the loader as it read one beside node.toml, word for word.
        assert_eq!(sh.lesson.as_deref(), Some(example().as_str()));
        let l = vleo_sheet::lesson::of(sh).unwrap().unwrap();
        assert_eq!(l.title, "How fast a very low orbit goes");
        // And it holds the gate's check, rows and all, in the design.
        assert_eq!(
            vleo_sheet::lesson::problems(&l, &tree),
            Vec::<String>::new()
        );
        sh.dir.clone()
    };
    assert!(sh_dir.ends_with(&uid));
    assert_eq!(
        served.read(&sh_dir.join(vleo_sheet::lesson::FILE)).unwrap(),
        example().into_bytes()
    );
    // No other row is given it.
    let tree = vleo_sheet::load::load_all_from(&served, &root()).unwrap();
    assert_eq!(
        tree.sheets
            .values()
            .filter(|sh| sh.lesson.is_some())
            .count(),
        1
    );
}

#[test]
fn a_lesson_is_checked_with_its_node() {
    let lesson_findings = |l: &str| -> Vec<String> {
        let (files, _) = with_lesson(Some(l));
        checks::check_release(node_file(&files))
            .errors()
            .filter(|e| e.what.contains("lesson"))
            .map(|e| e.what.clone())
            .collect()
    };
    assert_eq!(lesson_findings(&example()), Vec::<String>::new());
    for (what, broken, says) in [
        (
            "a misspelt key",
            example().replacen(
                "claim = \"derived\"",
                "claim = \"derived\"\nsourse = \"x\"",
                1,
            ),
            "does not read",
        ),
        (
            "markup",
            example().replacen(
                "How fast the spacecraft",
                "<b>How</b> fast the spacecraft",
                1,
            ),
            "is markup",
        ),
        (
            "an untagged claim",
            example().replacen("claim = \"derived\"", "claim = \"obvious\"", 1),
            "claim 'obvious' is not one of",
        ),
        ("not TOML", "[lesson\n".to_string(), "does not read"),
    ] {
        let found = lesson_findings(&broken);
        assert!(found.iter().any(|f| f.contains(says)), "{what}: {found:?}");
    }
    // The node's own checks, the ones a signing is held to, refuse it too.
    let (files, _) = with_lesson(Some("[lesson\n"));
    assert!(checks::check_node(node_file(&files))
        .errors()
        .any(|e| e.what.contains("its lesson does not read")));
}

#[test]
fn a_lesson_is_signed_with_its_node() {
    let (files, uid) = with_lesson(Some(&example()));
    let (changed, _) = with_lesson(Some(&example().replacen("very low", "very, very low", 1)));
    let (none, _) = with_lesson(None);
    let d = |files: &[(String, File)]| chain::digest(node_file(files), &uid).unwrap();
    assert_ne!(d(&files), d(&changed), "a changed lesson is a changed node");
    assert_ne!(d(&files), d(&none), "and so is one taken out");
}
