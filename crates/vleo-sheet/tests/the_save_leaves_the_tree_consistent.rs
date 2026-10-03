//! The save transaction, against the real tree.
//!
//! What matters here is not that a good edit works — it is that a bad one
//! leaves NOTHING changed. A browser that half-edits a repository is worse than
//! one that cannot edit it at all, so every failing path is checked for the
//! sheet being exactly as it was.

use std::path::{Path, PathBuf};
use vleo_sheet::form::{self, Saved};

/// A copy of the tree in a temporary folder, made once for this test binary.
///
/// These tests write sheets and regenerate folders. They once did it in the
/// checkout itself, under a lock that only this binary respected, so a
/// developer running the suite beside an edit in progress could have that edit
/// overwritten and put back. Everything they touch is copied here instead.
fn root() -> PathBuf {
    static COPY: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    COPY.get_or_init(|| {
        let real = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let to = std::env::temp_dir().join(format!("vleo-save-tree-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&to);
        for e in std::fs::read_dir(real.join("crates")).unwrap().flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("vleo-mod-") {
                copy(
                    &e.path().join("nodes"),
                    &to.join("crates").join(&name).join("nodes"),
                );
            }
        }
        copy(
            &real.join("crates/vleo-wasm/src"),
            &to.join("crates/vleo-wasm/src"),
        );
        for d in ["layers", "cases", "sources", "web"] {
            copy(&real.join(d), &to.join(d));
        }
        to
    })
    .clone()
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            copy(&p, &to.join(e.file_name()));
        } else {
            std::fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

/// A published row with a plain single-line field to move.
const ROW: &str = "gnc_along_track_error";

fn sheet_path(root: &Path) -> PathBuf {
    root.join("crates/vleo-mod-acs/nodes")
        .join(ROW)
        .join("node.toml")
}

/// EVERY TEST IN THIS FILE EDITS THE SAME COPY OF THE TREE, so they run one at
/// a time.
///
/// Cargo runs the tests in one binary on several threads. Two of these reading a
/// sheet while a third was mid-edit failed the suite about one run in three, on
/// an assertion about a sheet the failing test had not touched — the worst kind
/// of red, because it points at the wrong test.
///
/// The lock is held for the whole of each test rather than around each write:
/// what races is not the write but the read-edit-compare around it.
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
    // The rule itself, as a function: no git, no files, no checkout. The names an
    // assistant arrives under, in the shapes a name arrives in.
    let root = root();
    for who in [
        "Claude",
        "claude opus 5",
        "Assistant",
        "Copilot",
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
            e.message().contains("blank"),
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
                e.contains("assistant") || e.contains("user.name"),
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

#[test]
fn a_refused_method_leaves_the_rust_its_hole_held() {
    // Found taking a group's release in: a method, with its author's cases but
    // not their code, applied to a published row. The gate refused, as it
    // should — and the restore regenerated model.rs from the restored sheet,
    // which wrote the hole back EMPTY, because the regeneration from the
    // edited sheet had already dropped the hole's Rust. Under "nothing changed".
    use vleo_sheet::template::{self, Derisk, Form};
    let _serial = serially();
    let root = root();
    let row = "sw_f107_design_long";
    let tree = vleo_sheet::load::load_all(&root).unwrap();
    let sh = tree
        .sheets
        .get(row)
        .expect("the published row this test edits");
    let dir = sh.dir.clone();
    let _guard = RestoreFolder::take(&dir);
    let model = std::fs::read(dir.join("model.rs")).unwrap();
    assert!(
        String::from_utf8_lossy(&model).contains("1.28"),
        "this test needs {row}'s hole to hold its Rust"
    );
    let original = template::content(sh);
    let mut filled = original.clone();
    filled.fields.insert(
        "method_text".into(),
        "if central < 60 then\n  refuse \"below the floor\"\nend\nreturn central + 1.28 * spread"
            .into(),
    );
    let case = |label: &str, refuse: bool, expect: &str, inputs: &str| {
        let mut r = std::collections::BTreeMap::new();
        r.insert("label".to_string(), label.to_string());
        r.insert("refuse".into(), if refuse { "yes" } else { "no" }.into());
        if !refuse {
            r.insert("expect".into(), expect.into());
            r.insert("tolerance".into(), "1e-9".into());
        }
        r.insert("inputs".into(), inputs.into());
        r
    };
    filled.arrays.insert(
        "case".into(),
        vec![
            case("one", false, "125.6", "{ central = 100.0, spread = 20.0 }"),
            case(
                "two",
                false,
                "177.22160896",
                "{ central = 160.0, spread = 13.454382 }",
            ),
            case(
                "three",
                false,
                "112.8",
                "{ central = 100.0, spread = 10.0 }",
            ),
            case("floor", true, "", "{ central = 50.0, spread = 0.0 }"),
        ],
    );
    let base = form::file_hash(&std::fs::read_to_string(dir.join("node.toml")).unwrap());
    let f = Form {
        node: row.into(),
        base,
        name: "A. Person".into(),
        date: "2026-10-02".into(),
        ai: "none".into(),
        original,
        filled,
        derisk: Derisk {
            believed: "the hole was the method".into(),
            tested: "the cases below".into(),
            learned: "they agree".into(),
            cost: "none".into(),
            changed: "the method is written down".into(),
            risks: String::new(),
            rests_on: "the cases".into(),
            breaks_if: "a case disagrees".into(),
        },
        ..Form::default()
    };
    // The kernel's translated methods are written in the same step, and must be
    // put back as well: the refused method once stayed there, named in mod.rs.
    let methods = root.join("crates/vleo-core/src/physics/methods");
    let kernel = |d: &std::path::Path| -> Vec<(std::path::PathBuf, Vec<u8>)> {
        // No folder yet is no methods yet: the copy of the tree starts without one.
        let mut v: Vec<_> = std::fs::read_dir(d)
            .into_iter()
            .flatten()
            .map(|e| e.unwrap().path())
            .map(|p| {
                let b = std::fs::read(&p).unwrap();
                (p, b)
            })
            .collect();
        v.sort();
        v
    };
    let kernel_before = kernel(&methods);
    let p = template::plan_form(&root, f).unwrap();
    assert!(p.applicable() > 0, "the plan has something to apply");
    match template::apply(&root, &p) {
        Saved::Refused(e) => assert!(e.contains("nothing changed"), "{e}"),
        Saved::Ok { .. } => panic!("cases with no author code were applied"),
        Saved::Stale { current } => panic!("unexpectedly stale, current {current}"),
    }
    assert!(
        std::fs::read(dir.join("model.rs")).unwrap() == model,
        "a refused method must leave model.rs byte for byte as it was — its hole's Rust included"
    );
    assert!(
        kernel(&methods) == kernel_before,
        "a refused method must leave the kernel's methods as they were — no translation left behind"
    );
}
