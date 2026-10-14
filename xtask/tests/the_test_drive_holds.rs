//! The test drive every browser scenario starts from holds together
//! (docs/PLAN_F.md, decision 5): the same bytes every time it is written; its
//! programme's file checks through the anchor START HERE gives, and each
//! group's file through the programme's; every invented person's key opens
//! with the passphrase written for it and with no other; no private key is on
//! the drive; Solar's two releases beside it are as they were sealed; and a
//! folder that is not a test drive is refused, not emptied.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use vleo_files::chain;
use vleo_files::keys::LockedKey;
use vleo_files::sqlite;
use vleo_files::upgrade::Upgrade;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn scratch(name: &str) -> PathBuf {
    let s = std::env::temp_dir().join(format!("vleo-test-drive-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    s
}

fn test_drive(out: &Path) -> (bool, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["test-drive", "--out"])
        .arg(out)
        .current_dir(root())
        .output()
        .expect("xtask did not start");
    (
        o.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        ),
    )
}

/// Every file under `dir`, by its path inside it, with its bytes.
fn files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().into_owned();
                out.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

fn open(path: &Path) -> vleo_files::model::File {
    let how = Upgrade {
        app: "vleo (test)",
        at: "",
    };
    sqlite::open(path, &how).unwrap().file
}

#[test]
fn the_test_drive_is_the_same_every_time_and_checks_through_its_anchor() {
    let (a, b) = (scratch("a"), scratch("b"));
    let (ok, said) = test_drive(&a);
    assert!(ok, "{said}");
    let (ok, said) = test_drive(&b);
    assert!(ok, "{said}");
    let (fa, fb) = (files(&a), files(&b));
    assert_eq!(
        fa.keys().collect::<Vec<_>>(),
        fb.keys().collect::<Vec<_>>(),
        "two test drives hold different files"
    );
    for (path, bytes) in &fa {
        assert!(bytes == &fb[path], "{path} differs between two test drives");
    }
    // Written again over itself, it is the same again.
    let (ok, said) = test_drive(&a);
    assert!(ok, "{said}");
    assert!(
        files(&a) == fb,
        "a test drive written again is not the same"
    );

    // The anchor START HERE gives, and the programme's file through it.
    let drive = a.join("drive");
    let start = std::fs::read_to_string(drive.join("START HERE.md")).unwrap();
    assert!(start.contains("invented"), "{start}");
    let anchor = start
        .lines()
        .map(str::trim)
        .find(|l| l.len() == 64 && l.chars().all(|c| c.is_ascii_hexdigit()))
        .expect("START HERE gives no fingerprint");
    let programme = chain::open_programme(
        open(&drive.join("groups/programme/programme.vgroup")),
        anchor,
    )
    .unwrap_or_else(|e| panic!("the programme's file does not check: {e}"));
    let wrong = "0".repeat(64);
    assert!(
        chain::open_programme(
            open(&drive.join("groups/programme/programme.vgroup")),
            &wrong
        )
        .is_err(),
        "the programme's file checks against an anchor that is not its own"
    );

    // Each group's file through the programme's, and its node engineer in it.
    for g in ["l3_solar", "l3_prop", "l3_power"] {
        let f = open(&drive.join(format!("groups/{g}/{g}.vgroup")));
        let reg = chain::open_group(&f, &programme)
            .unwrap_or_else(|e| panic!("{g}'s group file does not check: {e}"));
        assert!(
            reg.file.people.iter().any(|p| p.role == "node engineer"),
            "{g} registers no node engineer"
        );
        assert!(
            drive.join(format!("groups/{g}/releases")).is_dir(),
            "{g} has no releases/"
        );
        let nodes = std::fs::read_dir(drive.join(format!("groups/{g}/nodes")))
            .unwrap()
            .count();
        let design = std::fs::read_dir(root().join(format!("design/groups/{g}/nodes")))
            .unwrap()
            .count();
        assert_eq!(nodes, design, "{g} does not carry design/'s node files");
    }

    // Every person: invented, and their key opens with their passphrase and
    // no other. No private key is on the drive.
    let people: toml::Value = std::fs::read_to_string(a.join("people.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let people = people["person"].as_array().unwrap();
    assert!(people.len() >= 5, "{people:?}");
    for p in people {
        let name = p["name"].as_str().unwrap();
        assert!(
            name.starts_with("Test "),
            "{name} does not say it is invented"
        );
        let key = open(&a.join(p["key"].as_str().unwrap()));
        let locked = LockedKey::from_row(&key.locked_keys[0]).unwrap();
        let opened = locked
            .unlock(p["passphrase"].as_str().unwrap())
            .unwrap_or_else(|e| panic!("{name}'s key does not open: {e}"));
        assert_eq!(
            opened.public().to_hex(),
            p["public_key"].as_str().unwrap(),
            "{name}'s key is not the one listed"
        );
        assert!(
            locked.unlock("not the passphrase").is_err(),
            "{name}'s key opens with a wrong passphrase"
        );
    }
    assert!(
        !files(&drive).keys().any(|p| p.ends_with(".vkey")),
        "a private key is on the drive"
    );

    // Solar's two releases beside the drive, each as its seal says.
    for v in ["1.1", "1.2"] {
        let r = vleo_files::intake::Release::open(&a.join(format!("releases/l3_solar-{v}.vleo")))
            .unwrap_or_else(|e| panic!("Solar {v} does not open: {e}"));
        assert_eq!(r.version, v);
        r.check_seal()
            .unwrap_or_else(|e| panic!("Solar {v} is not as it was sealed: {e}"));
    }
    let _ = std::fs::remove_dir_all(&a);
    let _ = std::fs::remove_dir_all(&b);
}

#[test]
fn a_folder_that_is_not_a_test_drive_is_refused_not_emptied() {
    let s = scratch("other");
    std::fs::create_dir_all(&s).unwrap();
    std::fs::write(s.join("keep.txt"), "somebody's work").unwrap();
    let (ok, said) = test_drive(&s);
    assert!(!ok, "{said}");
    assert!(said.contains("is not a test drive"), "{said}");
    assert_eq!(
        std::fs::read_to_string(s.join("keep.txt")).unwrap(),
        "somebody's work"
    );
    let _ = std::fs::remove_dir_all(&s);
}
