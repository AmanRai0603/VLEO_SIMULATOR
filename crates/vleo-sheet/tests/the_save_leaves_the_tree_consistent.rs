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
    match form::save(&root, ROW, "unit", "Kelvin", "000000") {
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
    let _guard = Restore {
        path: sheet_path(&root),
        bytes: before.clone(),
    };
    let h = hash_now(&root);
    for field in ["order", "parent", "kind", "owner"] {
        match form::save(&root, ROW, field, "99", &h) {
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
fn an_agent_is_refused_whatever_the_face() {
    // The rule itself, as a function: no git, no files, no checkout. Every name
    // on the roster and the two generic words, in the shapes a name arrives in.
    let root = root();
    for who in [
        "Claude",
        "claude opus 5",
        "hole-filler",
        "Claude/Opus",
        "AGENT",
        "  claude  ",
    ] {
        assert!(
            form::refuse_agent_attribution(&root, who).is_err(),
            "'{who}' must never be able to supply mathematics"
        );
    }
    for who in ["", "   "] {
        let e = form::refuse_agent_attribution(&root, who).unwrap_err();
        assert!(
            e.contains("blank"),
            "a blank attribution is not a way round it: {e}"
        );
    }
    // And a person's name is not refused, or the rule would block the work it
    // exists to make possible.
    assert!(form::refuse_agent_attribution(&root, "A. Rai").is_ok());
}

#[test]
fn the_relation_is_attributed_to_the_checkout_not_to_a_typed_name() {
    // `save` takes no name. It asks the checkout, so the sheet and the commit
    // make the same claim about who did the work.
    let root = root();
    let before = std::fs::read_to_string(sheet_path(&root)).unwrap();
    let _guard = Restore {
        path: sheet_path(&root),
        bytes: before.clone(),
    };
    let h = hash_now(&root);
    let who = form::git_identity(&root);
    match form::save(&root, ROW, "expression", "e = 2*x", &h) {
        Saved::Refused(e) => {
            // Which is what must happen wherever the checkout's own identity is
            // an agent's — as it is in the container this was written in, where
            // `git config user.name` is "Claude".
            assert!(
                e.contains("agent") || e.contains("user.name"),
                "a refusal here must be about the identity: {e}"
            );
            assert_eq!(
                std::fs::read_to_string(sheet_path(&root)).unwrap(),
                before,
                "and nothing may be written"
            );
        }
        Saved::Ok { .. } => {
            // A human checkout. Then the name it used must be that checkout's.
            let w = who.expect("a save that succeeded must have had an identity");
            assert!(
                form::refuse_agent_attribution(&root, &w).is_ok(),
                "it wrote under {w}, which the roster calls an agent"
            );
            // Put it back; this test is not here to edit the tree.
            let h2 = hash_now(&root);
            let orig = {
                let t: toml::Value = before.parse().unwrap();
                t["maths"]["expression"].as_str().unwrap().to_string()
            };
            let _ = form::save(&root, ROW, "expression", &orig, &h2);
            assert_eq!(std::fs::read_to_string(sheet_path(&root)).unwrap(), before);
        }
        Saved::Stale { .. } => panic!("unexpectedly stale"),
    }
}

#[test]
fn an_unknown_node_is_refused() {
    let root = root();
    match form::save(&root, "no_such_row", "unit", "Kelvin", "abc123") {
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
            self.path
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap(),
        ) {
            if let Some(sh) = t
                .sheets
                .values()
                .find(|s| s.dir == self.path.parent().unwrap())
            {
                let _ = form::regenerate_for_test(sh, &t);
            }
        }
    }
}

/// A different row from the refusal tests, so the two cannot race: the harness
/// runs tests in parallel and this one writes.
const EDIT_ROW: &str = "gnc_alignment_error";

fn edit_path(root: &Path) -> PathBuf {
    root.join("crates/vleo-mod-acs/nodes")
        .join(EDIT_ROW)
        .join("node.toml")
}

#[test]
fn a_good_edit_is_written_regenerated_and_then_put_back() {
    let root = root();
    let path = edit_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let _guard = Restore {
        path: path.clone(),
        bytes: before.clone(),
    };
    let h0 = form::file_hash(&before);
    let meaning0 = {
        let t = vleo_sheet::load::load_all(&root).unwrap();
        vleo_sheet::short_hex(t.sheets.get(EDIT_ROW).unwrap().sheet_hash)
    };

    // A real change to a real field, with a real person against it.
    let out = form::save(&root, EDIT_ROW, "reason_lower", "a rewritten reason", &h0);
    let (h1, meaning1, n) = match out {
        Saved::Ok {
            file_hash,
            sheet_hash,
            regenerated,
        } => (file_hash, sheet_hash, regenerated),
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
    assert!(
        n >= 1,
        "at least the page and the metadata carry the reason"
    );
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("a rewritten reason"));
    assert_eq!(
        after
            .lines()
            .filter(|l| l.trim_start().starts_with('#'))
            .count(),
        before
            .lines()
            .filter(|l| l.trim_start().starts_with('#'))
            .count(),
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
    match form::save(&root, EDIT_ROW, "reason_lower", &original, &h1b) {
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

// ── putting the edits on a branch ────────────────────────────────────────────

#[test]
fn a_proposal_with_nothing_edited_does_nothing() {
    // And says so, rather than making an empty branch. The checkout must be
    // exactly where it was afterwards — this test runs on whatever branch the
    // developer is on, so leaving them somewhere else would be the worst thing
    // it could do.
    let root = root();
    let was = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap();
    let was = String::from_utf8_lossy(&was.stdout).trim().to_string();
    // Only meaningful when no node folder is dirty; if one is, this says so
    // rather than pretending to have tested something.
    let dirty = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["status", "--porcelain", "--", "crates"])
        .output()
        .unwrap();
    let dirty: Vec<String> = String::from_utf8_lossy(&dirty.stdout)
        .lines()
        .filter(|l| l.contains("/nodes/"))
        .map(|s| s.to_string())
        .collect();
    if !dirty.is_empty() {
        eprintln!(
            "skipped: a node folder is already edited ({} path(s))",
            dirty.len()
        );
        return;
    }
    // Nothing edited, so nothing to propose — and this must hold on a machine
    // with no `git config user.name` too, which is what a fresh CI runner is.
    // The first version asked for the identity before looking at what was
    // dirty, and failed there for a reason that had nothing to do with the
    // case it was testing.
    match form::propose(&root, "anything", "docs") {
        form::Proposed::Nothing => {}
        form::Proposed::Ok { branch, .. } => panic!("it made a branch: {branch}"),
        form::Proposed::Refused(e) => panic!("expected Nothing, got: {e}"),
    }
    let now = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&now.stdout).trim(),
        was,
        "a proposal that did nothing must leave the checkout on its own branch"
    );
}

// ── what a face must not be able to write ────────────────────────────────────

#[test]
fn a_type_that_is_not_a_quantity_is_refused_before_anything_is_written() {
    // THE ONE THAT BROKE THE BUILD. `[output] type` is emitted verbatim into the
    // generated signature, so `Nonsense` becomes `Result<Nonsense, Fault>` and
    // the tree stops compiling. The gate passed it — the gate never compiles
    // anything — so the save reported success on a repository it had broken.
    let root = root();
    let before = std::fs::read_to_string(sheet_path(&root)).unwrap();
    // THE GUARD IS NOT OPTIONAL ON A TEST THAT CALLS `save`. A mutation run is
    // exactly the case where the refusal does not happen, the write goes
    // through and the assertion below fails — and without this the broken value
    // stays in the tree. It did: disabling the type check left `Nonsense` in a
    // sheet and the workspace would not compile until it was noticed.
    let _guard = Restore {
        path: sheet_path(&root),
        bytes: before.clone(),
    };
    let h = hash_now(&root);
    for bad in ["Nonsense", "length", "Furlong", "", "Result<Length, Fault>"] {
        match form::save(&root, ROW, "type", bad, &h) {
            Saved::Refused(e) => assert!(
                e.contains("quantity") || e.contains("not a field"),
                "{bad:?} should be refused as a type: {e}"
            ),
            _ => panic!("{bad:?} must not be writable as a type"),
        }
    }
    assert_eq!(
        std::fs::read_to_string(sheet_path(&root)).unwrap(),
        before,
        "nothing may be written"
    );
    // And a real one is not refused, or the check blocks the work it exists for.
    assert!(form::value_allowed("type", "Length").is_ok());
    assert!(form::value_allowed("type", "Ratio").is_ok());
}

#[test]
fn a_unit_the_system_does_not_know_is_refused() {
    // It does not reach the signature, so it does not break the build — it
    // makes the row claim an answer in a unit nothing can convert at a face
    // boundary. The gate only ever asked whether the field was blank.
    let root = root();
    let _guard = Restore {
        path: sheet_path(&root),
        bytes: std::fs::read_to_string(sheet_path(&root)).unwrap(),
    };
    let h = hash_now(&root);
    for bad in ["Furlong", "metre", "m", "Fortnight"] {
        match form::save(&root, ROW, "unit", bad, &h) {
            Saved::Refused(e) => assert!(e.contains("unit"), "{bad:?}: {e}"),
            _ => panic!("{bad:?} must not be writable as a unit"),
        }
    }
    assert!(form::value_allowed("unit", "Metre").is_ok());
    assert!(form::value_allowed("unit", "Kelvin").is_ok());
}

#[test]
fn saving_the_relation_puts_a_name_against_it() {
    // The identity was verified and then went nowhere, so a relation saved
    // through the face came out with `confirmed_by` blank — which is what the
    // field exists to make impossible.
    let root = root();
    let path = edit_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let _guard = Restore {
        path: path.clone(),
        bytes: before.clone(),
    };

    let who = match form::git_identity(&root) {
        Ok(w) => w,
        Err(_) => {
            eprintln!("skipped: this checkout has no git identity");
            return;
        }
    };
    if form::refuse_agent_attribution(&root, &who).is_err() {
        // This container's identity IS an agent, which the save refuses before
        // it can stamp anything. That path has its own test; this one needs a
        // person, so it says why it did not run rather than passing quietly.
        eprintln!("skipped: this checkout's identity ({who}) is an agent");
        return;
    }
    let h = form::file_hash(&before);
    let original = {
        let t: toml::Value = before.parse().unwrap();
        t["maths"]["expression"].as_str().unwrap().to_string()
    };
    match form::save(&root, EDIT_ROW, "expression", &format!("{original} "), &h) {
        Saved::Ok { .. } => {}
        Saved::Refused(e) => panic!("a person could not save a relation: {e}"),
        Saved::Stale { .. } => panic!("unexpectedly stale"),
    }
    let after: toml::Value = std::fs::read_to_string(&path).unwrap().parse().unwrap();
    let got = after["maths"]["confirmed_by"].as_str().unwrap_or("");
    assert!(
        got.starts_with(&who),
        "the relation must carry the name of whoever supplied it; got {got:?}"
    );
}
