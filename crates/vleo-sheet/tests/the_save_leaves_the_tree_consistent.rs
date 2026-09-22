//! The save transaction, against the real tree.
//!
//! What matters here is not that a good edit works — it is that a bad one
//! leaves NOTHING changed. A browser that half-edits a repository is worse than
//! one that cannot edit it at all, so every failing path is checked for the
//! sheet being exactly as it was.

use std::path::{Path, PathBuf};
use vleo_sheet::form::{self, Saved};

fn root() -> PathBuf {
    // tests run with CWD at the crate root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// A published row with a plain single-line field to move.
const ROW: &str = "gnc_along_track_error";

fn sheet_path(root: &Path) -> PathBuf {
    root.join("crates/vleo-mod-acs/nodes")
        .join(ROW)
        .join("node.toml")
}

/// The hash a save must send back — of the file's bytes, not of the meaning.
fn hash_now(root: &Path) -> String {
    form::file_hash(&std::fs::read_to_string(sheet_path(root)).unwrap())
}

#[test]
fn a_stale_base_is_refused_and_nothing_is_written() {
    let root = root();
    let before = std::fs::read_to_string(sheet_path(&root)).unwrap();
    match form::save(&root, ROW, "unit", "Kelvin", "000000", "A. Person") {
        Saved::Stale { current } => assert_eq!(current, hash_now(&root)),
        Saved::Ok { .. } => panic!("a stale base must never be written"),
        Saved::Refused(e) => panic!("expected Stale, got refusal: {e}"),
    }
    assert_eq!(
        std::fs::read_to_string(sheet_path(&root)).unwrap(),
        before,
        "the sheet must be untouched"
    );
}

#[test]
fn a_structural_field_is_refused_and_nothing_is_written() {
    let root = root();
    let before = std::fs::read_to_string(sheet_path(&root)).unwrap();
    let h = hash_now(&root);
    for field in ["order", "parent", "kind", "owner"] {
        match form::save(&root, ROW, field, "99", &h, "A. Person") {
            Saved::Refused(e) => assert!(
                e.contains("not editable here"),
                "{field} should say why: {e}"
            ),
            _ => panic!("{field} must be refused"),
        }
    }
    assert_eq!(std::fs::read_to_string(sheet_path(&root)).unwrap(), before);
}

#[test]
fn an_agent_may_not_supply_the_relation_through_this_face_either() {
    let root = root();
    let before = std::fs::read_to_string(sheet_path(&root)).unwrap();
    let h = hash_now(&root);
    for who in ["Claude", "claude opus 5", "hole-filler", "Claude/Opus"] {
        match form::save(&root, ROW, "expression", "e = 2*x", &h, who) {
            Saved::Refused(e) => assert!(
                e.contains("agent") || e.contains("person"),
                "{who}: {e}"
            ),
            _ => panic!("'{who}' must not be able to supply mathematics"),
        }
    }
    // And a blank attribution is not a way round it.
    match form::save(&root, ROW, "expression", "e = 2*x", &h, "   ") {
        Saved::Refused(e) => assert!(e.contains("blank") || e.contains("cannot be blank"), "{e}"),
        _ => panic!("a blank attribution must be refused"),
    }
    assert_eq!(std::fs::read_to_string(sheet_path(&root)).unwrap(), before);
}

#[test]
fn an_unknown_node_is_refused() {
    let root = root();
    match form::save(&root, "no_such_row", "unit", "Kelvin", "abc123", "A. Person") {
        Saved::Refused(e) => assert!(e.contains("no node"), "{e}"),
        _ => panic!("must be refused"),
    }
}

/// Puts a sheet's bytes back when this goes out of scope, panic or not.
///
/// The first version of this test restored at the end, on the success path. It
/// then failed in the middle and left a rewritten `reason_lower` committed to
/// nobody's intention in the working tree — exactly the state the code under
/// test exists to prevent. A guard restores on the way out either way.
struct Restore {
    path: PathBuf,
    bytes: String,
}

impl Drop for Restore {
    fn drop(&mut self) {
        let _ = std::fs::write(&self.path, &self.bytes);
        // The artefacts too: a restored sheet beside artefacts generated from
        // the edit is the half-state this whole path avoids.
        if let Ok(t) = vleo_sheet::load::load_all(
            self.path.parent().unwrap().parent().unwrap().parent().unwrap()
                .parent().unwrap().parent().unwrap(),
        ) {
            if let Some(sh) = t.sheets.values().find(|s| s.dir == self.path.parent().unwrap()) {
                let _ = form::regenerate_for_test(sh, &t);
            }
        }
    }
}

/// A different row from the refusal tests, so the two cannot race: the harness
/// runs tests in parallel and this one writes.
const EDIT_ROW: &str = "gnc_alignment_error";

fn edit_path(root: &Path) -> PathBuf {
    root.join("crates/vleo-mod-acs/nodes").join(EDIT_ROW).join("node.toml")
}

#[test]
fn a_good_edit_is_written_regenerated_and_then_put_back() {
    let root = root();
    let path = edit_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let _guard = Restore { path: path.clone(), bytes: before.clone() };
    let h0 = form::file_hash(&before);
    let meaning0 = {
        let t = vleo_sheet::load::load_all(&root).unwrap();
        vleo_sheet::short_hex(t.sheets.get(EDIT_ROW).unwrap().sheet_hash)
    };

    // A real change to a real field, with a real person against it.
    let out = form::save(&root, EDIT_ROW, "reason_lower", "a rewritten reason", &h0, "A. Person");
    let (h1, meaning1, n) = match out {
        Saved::Ok { file_hash, sheet_hash, regenerated } => (file_hash, sheet_hash, regenerated),
        Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
        Saved::Refused(e) => panic!("a good edit was refused: {e}"),
    };
    assert_ne!(h1, h0, "the file hash must move when the file moves");
    // And the point of having two: a bound's REASON is not part of the node's
    // meaning, so the sheet hash is expected to sit still here. If it moved,
    // every artefact downstream would be invalidated by a reworded sentence.
    assert_eq!(
        meaning1, meaning0,
        "rewording a bound's reason must not change the node's meaning"
    );
    assert!(n >= 1, "at least the page and the metadata carry the reason");
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("a rewritten reason"));
    assert_eq!(
        after.lines().filter(|l| l.trim_start().starts_with('#')).count(),
        before.lines().filter(|l| l.trim_start().starts_with('#')).count(),
        "no comment may be lost"
    );

    // Put the row back exactly as it was, and regenerate, so this test leaves
    // no trace. A test that edits the repository and does not undo it is a test
    // that breaks the next one.
    let h1b = form::file_hash(&std::fs::read_to_string(&path).unwrap());
    let original = {
        let t: toml::Value = before.parse().unwrap();
        t["output"]["reason_lower"].as_str().unwrap().to_string()
    };
    match form::save(&root, EDIT_ROW, "reason_lower", &original, &h1b, "A. Person") {
        Saved::Ok { .. } => {}
        Saved::Stale { current } => panic!("could not restore, current {current}"),
        Saved::Refused(e) => panic!("could not restore: {e}"),
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "the sheet must be byte-identical to how this test found it"
    );
}
