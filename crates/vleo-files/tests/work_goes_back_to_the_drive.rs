//! A person's own work put back on the drive from their working copy, and
//! nothing else (docs/briefs/F3.md, F3-c).
//!
//! A small drive of invented people: Test Pat (71), programme manager; Test
//! Sam (72), system engineer; Test Ola (73), who owns solar; Test Nia (74),
//! assigned solar's `sw_activity_band`; Test Ivo (75), a node engineer of
//! solar assigned nothing. The node is the one solar's 1.1 release holds.
//! The copy is the drive's files, taken whole, with its record beside them.

use std::path::{Path, PathBuf};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::copied::{self, Copied};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::people::{self, Change, Valve};
use vleo_files::put_back::{digests, put_back, Drive};
use vleo_files::save::{save, sign_in_place, Saved};
use vleo_files::sqlite;

const APP: &str = "vleo 1.0.0 (test)";
const UID: &str = "sw_activity_band";
const NODE: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";
const SOLAR: &str = "groups/l3_solar/l3_solar.vgroup";
const AT: &str = "2026-10-15T12:00:00Z";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn register(f: &mut File, name: &str, role: &str, k: &SigningKey, by: &str) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(""),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: k.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s(by),
        revoked_from: s(""),
    });
}

fn mount(f: &mut File, uid: &str, group: &str, owner: &str) {
    f.blocks.push(Block {
        uid: s(uid),
        id: s(uid),
        behaviour: s("children"),
        ..Block::default()
    });
    f.mounts.push(Mount {
        block_uid: s(uid),
        group_id: s(group),
        release: s(""),
    });
    f.assignments.push(Assignment {
        block_uid: s(uid),
        person: s(owner),
        contract_version: 1,
    });
}

fn group(id: &str, writer: &str) -> File {
    let mut f = File::new(Kind::Group, APP);
    f.meta.insert(s("group_id"), s(id));
    f.meta.insert(s("writer"), s(writer));
    f.meta.insert(s("based_on"), s(""));
    f
}

fn signed(mut f: File, who: &str, k: &SigningKey) -> File {
    let row = chain::sign(&f, FILE, who, k, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
    f
}

/// The node as solar's 1.1 release holds it, in a node file of its own.
fn node() -> File {
    let how = vleo_files::upgrade::Upgrade {
        app: APP,
        at: "2026-10-01",
    };
    let rel = sqlite::open(
        &root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo"),
        &how,
    )
    .unwrap()
    .file;
    let mut n = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("block_uid", UID),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Test Nia"),
    ] {
        n.meta.insert(s(k), s(v));
    }
    n.blocks = vec![rel.blocks.iter().find(|b| b.uid == UID).unwrap().clone()];
    n.blocks[0].revision = 1;
    n.ports = rel
        .ports
        .iter()
        .filter(|p| p.block_uid == UID)
        .cloned()
        .collect();
    n.wires = rel
        .wires
        .iter()
        .filter(|w| w.to_block == UID)
        .cloned()
        .collect();
    n.texts = rel
        .texts
        .iter()
        .filter(|t| t.scope == UID)
        .cloned()
        .collect();
    n.tables = rel
        .tables
        .iter()
        .filter(|t| t.scope == UID)
        .cloned()
        .collect();
    n
}

/// The drive and a copy taken of it, under `scratch`.
fn world(name: &str) -> (PathBuf, PathBuf) {
    let scratch = std::env::temp_dir().join(format!("vleo-put-back-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let (drive, copy) = (scratch.join("drive"), scratch.join("copy"));
    let mut programme = group("programme", "Test Pat");
    register(
        &mut programme,
        "Test Pat",
        "programme manager",
        &key(71),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Sam",
        "system engineer",
        &key(72),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Ola",
        "subsystem engineer",
        &key(73),
        "Test Pat",
    );
    mount(&mut programme, "root", "systems", "Test Sam");
    let mut systems = group("systems", "Test Sam");
    mount(
        &mut systems,
        "sys_space_environment",
        "l3_solar",
        "Test Ola",
    );
    let mut solar = group("l3_solar", "Test Ola");
    register(
        &mut solar,
        "Test Nia",
        "node engineer",
        &key(74),
        "Test Ola",
    );
    register(
        &mut solar,
        "Test Ivo",
        "node engineer",
        &key(75),
        "Test Ola",
    );
    solar.assignments.push(Assignment {
        block_uid: s(UID),
        person: s("Test Nia"),
        contract_version: 1,
    });
    std::fs::create_dir_all(drive.join("groups/programme")).unwrap();
    std::fs::create_dir_all(drive.join("groups/systems")).unwrap();
    std::fs::create_dir_all(drive.join("groups/l3_solar/nodes")).unwrap();
    for (f, who, k, at) in [
        (
            programme,
            "Test Pat",
            key(71),
            "groups/programme/programme.vgroup",
        ),
        (
            systems,
            "Test Sam",
            key(72),
            "groups/systems/systems.vgroup",
        ),
        (solar, "Test Ola", key(73), SOLAR),
    ] {
        sqlite::write(&signed(f, who, &k), &drive.join(at)).unwrap();
    }
    sqlite::write(&node(), &drive.join(NODE)).unwrap();
    std::fs::write(
        drive.join("START HERE.md"),
        format!("# START HERE\n\n    {}\n", key(71).public().fingerprint()),
    )
    .unwrap();
    take_copy(&drive, &copy);
    (drive, copy)
}

/// The drive's files into `copy`, whole, and the record beside them.
fn take_copy(drive: &Path, copy: &Path) {
    let files = digests(drive).unwrap();
    for path in files.keys() {
        let to = copy.join(path);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(drive.join(path), &to).unwrap();
    }
    let r = Copied {
        drive: std::fs::canonicalize(drive).unwrap().display().to_string(),
        copied_at: s("2026-10-15T09:00:00Z"),
        files,
    };
    std::fs::write(copy.join(copied::FILE_NAME), r.to_toml()).unwrap();
}

/// The drive's chain, opened from the drive's own files.
struct Opened {
    anchor: String,
    programme: Registry,
    systems: Registry,
    groups: Vec<Registry>,
}

fn opened(at: &Path) -> Opened {
    let read = |p: &str| sqlite::read(&at.join(p)).unwrap();
    let anchor = key(71).public().fingerprint();
    let programme =
        chain::open_programme(read("groups/programme/programme.vgroup"), &anchor).unwrap();
    let systems = chain::open_systems(&read("groups/systems/systems.vgroup"), &programme).unwrap();
    let solar = {
        let (parent, _) = chain::mounted_by("l3_solar", &[&programme, &systems]).unwrap();
        chain::open_group_in(&read(SOLAR), parent, &programme).ok()
    };
    Opened {
        anchor,
        programme,
        systems,
        groups: solar.into_iter().collect(),
    }
}

fn drive_of(o: &Opened) -> Drive<'_> {
    Drive {
        anchor: &o.anchor,
        programme: &o.programme,
        systems: Some(&o.systems),
        groups: &o.groups,
        left_out: &[],
    }
}

fn opened_digest(at: &Path) -> String {
    copied::digest(&std::fs::read(at).unwrap())
}

/// Nia signs her node in the copy.
fn nia_signs(copy: &Path) {
    let g = opened(copy).groups.remove(0);
    let seen = opened_digest(&copy.join(NODE));
    match sign_in_place(copy, NODE, &seen, "Test Nia", &key(74), AT, APP, &g).unwrap() {
        Saved::Written { .. } => {}
        other => panic!("not signed: {other:?}"),
    }
}

fn line<'a>(r: &'a vleo_files::put_back::Report, path: &str) -> &'a vleo_files::put_back::Line {
    r.lines
        .iter()
        .find(|l| l.path == path)
        .unwrap_or_else(|| panic!("no line for {path}: {:?}", r.lines))
}

#[test]
fn the_signed_node_goes_back_and_a_copy_taken_again_holds_it_signed() {
    let (drive, copy) = world("signed");
    nia_signs(&copy);
    let o = opened(&drive);
    let r = put_back(&copy, &drive, "Test Nia", &drive_of(&o), None).unwrap();
    assert_eq!(r.back(), [NODE], "{:?}", r.lines);
    assert!(line(&r, NODE).why.contains("signed by Test Nia"));
    assert!(!r.written);
    assert_ne!(
        std::fs::read(drive.join(NODE)).unwrap(),
        std::fs::read(copy.join(NODE)).unwrap()
    );
    // Named as the report says: written.
    let r = put_back(&copy, &drive, "Test Nia", &drive_of(&o), Some(&[s(NODE)])).unwrap();
    assert!(r.written);
    assert_eq!(
        std::fs::read(drive.join(NODE)).unwrap(),
        std::fs::read(copy.join(NODE)).unwrap()
    );
    // The record says the file is as the drive holds it: nothing is left.
    let again = put_back(&copy, &drive, "Test Nia", &drive_of(&o), None).unwrap();
    assert!(again.lines.is_empty(), "{:?}", again.lines);
    // No file of the program's own is left on the drive.
    let left: Vec<_> = std::fs::read_dir(drive.join("groups/l3_solar/nodes"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, ["sw_activity_band.vnode"]);
    assert!(!drive.join(".vleo-put-back.lock").exists());
    // A copy taken again holds her node, and her signature counts in it.
    let second = drive.parent().unwrap().join("second");
    take_copy(&drive, &second);
    let f = sqlite::read(&second.join(NODE)).unwrap();
    chain::check_node(&f, UID, &opened(&second).groups[0]).expect("it is signed on the drive");
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_file_changed_on_the_drive_since_the_copy_is_never_written_over() {
    let (drive, copy) = world("changed");
    nia_signs(&copy);
    // Someone else's revision lands on the drive meanwhile.
    let mut theirs = node();
    theirs.meta.insert(s("writer"), s("Test Ivo"));
    sqlite::write(&theirs, &drive.join(NODE)).unwrap();
    let before = std::fs::read(drive.join(NODE)).unwrap();
    let o = opened(&drive);
    let r = put_back(&copy, &drive, "Test Nia", &drive_of(&o), None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back && l.why.contains("changed on the drive since your copy"),
        "{l:?}"
    );
    let e = put_back(&copy, &drive, "Test Nia", &drive_of(&o), Some(&[s(NODE)])).unwrap_err();
    assert!(
        e.message().contains("nothing in your copy goes back"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(drive.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_file_its_person_did_not_sign_does_not_go_back() {
    let (drive, copy) = world("unsigned");
    // Saved, not signed.
    let g = opened(&copy).groups.remove(0);
    let mut edit = sqlite::read(&copy.join(NODE)).unwrap();
    edit.texts.retain(|t| !(t.scope == UID && t.kind == "note"));
    edit.texts.push(Text {
        scope: s(UID),
        kind: s("note"),
        body: s("A line of Test Nia's own."),
    });
    let seen = opened_digest(&copy.join(NODE));
    save(&copy, NODE, &edit, &seen, "Test Nia", APP, &g).unwrap();
    let o = opened(&drive);
    let r = put_back(&copy, &drive, "Test Nia", &drive_of(&o), None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back && l.why.contains("carries no current signature by Test Nia"),
        "{l:?}"
    );
    // Signed by Nia, it is still not Ivo's to put back.
    nia_signs(&copy);
    let r = put_back(&copy, &drive, "Test Ivo", &drive_of(&o), None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back && l.why.contains("carries no current signature by Test Ivo"),
        "{l:?}"
    );
    let before = std::fs::read(drive.join(NODE)).unwrap();
    let e = put_back(&copy, &drive, "Test Ivo", &drive_of(&o), Some(&[s(NODE)])).unwrap_err();
    assert!(
        e.message().contains("nothing in your copy goes back"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(drive.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_key_revoked_on_the_drive_since_the_copy_puts_nothing_back() {
    let (drive, copy) = world("revoked");
    nia_signs(&copy);
    // On the drive, Ola revokes Nia's key from before the signing.
    let o = opened(&drive);
    let (parent, _) = chain::mounted_by("l3_solar", &[&o.programme, &o.systems]).unwrap();
    let valve = Valve::Group {
        mounted_by: parent,
        programme: &o.programme,
    };
    let revoked = people::change(
        &sqlite::read(&drive.join(SOLAR)).unwrap(),
        &valve,
        "Test Ola",
        &key(73),
        "2026-10-16T00:00:00Z",
        &Change::Revoke {
            person: s("Test Nia"),
            public_key: key(74).public().to_hex(),
            from: s("2026-10-01"),
        },
    )
    .unwrap();
    sqlite::write(&revoked, &drive.join(SOLAR)).unwrap();
    let o = opened(&drive);
    let r = put_back(&copy, &drive, "Test Nia", &drive_of(&o), None).unwrap();
    let l = line(&r, NODE);
    assert!(!l.back && l.why.contains("revoked"), "{l:?}");
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_group_file_changed_at_its_valve_goes_back_as_its_owner_s() {
    let (drive, copy) = world("group");
    let o = opened(&copy);
    let (parent, _) = chain::mounted_by("l3_solar", &[&o.programme, &o.systems]).unwrap();
    let valve = Valve::Group {
        mounted_by: parent,
        programme: &o.programme,
    };
    let changed = people::change(
        &sqlite::read(&copy.join(SOLAR)).unwrap(),
        &valve,
        "Test Ola",
        &key(73),
        AT,
        &Change::Register {
            person: Person {
                name: s("Test Kai"),
                role: s("node engineer"),
                deputy_for: s(""),
            },
            public_key: key(76).public().to_hex(),
            proof: people::proof(&key(76), "Test Kai"),
        },
    )
    .unwrap();
    sqlite::write(&changed, &copy.join(SOLAR)).unwrap();
    let o = opened(&drive);
    let r = put_back(&copy, &drive, "Test Ola", &drive_of(&o), None).unwrap();
    assert_eq!(r.back(), [SOLAR], "{:?}", r.lines);
    let r = put_back(&copy, &drive, "Test Nia", &drive_of(&o), None).unwrap();
    let l = line(&r, SOLAR);
    assert!(
        !l.back && l.why.contains("carries no current signature by Test Nia"),
        "{l:?}"
    );
    let r = put_back(&copy, &drive, "Test Ola", &drive_of(&o), Some(&[s(SOLAR)])).unwrap();
    assert!(r.written);
    assert!(opened(&drive).groups[0]
        .file
        .people
        .iter()
        .any(|p| p.name == "Test Kai"));
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn only_the_files_the_report_names_are_written_and_nothing_is_deleted() {
    let (drive, copy) = world("named");
    nia_signs(&copy);
    std::fs::write(copy.join("notes.txt"), "a note of my own").unwrap();
    std::fs::remove_file(copy.join("START HERE.md")).unwrap();
    let o = opened(&drive);
    let r = put_back(&copy, &drive, "Test Nia", &drive_of(&o), None).unwrap();
    assert_eq!(r.back(), [NODE]);
    let l = line(&r, "notes.txt");
    assert!(
        !l.back && l.why.contains("not a file the application puts back"),
        "{l:?}"
    );
    let l = line(&r, "START HERE.md");
    assert!(
        !l.back && l.why.contains("nothing is deleted on the drive"),
        "{l:?}"
    );
    let before = std::fs::read(drive.join(NODE)).unwrap();
    for named in [vec![], vec![s(NODE), s("notes.txt")], vec![s("notes.txt")]] {
        let e = put_back(&copy, &drive, "Test Nia", &drive_of(&o), Some(&named)).unwrap_err();
        assert!(
            e.message().contains("not the ones that go back now"),
            "{named:?}: {}",
            e.message()
        );
    }
    assert_eq!(std::fs::read(drive.join(NODE)).unwrap(), before);
    assert!(!drive.join("notes.txt").exists());
    assert!(drive.join("START HERE.md").exists());
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_drive_someone_is_writing_or_another_drive_is_not_written() {
    let (drive, copy) = world("locked");
    nia_signs(&copy);
    let o = opened(&drive);
    std::fs::write(drive.join(".vleo-put-back.lock"), "1").unwrap();
    let before = std::fs::read(drive.join(NODE)).unwrap();
    let e = put_back(&copy, &drive, "Test Nia", &drive_of(&o), Some(&[s(NODE)])).unwrap_err();
    assert!(
        e.message().contains("someone is putting work back"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(drive.join(NODE)).unwrap(), before);
    std::fs::remove_file(drive.join(".vleo-put-back.lock")).unwrap();
    let elsewhere = drive.parent().unwrap().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let e = put_back(&copy, &elsewhere, "Test Nia", &drive_of(&o), None).unwrap_err();
    assert!(
        e.message()
            .contains("goes back only to the drive it came from"),
        "{}",
        e.message()
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_node_whose_checks_refuse_it_does_not_go_back_whatever_signed_it() {
    let (drive, copy) = world("checks");
    // Signed past its checks, by a tool that signs anything: its case
    // disagrees with its method, and its signature still counts.
    let mut bad = node();
    let t = bad
        .tables
        .iter_mut()
        .find(|t| t.path == "results/isolation.csv")
        .unwrap();
    t.csv = t.csv.replacen("\n64,1,", "\n64,2,", 1);
    let row = chain::sign(&bad, UID, "Test Nia", &key(74), AT, "ok", "").unwrap();
    bad.signatures.push(row);
    sqlite::write(&bad, &copy.join(NODE)).unwrap();
    let o = opened(&drive);
    chain::check_node(&bad, UID, &o.groups[0]).expect("its signature counts");
    let r = put_back(&copy, &drive, "Test Nia", &drive_of(&o), None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back && l.why.contains("its checks refuse it") && l.why.contains("does not reproduce"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_file_already_on_the_drive_as_it_is_in_the_copy_is_said_so() {
    let (drive, copy) = world("same");
    nia_signs(&copy);
    std::fs::copy(copy.join(NODE), drive.join(NODE)).unwrap();
    let o = opened(&drive);
    let r = put_back(&copy, &drive, "Test Nia", &drive_of(&o), None).unwrap();
    let l = line(&r, NODE);
    assert!(!l.back && l.why.contains("already on the drive"), "{l:?}");
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}
