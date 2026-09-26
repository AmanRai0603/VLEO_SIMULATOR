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

/// EVERY TEST IN THIS FILE EDITS THE REAL TREE, so they run one at a time.
///
/// Cargo runs the tests in one binary on several threads. Two of these reading a
/// sheet while a third was mid-edit failed the suite about one run in three, on
/// an assertion about a sheet the failing test had not touched — the worst kind
/// of red, because it points at the wrong test.
///
/// The lock is held for the whole of each test rather than around each write:
/// what races is not the write but the read-edit-compare around it, and
/// `a_proposal_with_nothing_edited_does_nothing` asks whether the WHOLE checkout
/// is clean, which no other test may be inside.
fn serially() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    // A panicking test poisons the mutex, and a poisoned lock would turn one
    // real failure into ten reported as lock errors. The guard is what is
    // wanted; the poison says only that an earlier test failed, which is
    // already being reported.
    LOCK.lock().unwrap_or_else(|p| p.into_inner())
}

/// The hash a save must send back — of the file's bytes, not of the meaning.
fn hash_now(root: &Path) -> String {
    form::file_hash(&std::fs::read_to_string(sheet_path(root)).unwrap())
}

#[test]
fn a_stale_base_is_refused_and_nothing_is_written() {
    let _serial = serially();
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
    let _serial = serially();
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
    let _serial = serially();
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
    let _serial = serially();
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
            // A human checkout. Then the name it used must be that checkout's,
            // and it must be ON the sheet — not merely checked and discarded.
            let w = who.expect("a save that succeeded must have had an identity");
            assert!(
                form::refuse_agent_attribution(&root, &w).is_ok(),
                "it wrote under {w}, which the roster calls an agent"
            );
            let after = std::fs::read_to_string(sheet_path(&root)).unwrap();
            let v: toml::Value = after.parse().unwrap();
            let stamped = v["maths"]["confirmed_by"].as_str().unwrap_or_default();
            assert!(
                stamped.starts_with(&w),
                "the relation must carry {w}, not {stamped:?}"
            );
            // THE GUARD PUTS IT BACK, not a second save. This branch used to
            // restore by saving the old expression again, and never ran here —
            // `git config user.name` was an agent's, so every run took the
            // refusal above. Under a human name it fails, and correctly: a
            // relation save also stamps `confirmed_by`, so saving the old
            // expression back leaves an attribution line the sheet did not have
            // and the file never returns byte for byte. Only the bytes can
            // restore the bytes.
        }
        Saved::Stale { .. } => panic!("unexpectedly stale"),
    }
}

#[test]
fn an_unknown_node_is_refused() {
    let _serial = serially();
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
    let _serial = serially();
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
    let _serial = serially();
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
    let _serial = serially();
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
    let _serial = serially();
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
    let _serial = serially();
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

#[test]
fn each_new_shape_survives_the_whole_transaction() {
    // The form covers more than the nine that block generation, and each of the
    // extra shapes is written differently: a bound goes in bare, a note has to
    // be ADDED to a table that has not got the key, and the derivation has to
    // create the table itself. Every one of those is a way to write a sheet that
    // no longer loads, so each is driven through the real save — write,
    // regenerate, gate — rather than through `set` alone.
    let _serial = serially();
    let root = root();
    let path = edit_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let _guard = Restore {
        path: path.clone(),
        bytes: before.clone(),
    };
    assert!(
        !before.contains("[theory]") && !before.contains("\nnote ="),
        "this test needs a row that has neither, or it proves nothing about adding them"
    );

    for (field, value, expect) in [
        // Bare, and normalised to carry its decimal point.
        ("lower", "2", "lower = 2.0"),
        // Added under [question], which has not got the key.
        (
            "note",
            "what this row is not for",
            "note = \"what this row is not for\"",
        ),
        // Added along with the table it lives in.
        (
            "theory_why",
            "because the geometry admits no other reading",
            "why = \"because the geometry admits no other reading\"",
        ),
    ] {
        let h = form::file_hash(&std::fs::read_to_string(&path).unwrap());
        match form::save(&root, EDIT_ROW, field, value, &h) {
            Saved::Ok { .. } => {}
            Saved::Stale { current } => panic!("{field}: unexpectedly stale, current {current}"),
            Saved::Refused(e) => panic!("{field} was refused: {e}"),
        }
        let after = std::fs::read_to_string(&path).unwrap();
        assert!(
            after.contains(expect),
            "{field}: expected `{expect}` in the sheet, got:\n{after}"
        );
        // AND IT STILL LOADS AS THE VALUE THAT WAS SENT. A bound written as a
        // quoted string parses and the loader reads it back as zero, so "the
        // file changed" is not the assertion that matters.
        let t = vleo_sheet::load::load_all(&root).unwrap();
        let sh = t.sheets.get(EDIT_ROW).unwrap();
        assert_eq!(
            form::value(sh, field).trim(),
            form::normalise(field, value).unwrap().trim(),
            "{field} did not read back as what was written"
        );
    }
}

#[test]
fn a_bound_that_crosses_its_partner_is_refused_and_rolled_back() {
    // The one that the gate catches rather than the form: a lower bound above
    // the upper is a domain the generated guard can never satisfy, and it is
    // only wrong in relation to another field. So the sheet IS written, the
    // artefacts ARE regenerated, and then the whole thing has to come back —
    // which is the path with the most to go wrong in it.
    let _serial = serially();
    let root = root();
    let path = edit_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let _guard = Restore {
        path: path.clone(),
        bytes: before.clone(),
    };
    let h = form::file_hash(&before);
    match form::save(&root, EDIT_ROW, "lower", "9999999", &h) {
        Saved::Refused(e) => {
            assert!(
                e.contains("not below the upper"),
                "the refusal must name what is wrong: {e}"
            );
            assert!(
                e.contains("restored"),
                "and say the sheet was put back: {e}"
            );
        }
        Saved::Ok { .. } => panic!("a lower bound above the upper must never be accepted"),
        Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "the sheet must be exactly as it was"
    );
}

// ── the repeated blocks, against the real tree ───────────────────────────────
//
// `the_form_writes_a_repeated_block` covers the textual half. What is here is
// the half that can only be asked where the tree and the hand-written hole
// bodies are: an edge that does not resolve, an edge declared twice, and a
// binding a filled hole uses by name.

/// A published row with one input, one numbered step, and that hole filled.
const BLOCK_ROW: &str = "sw_ap_design";

fn block_path(root: &Path) -> PathBuf {
    root.join("crates/vleo-mod-solar/nodes")
        .join(BLOCK_ROW)
        .join("node.toml")
}

#[test]
fn an_input_a_filled_hole_binds_may_not_be_renamed_or_removed() {
    // THE GATE CANNOT CATCH THIS. An input's `binding` is a parameter name in
    // the generated signature and the hole body uses it; change one without the
    // other and the row stops compiling. The gate never compiles anything, so
    // the save would report success and leave the workspace broken.
    let _serial = serially();
    let root = root();
    let path = block_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let _guard = Restore {
        path: path.clone(),
        bytes: before.clone(),
    };
    let binding = {
        let t: toml::Value = before.parse().unwrap();
        t["input"][0]["binding"].as_str().unwrap().to_string()
    };
    let body = std::fs::read_to_string(path.parent().unwrap().join("model.rs")).unwrap();
    assert!(
        body.contains(&binding),
        "this test needs a row whose hole body uses its input's binding"
    );
    let h = form::file_hash(&before);
    for (op, values) in [
        ("set", vec![("binding", "renamed".to_string())]),
        ("remove", vec![]),
    ] {
        match form::save_block(&root, BLOCK_ROW, "input", op, 0, &values, &h) {
            Saved::Refused(e) => {
                assert!(
                    e.contains("hole"),
                    "{op}: the refusal must name the hole: {e}"
                );
                assert!(e.contains(&binding), "{op}: and the binding it uses: {e}");
            }
            Saved::Ok { .. } => panic!("{op} must not be accepted under a filled hole"),
            Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
        }
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "and nothing was written"
    );
}

#[test]
fn an_edge_declared_twice_is_refused_by_name() {
    let _serial = serially();
    let root = root();
    let path = block_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let _guard = Restore {
        path: path.clone(),
        bytes: before.clone(),
    };
    let existing = {
        let t: toml::Value = before.parse().unwrap();
        t["input"][0]["var"].as_str().unwrap().to_string()
    };
    let h = form::file_hash(&before);
    match form::save_block(
        &root,
        BLOCK_ROW,
        "input",
        "add",
        0,
        &[
            ("binding", "again".to_string()),
            ("var", existing.clone()),
            ("type", "Ratio".to_string()),
        ],
        &h,
    ) {
        Saved::Refused(e) => {
            assert!(e.contains(&existing), "the refusal must name the row: {e}");
            assert!(e.contains("already reads"), "{e}");
        }
        Saved::Ok { .. } => panic!("two edges the graph cannot tell apart must be refused"),
        Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn an_edge_to_a_row_that_is_not_there_is_written_gated_and_rolled_back() {
    // The other shape of failure, and the one with the most to go wrong: the
    // sheet IS written and the artefacts ARE regenerated before anything
    // notices, so the whole edit has to come back.
    let _serial = serially();
    let root = root();
    let path = block_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let _guard = Restore {
        path: path.clone(),
        bytes: before.clone(),
    };
    let h = form::file_hash(&before);
    match form::save_block(
        &root,
        BLOCK_ROW,
        "input",
        "add",
        0,
        &[
            ("binding", "nope".to_string()),
            ("var", "no_such_row_anywhere".to_string()),
            ("type", "Ratio".to_string()),
        ],
        &h,
    ) {
        Saved::Refused(e) => {
            assert!(e.contains("names no node"), "the gate's own words: {e}");
            assert!(
                e.contains("restored"),
                "and it says the sheet came back: {e}"
            );
        }
        Saved::Ok { .. } => panic!("an edge to nothing must be refused"),
        Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "the sheet must be exactly as it was"
    );
}

#[test]
fn a_theory_step_is_added_and_removed_through_the_whole_transaction() {
    // The free case: prose that nothing generated depends on structurally, so
    // it goes in and comes out again.
    let _serial = serially();
    let root = root();
    let path = block_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let _guard = Restore {
        path: path.clone(),
        bytes: before.clone(),
    };
    let had = {
        let t: toml::Value = before.parse().unwrap();
        t["theory"]["step"].as_array().unwrap().len()
    };
    let h = form::file_hash(&before);
    match form::save_block(
        &root,
        BLOCK_ROW,
        "theory",
        "add",
        0,
        &[("text", "and one more thing about the ceiling".to_string())],
        &h,
    ) {
        Saved::Ok { .. } => {}
        Saved::Refused(e) => panic!("a theory step was refused: {e}"),
        Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
    }
    let after = std::fs::read_to_string(&path).unwrap();
    let t: toml::Value = after.parse().unwrap();
    assert_eq!(t["theory"]["step"].as_array().unwrap().len(), had + 1);
    assert_eq!(
        t["theory"]["step"][had]["text"].as_str(),
        Some("and one more thing about the ceiling")
    );

    let h2 = form::file_hash(&after);
    match form::save_block(&root, BLOCK_ROW, "theory", "remove", had, &[], &h2) {
        Saved::Ok { .. } => {}
        Saved::Refused(e) => panic!("removing it was refused: {e}"),
        Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
    }
    let t: toml::Value = std::fs::read_to_string(&path).unwrap().parse().unwrap();
    assert_eq!(t["theory"]["step"].as_array().unwrap().len(), had);
}

// ── how the answer is drawn ──────────────────────────────────────────────────

#[test]
fn a_view_is_rewritten_as_one_table_and_a_sweep_over_nothing_is_refused() {
    let _serial = serially();
    let root = root();
    let path = block_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let _guard = Restore {
        path: path.clone(),
        bytes: before.clone(),
    };
    let h = form::file_hash(&before);

    // NOTHING ELSE CHECKS THIS. The gate validates an input's `var` because an
    // input is an edge; a view's `over` is not one, so a sweep naming a row
    // that was renamed draws nothing and says nothing about why.
    match form::save_view(&root, BLOCK_ROW, "line", "no_such_row_anywhere", "60", &h) {
        Saved::Refused(e) => assert!(e.contains("no row"), "{e}"),
        _ => panic!("a sweep over a row that is not there must be refused"),
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);

    match form::save_view(&root, BLOCK_ROW, "line", BLOCK_ROW, "60", &h) {
        Saved::Refused(e) => assert!(e.contains("against itself"), "{e}"),
        _ => panic!("a row swept against itself must be refused"),
    }

    // And the real thing, both ways round.
    match form::save_view(&root, BLOCK_ROW, "line", "orbit_altitude", "60", &h) {
        Saved::Ok { .. } => {}
        Saved::Refused(e) => panic!("a real sweep was refused: {e}"),
        Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
    }
    let t: toml::Value = std::fs::read_to_string(&path).unwrap().parse().unwrap();
    assert_eq!(t["view"]["kind"].as_str(), Some("line"));
    assert_eq!(t["view"]["over"].as_str(), Some("orbit_altitude"));
    assert_eq!(t["view"]["points"].as_integer(), Some(60));

    let h2 = form::file_hash(&std::fs::read_to_string(&path).unwrap());
    match form::save_view(&root, BLOCK_ROW, "number", "", "", &h2) {
        Saved::Ok { .. } => {}
        Saved::Refused(e) => panic!("going back to a number was refused: {e}"),
        Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
    }
    let back = std::fs::read_to_string(&path).unwrap();
    let t: toml::Value = back.parse().unwrap();
    assert_eq!(t["view"]["kind"].as_str(), Some("number"));
    assert!(
        t["view"].get("over").is_none(),
        "a number is not drawn against anything, so the key goes: {back}"
    );
}

// ── publishing ───────────────────────────────────────────────────────────────

#[test]
fn publishing_a_row_that_is_not_ready_says_every_reason_and_writes_nothing() {
    let _serial = serially();
    let root = root();
    let tree = vleo_sheet::load::load_all(&root).unwrap();
    // A seeded computed row with nothing declared under it: the case the
    // preconditions exist for. Found rather than named, so this does not break
    // when somebody finishes the row it happened to pick.
    let Some(sh) = tree
        .ordered()
        .iter()
        .find(|s| s.is_seeded() && !s.is_declared() && s.inputs.is_empty())
        .map(|s| (*s).clone())
    else {
        return;
    };
    let why = form::unpublishable(&sh);
    assert!(
        why.iter().any(|w| w.contains("reads something")),
        "a computed row with no inputs is refused, and told why: {why:?}"
    );
    let path = sh.dir.join("node.toml");
    let before = std::fs::read_to_string(&path).unwrap();
    let h = form::file_hash(&before);
    match form::publish(&root, &sh.id, &h) {
        Saved::Refused(e) => assert!(e.contains("not ready to publish"), "{e}"),
        _ => panic!("{} must not publish", sh.id),
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn a_row_that_is_already_published_is_not_published_again() {
    let _serial = serially();
    let root = root();
    let path = block_path(&root);
    let before = std::fs::read_to_string(&path).unwrap();
    let h = form::file_hash(&before);
    match form::publish(&root, BLOCK_ROW, &h) {
        Saved::Refused(e) => assert!(e.contains("already"), "{e}"),
        _ => panic!("publishing moves a seeded row and nothing else"),
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

// ── putting the edits on a branch ────────────────────────────────────────────

/// A throwaway repository shaped like this one, so `propose` can be run for
/// real without committing anything here.
///
/// The bug this exists for could only be seen end to end: `git status
/// --porcelain` puts the status in two columns and an unstaged modification is
/// ` M path`, so the path starts at column 3 — and the git helper trimmed BOTH
/// ends of its output, eating the leading space of the first line only. The
/// path parsed as `rates/…`, `git add` failed, and the whole proposal was
/// refused. It survived review because it depends on what the first dirty line
/// is: an untracked file is `?? path` and works.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch_repo(name: &str) -> Option<Scratch> {
    let dir = std::env::temp_dir().join(format!("vleo-propose-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    let nodes = dir.join("crates/vleo-mod-demo/nodes/demo_row");
    std::fs::create_dir_all(&nodes).ok()?;
    let run = |args: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
    };
    run(&["init", "--quiet", "-b", "work"])?;
    run(&["config", "user.email", "nobody@example.invalid"])?;
    run(&["config", "user.name", "A. Person"])?;
    // No hooks: this repository's commit-msg hook is what validates a message,
    // and a scratch clone has not got it. What is under test is the path
    // parsing, not the hook.
    std::fs::write(
        nodes.join("node.toml"),
        "id = \"demo_row\"\nlabel = \"first\"\n",
    )
    .ok()?;
    run(&["add", "-A"])?;
    run(&["commit", "--quiet", "-m", "chore(tree): seed"])?;
    // The shape that broke it: a TRACKED file, modified and not staged, so the
    // first porcelain line begins with a space.
    std::fs::write(
        nodes.join("node.toml"),
        "id = \"demo_row\"\nlabel = \"second\"\n",
    )
    .ok()?;
    Some(Scratch(dir))
}

#[test]
fn a_modified_row_is_committed_to_a_branch_of_its_own() {
    let Some(s) = scratch_repo("modified") else {
        return; // no usable git here; the other tests still cover the rest
    };
    let root = s.0.clone();
    match form::propose(&root, "reword the label", "docs") {
        form::Proposed::Ok {
            branch,
            commit,
            files,
            ..
        } => {
            assert_eq!(files, 1, "one row was edited");
            assert!(
                branch.starts_with("sheet/demo_row-"),
                "named for the row: {branch}"
            );
            assert!(!commit.is_empty());
            // AND THE FILE IS ACTUALLY IN THE COMMIT. "It made a branch" is not
            // the assertion that matters — the previous version got as far as
            // creating one and then failed to add anything to it.
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(["show", "--name-only", "--format=", "HEAD"])
                .output()
                .unwrap();
            let named = String::from_utf8_lossy(&out.stdout);
            assert!(
                named.contains("crates/vleo-mod-demo/nodes/demo_row/node.toml"),
                "the edited row must be in the commit, whole path and all: {named}"
            );
        }
        form::Proposed::Nothing => panic!("a modified row is something to propose"),
        form::Proposed::Refused(e) => panic!("refused: {e}"),
    }
}

#[test]
fn an_untracked_row_is_committed_too() {
    // The case that always worked, kept so a fix for one does not break the
    // other: an untracked file's porcelain line is `?? path`, with no leading
    // space.
    let Some(s) = scratch_repo("untracked") else {
        return;
    };
    let root = s.0.clone();
    std::fs::write(
        root.join("crates/vleo-mod-demo/nodes/demo_row/page.html"),
        "<p>generated</p>\n",
    )
    .unwrap();
    match form::propose(&root, "add the page", "docs") {
        form::Proposed::Ok { files, .. } => assert_eq!(
            files, 2,
            "both the modified sheet and the untracked artefact"
        ),
        form::Proposed::Nothing => panic!("there was something to propose"),
        form::Proposed::Refused(e) => panic!("refused: {e}"),
    }
}

/// Puts a whole row folder back when this goes out of scope: every file to its
/// bytes, and every file that was not there removed. `Restore` puts back one
/// sheet; a publish writes files the row never had.
struct RestoreFolder {
    dir: PathBuf,
    files: Vec<(PathBuf, Vec<u8>)>,
}

impl RestoreFolder {
    fn take(dir: &Path) -> RestoreFolder {
        let files = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .map(|p| {
                let b = std::fs::read(&p).unwrap();
                (p, b)
            })
            .collect();
        RestoreFolder {
            dir: dir.to_path_buf(),
            files,
        }
    }
    fn names(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }
}

impl Drop for RestoreFolder {
    fn drop(&mut self) {
        for e in std::fs::read_dir(&self.dir).unwrap().flatten() {
            if !self.files.iter().any(|(p, _)| *p == e.path()) {
                let _ = std::fs::remove_file(e.path());
            }
        }
        for (p, b) in &self.files {
            let _ = std::fs::write(p, b);
        }
    }
}

#[test]
fn a_refused_publish_leaves_no_file_behind() {
    // The whole journey that found it: a seeded row filled in completely, with
    // one disagreement the per-save checks of a seeded row do not look for —
    // the lower bound above the upper — and then published. Publishing runs
    // every check, the domain check refuses, and the restore said "nothing
    // changed" while the model, contract, module and evidence it had just
    // generated for the first time stayed in the folder.
    let _serial = serially();
    let root = root();
    let row = "sys_attitude_control_sizing_aerodynamic_trim_angle";
    let tree = vleo_sheet::load::load_all(&root).unwrap();
    let sh = tree
        .sheets
        .get(row)
        .expect("the seeded row this test fills");
    assert!(sh.is_seeded(), "this test needs {row} to be seeded");
    let dir = sh.dir.clone();
    let _guard = RestoreFolder::take(&dir);
    let files_before = RestoreFolder::names(&dir);

    // Filled in textually — the same writer the form uses — so the only thing
    // under test is what publishing does with it.
    let path = dir.join("node.toml");
    let mut t = std::fs::read_to_string(&path).unwrap();
    for (f, v) in [
        (
            "question",
            "At what angle does the aerodynamic moment balance the control authority?",
        ),
        (
            "expression",
            "alpha_trim = atan2(C_m_delta * delta_max, C_m_alpha)",
        ),
        ("source", "larson_wertz"),
        ("symbol", "alpha_trim"),
        ("type", "Angle"),
        ("unit", "Degree"),
        (
            "reason_lower",
            "below this the vehicle is not trimming against the flow",
        ),
        (
            "reason_upper",
            "above this the panel model is outside its fitted range",
        ),
        ("lower", "40"),
        ("upper", "30"),
    ] {
        t = form::set(&t, f, v).unwrap_or_else(|e| panic!("{f}: {e}"));
    }
    let s = |v: &str| v.to_string();
    t = form::block_text(
        &t,
        "input",
        "add",
        0,
        &[
            ("binding", s("h")),
            ("var", s("orbit_altitude")),
            ("type", s("Length")),
        ],
    )
    .unwrap();
    t = form::block_text(
        &t,
        "algorithm",
        "add",
        0,
        &[
            ("text", s("balance the moments")),
            ("binds", s("out")),
            ("type", s("Angle")),
        ],
    )
    .unwrap();
    std::fs::write(&path, &t).unwrap();

    match form::publish(&root, row, &form::file_hash(&t)) {
        Saved::Refused(e) => assert!(
            e.contains("domain"),
            "refused, but not for the crossed bounds: {e}"
        ),
        Saved::Ok { .. } => panic!("a row whose lower bound is above its upper was published"),
        Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
    }
    assert_eq!(
        RestoreFolder::names(&dir),
        files_before,
        "a refused publish must leave the folder holding exactly the files it held"
    );
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("state = \"empty\""),
        "and the row is still seeded"
    );
}
