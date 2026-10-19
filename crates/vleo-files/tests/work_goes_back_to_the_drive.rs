//! A person's own work put back on the drive from their working copy, and
//! nothing else (docs/briefs/F3.md, F3-c).
//!
//! A small drive of invented people: Test Pat (71), programme manager; Test
//! Sam (72), system engineer; Test Ola (73), who owns solar; Test Nia (74),
//! assigned solar's `sw_activity_band`; Test Ivo (75), a node engineer of
//! solar assigned nothing. The node is the one solar's 1.1 release holds.
//! The copy is the drive's files, taken whole, with its record beside them.
//!
//! A request asked in a node's file (F3-d) goes back with it, or by itself.

use std::path::{Path, PathBuf};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::copied::{self, Copied};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::people::{self, Change, Valve};
use vleo_files::put_back::{digests, put_back, Chain, Report};
use vleo_files::save::{ask_in_place, save, sign_in_place, Asked, Saved};
use vleo_files::sqlite;

const APP: &str = "vleo 1.0.0 (test)";
const UID: &str = "sw_activity_band";
const NODE: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";
const SOLAR: &str = "groups/l3_solar/l3_solar.vgroup";
const AT: &str = "2026-10-15T12:00:00Z";
const IVO: &str = "groups/l3_solar/nodes/ivo_node.vnode";
const SYS: &str = "groups/systems/nodes/sys_node.vnode";
const ROOT: &str = "groups/programme/nodes/root.vnode";

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
        proof: vleo_files::people::proof(k, name),
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

/// A node file holding one bare block, revision 1.
fn bare(group: &str, uid: &str, id: &str, writer: &str) -> File {
    let mut n = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", group),
        ("block_uid", uid),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", writer),
    ] {
        n.meta.insert(s(k), s(v));
    }
    n.blocks.push(Block {
        uid: s(uid),
        id: s(id),
        behaviour: s("method"),
        revision: 1,
        contract_version: 1,
        ..Block::default()
    });
    n
}

/// `f`, solar's node, as the node `uid` of `group`: every row of its own
/// under the new uid, and its file's meta saying so.
fn renamed(mut f: File, group: &str, uid: &str) -> File {
    f.meta.insert(s("group_id"), s(group));
    f.meta.insert(s("block_uid"), s(uid));
    f.blocks[0].uid = s(uid);
    f.blocks[0].id = s(uid);
    for p in f.ports.iter_mut() {
        p.block_uid = s(uid);
    }
    for w in f.wires.iter_mut() {
        w.to_block = s(uid);
    }
    for t in f.texts.iter_mut() {
        t.scope = s(uid);
    }
    for t in f.tables.iter_mut() {
        t.scope = s(uid);
    }
    f
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
    register(
        &mut programme,
        "Test Pia",
        "subsystem engineer",
        &key(79),
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
    // Power, Pia's, which answers solar's releases.
    mount(&mut systems, "sys_power", "l3_power", "Test Pia");
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
    // Ola's record of which belief broke, for each release of solar sealed.
    solar.tables.push(Tbl {
        scope: s(FILE),
        path: s("versions.csv"),
        csv: s("version,date,by,believed,tested,learned,changed,risks,cost,rests_on,breaks_if\n\
             1.0,2026-10-01,Test Ola,,,,,,,the four NOAA SWPC activity levels,a correction to those levels\n\
             1.1,2026-10-05,Test Ola,The bands held.,Read against their cases.,They held.,Nothing but words.,,,the four NOAA SWPC activity levels,a correction to those levels\n"),
    });
    for (uid, who) in [(UID, "Test Nia"), ("ivo_uid", "Test Ivo")] {
        solar.assignments.push(Assignment {
            block_uid: s(uid),
            person: s(who),
            contract_version: 1,
        });
    }
    // The systems' own node, and its node engineer, whom the systems' file
    // registers.
    register(
        &mut systems,
        "Test Uma",
        "node engineer",
        &key(77),
        "Test Sam",
    );
    systems.assignments.push(Assignment {
        block_uid: s("sys_node"),
        person: s("Test Uma"),
        contract_version: 1,
    });
    std::fs::create_dir_all(drive.join("groups/programme/nodes")).unwrap();
    std::fs::create_dir_all(drive.join("groups/systems/nodes")).unwrap();
    std::fs::create_dir_all(drive.join("groups/l3_solar/nodes")).unwrap();
    // The system engineer's folder for what is being integrated.
    std::fs::create_dir_all(drive.join("integration")).unwrap();
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
    sqlite::write(
        &bare("l3_solar", "ivo_uid", "ivo_node", "Test Ivo"),
        &drive.join(IVO),
    )
    .unwrap();
    let mut sys = renamed(node(), "systems", "sys_node");
    sys.meta.insert(s("writer"), s("Test Uma"));
    sqlite::write(&sys, &drive.join(SYS)).unwrap();
    // A file in the programme's nodes for the block systems hangs from.
    sqlite::write(
        &bare("programme", "root", "root", "Test Sam"),
        &drive.join(ROOT),
    )
    .unwrap();
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

/// The moment the work goes back, in these tests: the day after the signing.
const NOW: &str = "2026-10-16T09:00:00Z";

/// A put-back from `copy` to `drive` as `me` at `at`, the chain opened from
/// the drive's own files.
fn pb_at(
    copy: &Path,
    drive: &Path,
    me: &str,
    at: &str,
    write: Option<Vec<(String, String)>>,
) -> Result<Report, vleo_files::error::Error> {
    let open = || {
        let o = opened(drive);
        Ok(Chain {
            anchor: o.anchor,
            programme: o.programme,
            systems: Some(o.systems),
            groups: o.groups,
            left_out: Vec::new(),
        })
    };
    put_back(copy, drive, me, at, &open, write.as_deref())
}

fn pb(
    copy: &Path,
    drive: &Path,
    me: &str,
    write: Option<Vec<(String, String)>>,
) -> Result<Report, vleo_files::error::Error> {
    pb_at(copy, drive, me, NOW, write)
}

/// `path` named as the report gives it: with the digest of its bytes in the
/// copy.
fn named(copy: &Path, path: &str) -> (String, String) {
    (s(path), opened_digest(&copy.join(path)))
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
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    assert_eq!(r.back(), [NODE], "{:?}", r.lines);
    assert!(line(&r, NODE).why.contains("signed by Test Nia"));
    assert!(!r.written);
    assert_ne!(
        std::fs::read(drive.join(NODE)).unwrap(),
        std::fs::read(copy.join(NODE)).unwrap()
    );
    // Named as the report says: written.
    let r = pb(&copy, &drive, "Test Nia", Some(vec![named(&copy, NODE)])).unwrap();
    assert!(r.written);
    assert_eq!(
        std::fs::read(drive.join(NODE)).unwrap(),
        std::fs::read(copy.join(NODE)).unwrap()
    );
    // The record says the file is as the drive holds it: nothing is left.
    let again = pb(&copy, &drive, "Test Nia", None).unwrap();
    assert!(again.lines.is_empty(), "{:?}", again.lines);
    // No file of the program's own is left on the drive.
    let left: Vec<_> = std::fs::read_dir(drive.join("groups/l3_solar/nodes"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(left.iter().all(|n| !n.starts_with(".vleo-")), "{left:?}");
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
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back && l.why.contains("changed on the drive since your copy"),
        "{l:?}"
    );
    let e = pb(&copy, &drive, "Test Nia", Some(vec![named(&copy, NODE)])).unwrap_err();
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
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back && l.why.contains("carries no current signature by Test Nia"),
        "{l:?}"
    );
    // Signed by Nia, it is still not Ivo's to put back.
    nia_signs(&copy);
    let r = pb(&copy, &drive, "Test Ivo", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back && l.why.contains("it brings a signature by Test Nia"),
        "{l:?}"
    );
    let before = std::fs::read(drive.join(NODE)).unwrap();
    let e = pb(&copy, &drive, "Test Ivo", Some(vec![named(&copy, NODE)])).unwrap_err();
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
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(!l.back && l.why.contains("revoked"), "{l:?}");
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_group_file_does_not_go_back_yet_whoever_changed_it() {
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
    let before = std::fs::read(drive.join(SOLAR)).unwrap();
    for who in ["Test Ola", "Test Nia"] {
        let r = pb(&copy, &drive, who, None).unwrap();
        let l = line(&r, SOLAR);
        assert!(
            !l.back && l.why.contains("l3_solar's group file does not go back yet"),
            "{who}: {l:?}"
        );
    }
    let e = pb(&copy, &drive, "Test Ola", Some(vec![named(&copy, SOLAR)])).unwrap_err();
    assert!(
        e.message().contains("nothing in your copy goes back"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(drive.join(SOLAR)).unwrap(), before);
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn only_the_files_the_report_names_are_written_and_nothing_is_deleted() {
    let (drive, copy) = world("named");
    nia_signs(&copy);
    std::fs::write(copy.join("notes.txt"), "a note of my own").unwrap();
    std::fs::remove_file(copy.join("START HERE.md")).unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
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
    for names in [vec![], vec![NODE, "notes.txt"], vec!["notes.txt"]] {
        let e = pb(
            &copy,
            &drive,
            "Test Nia",
            Some(names.iter().map(|p| named(&copy, p)).collect()),
        )
        .unwrap_err();
        assert!(
            e.message().contains("not the ones that go back now"),
            "{names:?}: {}",
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
    std::fs::write(drive.join(".vleo-put-back.lock"), "1").unwrap();
    let before = std::fs::read(drive.join(NODE)).unwrap();
    let e = pb(&copy, &drive, "Test Nia", Some(vec![named(&copy, NODE)])).unwrap_err();
    assert!(
        e.message().contains("someone is putting work back"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(drive.join(NODE)).unwrap(), before);
    std::fs::remove_file(drive.join(".vleo-put-back.lock")).unwrap();
    let elsewhere = drive.parent().unwrap().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let e = pb(&copy, &elsewhere, "Test Nia", None).unwrap_err();
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
    // A later revision, so the node's own checks are what is read.
    bad.meta.insert(s("revision"), s("2"));
    bad.blocks[0].revision = 2;
    let row = chain::sign(&bad, UID, "Test Nia", &key(74), AT, "ok", "").unwrap();
    bad.signatures.push(row);
    sqlite::write(&bad, &copy.join(NODE)).unwrap();
    let o = opened(&drive);
    chain::check_node(&bad, UID, &o.groups[0]).expect("its signature counts");
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
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
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(!l.back && l.why.contains("already on the drive"), "{l:?}");
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

/// `f` signed `ok` for `uid` by `who`, past any check: as a tool that signs
/// anything would.
fn signed_raw(mut f: File, uid: &str, who: &str, k: &SigningKey) -> File {
    let row = chain::sign(&f, uid, who, k, AT, "ok", "").unwrap();
    f.signatures.push(row);
    f
}

#[test]
fn a_node_s_file_never_goes_back_over_another_node_s() {
    let (drive, copy) = world("another");
    // Nia's own node, signed by her, written in place of Ivo's.
    nia_signs(&copy);
    let mut mine = sqlite::read(&copy.join(NODE)).unwrap();
    mine.blocks[0].id = s("ivo_node");
    let mine = signed_raw(mine, UID, "Test Nia", &key(74));
    std::fs::remove_file(copy.join(IVO)).unwrap();
    sqlite::write(&mine, &copy.join(IVO)).unwrap();
    let before = std::fs::read(drive.join(IVO)).unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, IVO);
    assert!(
        !l.back
            && l.why
                .contains("does not put one node's file in another's place"),
        "{l:?}"
    );
    assert_eq!(std::fs::read(drive.join(IVO)).unwrap(), before);
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_node_s_file_goes_back_changed_only_in_the_node_s_own_rows() {
    type Change = fn(&mut File);
    let cases: [(&str, Change, &str); 6] = [
        (
            "its block's contract version",
            |f| f.blocks[0].contract_version = 2,
            "changes the node's id, the block it hangs from or its contract's version",
        ),
        (
            "the block it hangs from",
            |f| f.blocks[0].parent_uid = s("elsewhere"),
            "changes the node's id, the block it hangs from or its contract's version",
        ),
        (
            "a person",
            |f| {
                f.people.push(Person {
                    name: s("Test Mal"),
                    role: s("subsystem engineer"),
                    deputy_for: s(""),
                })
            },
            "not sw_activity_band's own to change (person)",
        ),
        (
            "a key",
            |f| {
                f.people.push(Person {
                    name: s("Test Mal"),
                    role: s("node engineer"),
                    deputy_for: s(""),
                });
                f.keys.push(PersonKey {
                    person: s("Test Mal"),
                    public_key: key(99).public().to_hex(),
                    registered_at: s("2026-01-01"),
                    registered_by: s("Test Nia"),
                    revoked_from: s(""),
                    proof: s(""),
                })
            },
            "person_key",
        ),
        (
            "the contract's version",
            |f| {
                f.meta.insert(s("contract_version"), s("7"));
            },
            "not sw_activity_band's own to change (meta)",
        ),
        (
            "a revision its block does not say",
            |f| {
                f.meta.insert(s("revision"), s("99"));
            },
            "it says it is revision 99, and its block says 1",
        ),
    ];
    for (what, change, says) in cases {
        let (drive, copy) = world(&format!("own-{}", what.replace(' ', "-")));
        nia_signs(&copy);
        let mut f = sqlite::read(&copy.join(NODE)).unwrap();
        change(&mut f);
        sqlite::write(&f, &copy.join(NODE)).unwrap();
        let r = pb(&copy, &drive, "Test Nia", None).unwrap();
        let l = line(&r, NODE);
        assert!(!l.back && l.why.contains(says), "{what}: {l:?}");
        let _ = std::fs::remove_dir_all(drive.parent().unwrap());
    }
}

#[test]
fn an_older_revision_or_another_of_the_same_never_goes_back() {
    // The drive holds revision 2 when the copy is taken.
    let (drive, copy) = world("older");
    let mut two = node();
    two.meta.insert(s("revision"), s("2"));
    two.blocks[0].revision = 2;
    sqlite::write(&two, &drive.join(NODE)).unwrap();
    let _ = std::fs::remove_dir_all(&copy);
    take_copy(&drive, &copy);
    // Revision 1, signed, put in the copy.
    sqlite::write(
        &signed_raw(node(), UID, "Test Nia", &key(74)),
        &copy.join(NODE),
    )
    .unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back
            && l.why
                .contains("it is revision 1, and the drive holds revision 2"),
        "{l:?}"
    );
    // Revision 2 again, with other content, signed.
    let mut other = two.clone();
    other.texts.push(Text {
        scope: s(UID),
        kind: s("note"),
        body: s("not the drive's revision 2"),
    });
    sqlite::write(
        &signed_raw(other, UID, "Test Nia", &key(74)),
        &copy.join(NODE),
    )
    .unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back
            && l.why
                .contains("differs from it in more than Test Nia's signature"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[cfg(unix)]
#[test]
fn nothing_goes_back_through_a_link_on_the_drive() {
    let (drive, copy) = world("link");
    nia_signs(&copy);
    let outside = drive.parent().unwrap().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    let nodes = drive.join("groups/l3_solar/nodes");
    for e in std::fs::read_dir(&nodes).unwrap() {
        let e = e.unwrap();
        std::fs::copy(e.path(), outside.join(e.file_name())).unwrap();
    }
    std::fs::remove_dir_all(&nodes).unwrap();
    std::os::unix::fs::symlink(&outside, &nodes).unwrap();
    let before = std::fs::read(outside.join("sw_activity_band.vnode")).unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(!l.back && l.why.contains("reached through a link"), "{l:?}");
    let e = pb(&copy, &drive, "Test Nia", Some(vec![named(&copy, NODE)])).unwrap_err();
    assert!(
        e.message().contains("nothing in your copy goes back"),
        "{}",
        e.message()
    );
    assert_eq!(
        std::fs::read(outside.join("sw_activity_band.vnode")).unwrap(),
        before
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_key_revoked_on_the_drive_after_the_signing_still_puts_nothing_back() {
    let (drive, copy) = world("revoked-after");
    nia_signs(&copy);
    let o = opened(&drive);
    let (parent, _) = chain::mounted_by("l3_solar", &[&o.programme, &o.systems]).unwrap();
    let valve = Valve::Group {
        mounted_by: parent,
        programme: &o.programme,
    };
    // Revoked from the day after the signing: her signature still counts at
    // the moment it says it was made, and her key no longer counts now.
    let revoked = people::change(
        &sqlite::read(&drive.join(SOLAR)).unwrap(),
        &valve,
        "Test Ola",
        &key(73),
        "2026-10-16T08:00:00Z",
        &Change::Revoke {
            person: s("Test Nia"),
            public_key: key(74).public().to_hex(),
            from: s("2026-10-16"),
        },
    )
    .unwrap();
    sqlite::write(&revoked, &drive.join(SOLAR)).unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back
            && l.why
                .contains("does not count for them at 2026-10-16T09:00:00Z"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn the_systems_own_node_goes_back_and_a_mount_is_not_a_node() {
    let (drive, copy) = world("systems");
    let systems = opened(&copy).systems;
    let seen = opened_digest(&copy.join(SYS));
    sign_in_place(&copy, SYS, &seen, "Test Uma", &key(77), AT, APP, &systems).unwrap();
    let r = pb(&copy, &drive, "Test Uma", None).unwrap();
    assert_eq!(r.back(), [SYS], "{:?}", r.lines);
    // Sam is assigned the block systems hangs from, in the programme's file:
    // a "node" for it is not put back, whoever signs it.
    let root = signed_raw(
        sqlite::read(&copy.join(ROOT)).unwrap(),
        "root",
        "Test Sam",
        &key(72),
    );
    sqlite::write(&root, &copy.join(ROOT)).unwrap();
    let r = pb(&copy, &drive, "Test Sam", None).unwrap();
    let l = line(&r, ROOT);
    assert!(!l.back && l.why.contains("is where a group hangs"), "{l:?}");
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_node_s_file_out_of_its_place_or_not_one_block_is_refused_by_name() {
    let (drive, copy) = world("place");
    // Her node, signed, in another node's place.
    let mine = signed_raw(node(), UID, "Test Nia", &key(74));
    sqlite::write(&mine, &copy.join(IVO)).unwrap();
    // Two blocks, and none, where her node's file is.
    let (dr2, copy2) = world("two-blocks");
    let mut two = node();
    let mut other = two.blocks[0].clone();
    other.uid = s("x");
    other.id = s("x");
    two.blocks.push(other);
    sqlite::write(&two, &copy2.join(NODE)).unwrap();
    let (dr3, copy3) = world("no-block");
    let mut none = node();
    none.blocks.clear();
    none.ports.clear();
    none.wires.clear();
    sqlite::write(&none, &copy3.join(NODE)).unwrap();
    for (c, d, path, says) in [
        (&copy, &drive, IVO, "is not where the node it holds is kept"),
        (&copy2, &dr2, NODE, "holds 2 blocks"),
        (&copy3, &dr3, NODE, "holds 0 blocks"),
    ] {
        let r = pb(c, d, "Test Nia", None).unwrap();
        let l = line(&r, path);
        assert!(!l.back && l.why.contains(says), "{says}: {l:?}");
    }
    for d in [&drive, &dr2, &dr3] {
        let _ = std::fs::remove_dir_all(d.parent().unwrap());
    }
}

#[test]
fn a_file_already_on_the_drive_is_the_record_s_once_put_back() {
    let (drive, copy) = world("already-record");
    nia_signs(&copy);
    std::fs::copy(copy.join(NODE), drive.join(NODE)).unwrap();
    let r = pb(&copy, &drive, "Test Nia", Some(vec![])).unwrap();
    assert!(!r.written);
    assert_eq!(r.recorded, [NODE]);
    let again = pb(&copy, &drive, "Test Nia", None).unwrap();
    assert!(again.lines.is_empty(), "{:?}", again.lines);
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_file_named_with_another_digest_than_the_copy_s_is_not_written() {
    let (drive, copy) = world("digest");
    nia_signs(&copy);
    let before = std::fs::read(drive.join(NODE)).unwrap();
    let e = pb(
        &copy,
        &drive,
        "Test Nia",
        Some(vec![(s(NODE), "0".repeat(64))]),
    )
    .unwrap_err();
    assert!(
        e.message().contains("not the ones that go back now"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(drive.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[cfg(unix)]
#[test]
fn a_name_of_the_program_s_own_hides_nothing_in_a_folder() {
    let (drive, _copy) = world("own-names");
    std::fs::create_dir_all(drive.join(".vleo-folder")).unwrap();
    std::fs::write(drive.join(".vleo-folder/kept.txt"), "kept").unwrap();
    assert!(digests(&drive)
        .unwrap()
        .contains_key(".vleo-folder/kept.txt"));
    std::os::unix::fs::symlink("/etc", drive.join(".vleo-link")).unwrap();
    let e = digests(&drive).unwrap_err();
    assert!(
        e.message().contains(".vleo-link is a link"),
        "{}",
        e.message()
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_new_node_s_file_is_not_made_on_the_drive() {
    let (drive, copy) = world("new-node");
    let mut f = node();
    f.blocks[0].id = s("brand_new");
    let path = "groups/l3_solar/nodes/brand_new.vnode";
    sqlite::write(&signed_raw(f, UID, "Test Nia", &key(74)), &copy.join(path)).unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, path);
    assert!(
        !l.back && l.why.contains("the drive holds no node file at this path"),
        "{l:?}"
    );
    assert!(!drive.join(path).exists());
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

/// Nia's signed node put back, so the drive's file carries her signature.
fn nia_signed_on_the_drive(name: &str) -> (PathBuf, PathBuf) {
    let (drive, copy) = world(name);
    nia_signs(&copy);
    pb(&copy, &drive, "Test Nia", Some(vec![named(&copy, NODE)])).unwrap();
    (drive, copy)
}

#[test]
fn at_the_drive_s_revision_nothing_but_the_person_s_signature_goes_back() {
    type Change = fn(&mut File);
    let cases: [(&str, Change, &str); 4] = [
        (
            "a comment",
            |f| {
                f.comments.push(Comment {
                    scope: s(UID),
                    section: s(""),
                    author: s("Test Ola"),
                    at: s(AT),
                    body: s("approved"),
                    resolved: 0,
                })
            },
            "differs from it in more than Test Nia's signature",
        ),
        (
            "a request",
            |f| {
                f.requests.push(Request {
                    uid: s("r1"),
                    kind: s("contract"),
                    scope: s(UID),
                    raised_by: s("Test Ola"),
                    at: s(AT),
                    body: s("more inputs"),
                    status: s("open"),
                    answer: s(""),
                })
            },
            // Someone else's request: refused by name before the file is
            // compared (F3-d).
            "it brings the request r1 by Test Ola: only Test Nia's own",
        ),
        (
            "another writer",
            |f| {
                f.meta.insert(s("writer"), s("Test Ivo"));
            },
            "differs from it in more than Test Nia's signature",
        ),
        (
            "someone else's signature",
            |f| {
                let row = chain::sign(f, UID, "Test Ola", &key(73), AT, "ok", "").unwrap();
                f.signatures.push(row);
            },
            "it brings a signature by Test Ola",
        ),
    ];
    for (what, change, says) in cases {
        let (drive, copy) = nia_signed_on_the_drive(&format!("same-{}", what.replace(' ', "-")));
        let mut f = sqlite::read(&copy.join(NODE)).unwrap();
        change(&mut f);
        sqlite::write(&f, &copy.join(NODE)).unwrap();
        let r = pb(&copy, &drive, "Test Nia", None).unwrap();
        let l = line(&r, NODE);
        assert!(!l.back && l.why.contains(says), "{what}: {l:?}");
        let _ = std::fs::remove_dir_all(drive.parent().unwrap());
    }
}

#[test]
fn a_signature_the_drive_holds_is_never_taken_away() {
    let (drive, copy) = nia_signed_on_the_drive("taken-away");
    // Her revision 2, saved and signed, without her signature of revision 1.
    let g = opened(&copy).groups.remove(0);
    let mut edit = sqlite::read(&copy.join(NODE)).unwrap();
    edit.texts.retain(|t| !(t.scope == UID && t.kind == "note"));
    edit.texts.push(Text {
        scope: s(UID),
        kind: s("note"),
        body: s("revision 2"),
    });
    save(
        &copy,
        NODE,
        &edit,
        &opened_digest(&copy.join(NODE)),
        "Test Nia",
        APP,
        &g,
    )
    .unwrap();
    nia_signs(&copy);
    let mut f = sqlite::read(&copy.join(NODE)).unwrap();
    f.signatures.retain(|r| r.revision != "1");
    sqlite::write(&f, &copy.join(NODE)).unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back
            && l.why
                .contains("leaves out a signature by Test Nia that the drive's file holds"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_node_whose_writer_may_not_write_it_does_not_go_back() {
    let (drive, copy) = world("writer");
    let mut f = node();
    f.meta.insert(s("revision"), s("2"));
    f.meta.insert(s("writer"), s("Test Ivo"));
    f.blocks[0].revision = 2;
    sqlite::write(&signed_raw(f, UID, "Test Nia", &key(74)), &copy.join(NODE)).unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back && l.why.contains("\"Test Ivo\" wrote it, who may not write"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn one_signature_must_both_count_as_made_and_be_made_with_a_key_that_counts_now() {
    let (drive, copy) = world("two-keys");
    // Nia has a second key in solar, on the drive and so in the copy.
    let mut solar = sqlite::read(&drive.join(SOLAR)).unwrap();
    solar.keys.push(PersonKey {
        person: s("Test Nia"),
        public_key: key(78).public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Test Ola"),
        revoked_from: s(""),
        proof: vleo_files::people::proof(&key(78), "Test Nia"),
    });
    solar.signatures.clear();
    sqlite::write(&signed(solar, "Test Ola", &key(73)), &drive.join(SOLAR)).unwrap();
    let _ = std::fs::remove_dir_all(&copy);
    take_copy(&drive, &copy);
    // She signs with key 74; a stale row by key 78, of other content, rides
    // along.
    nia_signs(&copy);
    let mut f = sqlite::read(&copy.join(NODE)).unwrap();
    let mut other = node();
    other.texts.push(Text {
        scope: s(UID),
        kind: s("note"),
        body: s("other content"),
    });
    let stale = chain::sign(&other, UID, "Test Nia", &key(78), AT, "ok", "").unwrap();
    f.signatures.push(stale);
    sqlite::write(&f, &copy.join(NODE)).unwrap();
    // On the drive, key 74 is revoked from the day after the signing.
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
        "2026-10-16T08:00:00Z",
        &Change::Revoke {
            person: s("Test Nia"),
            public_key: key(74).public().to_hex(),
            from: s("2026-10-16"),
        },
    )
    .unwrap();
    sqlite::write(&revoked, &drive.join(SOLAR)).unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back && l.why.contains("does not count for them at"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn the_drive_s_lock_is_taken_before_its_chain_is_read_and_kept_if_not_its_own() {
    let (drive, copy) = world("lock-order");
    nia_signs(&copy);
    let lock = drive.join(".vleo-put-back.lock");
    let seen = std::cell::Cell::new(None);
    let open = || {
        seen.set(Some(lock.exists()));
        let o = opened(&drive);
        Ok(Chain {
            anchor: o.anchor,
            programme: o.programme,
            systems: Some(o.systems),
            groups: o.groups,
            left_out: Vec::new(),
        })
    };
    put_back(&copy, &drive, "Test Nia", NOW, &open, None).unwrap();
    assert_eq!(
        seen.get(),
        Some(false),
        "a report takes no lock on the drive"
    );
    // Someone else's word put in the lock while it is held: it is left there.
    let open_and_overwrite = || {
        seen.set(Some(lock.exists()));
        std::fs::write(&lock, "someone else's").unwrap();
        open()
    };
    let named = vec![named(&copy, NODE)];
    put_back(
        &copy,
        &drive,
        "Test Nia",
        NOW,
        &open_and_overwrite,
        Some(&named),
    )
    .unwrap();
    assert_eq!(
        seen.get(),
        Some(true),
        "the lock is held when the chain is read"
    );
    assert!(lock.exists(), "a lock that is not its own is never removed");
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn only_the_program_s_files_at_the_top_are_left_out_and_one_being_written_is_said() {
    let (drive, _copy) = world("skips");
    std::fs::write(drive.join("groups/l3_solar/.vleo-record-1"), "x").unwrap();
    std::fs::write(drive.join("groups/l3_solar/.vleo-put-back.lock"), "x").unwrap();
    std::fs::write(drive.join(".vleo-put-back.lock"), "x").unwrap();
    let d = digests(&drive).unwrap();
    assert!(d.contains_key("groups/l3_solar/.vleo-record-1"));
    assert!(d.contains_key("groups/l3_solar/.vleo-put-back.lock"));
    assert!(!d.contains_key(".vleo-put-back.lock"));
    std::fs::write(
        drive.join("groups/l3_solar/nodes/.vleo-incoming-1-0-x.vnode"),
        "x",
    )
    .unwrap();
    let e = digests(&drive).unwrap_err();
    assert!(
        e.message()
            .contains("is a file being put back on the drive"),
        "{}",
        e.message()
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

const ASK: &str = "The band's floor reads F10.7 as daily: it needs the 81-day mean.";
const HAND: &str = "sw_activity_band-r1";

/// Nia asks, in her node's file in the copy, for a contract change: the
/// request's uid.
fn nia_asks(copy: &Path) -> String {
    let g = opened(copy).groups.remove(0);
    let seen = opened_digest(&copy.join(NODE));
    match ask_in_place(
        copy,
        NODE,
        &seen,
        "Test Nia",
        &key(74),
        AT,
        "contract",
        ASK,
        APP,
        &g,
    )
    .unwrap()
    {
        Asked::Written { uid, .. } => uid,
        other => panic!("not asked: {other:?}"),
    }
}

#[test]
fn a_request_goes_back_by_itself_before_its_node_is_signed() {
    let (drive, copy) = world("asked");
    let uid = nia_asks(&copy);
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    assert_eq!(r.back(), [NODE], "{:?}", r.lines);
    assert!(
        line(&r, NODE)
            .why
            .contains("a request on the node sw_activity_band, asked and signed by Test Nia"),
        "{:?}",
        r.lines
    );
    let r = pb(&copy, &drive, "Test Nia", Some(vec![named(&copy, NODE)])).unwrap();
    assert!(r.written);
    // A copy taken again holds her request, and it counts there; the node is
    // still unsigned.
    let second = drive.parent().unwrap().join("second");
    take_copy(&drive, &second);
    let f = sqlite::read(&second.join(NODE)).unwrap();
    let g = opened(&second).groups.remove(0);
    chain::check_request(&f, &uid, &g).expect("the request counts on the drive");
    assert!(
        chain::check_node(&f, UID, &g).is_err(),
        "the node is not signed"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_request_and_a_signature_go_back_together() {
    let (drive, copy) = world("asked-and-signed");
    let uid = nia_asks(&copy);
    nia_signs(&copy);
    let r = pb(&copy, &drive, "Test Nia", Some(vec![named(&copy, NODE)])).unwrap();
    assert!(r.written, "{:?}", r.lines);
    assert!(
        line(&r, NODE).why.contains("signed by Test Nia"),
        "{:?}",
        r.lines
    );
    let f = sqlite::read(&drive.join(NODE)).unwrap();
    let g = opened(&drive).groups.remove(0);
    chain::check_request(&f, &uid, &g).unwrap();
    chain::check_node(&f, UID, &g).unwrap();
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

/// `f` with a request on `scope` by `who`, of `kind`, in state `status`,
/// signed by `signer` when one is given: written by hand, named as asking
/// names one (`HAND`).
fn by_hand(
    mut f: File,
    scope: &str,
    who: &str,
    kind: &str,
    status: &str,
    signer: Option<(&str, &SigningKey)>,
) -> File {
    f.requests.push(Request {
        uid: s(HAND),
        kind: s(kind),
        scope: s(scope),
        raised_by: s(who),
        at: s(AT),
        body: s(ASK),
        status: s(status),
        answer: s(""),
    });
    if let Some((name, k)) = signer {
        let row = chain::sign(&f, &chain::request_scope(HAND), name, k, AT, "ok", "").unwrap();
        f.signatures.push(row);
    }
    f
}

#[test]
fn a_request_goes_back_only_as_its_asker_s_own_signed_work() {
    type Make = fn(File) -> File;
    let nia = key(74);
    let _ = &nia;
    let cases: [(&str, Make, &str, &str); 5] = [
        (
            "unsigned",
            |f| by_hand(f, UID, "Test Nia", "contract", "open", None),
            "Test Nia",
            "does not count on the drive: the request sw_activity_band-r1 by Test Nia has no signature",
        ),
        (
            "of the code",
            |f| {
                by_hand(
                    f,
                    UID,
                    "Test Nia",
                    "code",
                    "open",
                    Some(("Test Nia", &key(74))),
                )
            },
            "Test Nia",
            "of \"code\"",
        ),
        (
            "answered",
            |f| {
                by_hand(
                    f,
                    UID,
                    "Test Nia",
                    "contract",
                    "accepted",
                    Some(("Test Nia", &key(74))),
                )
            },
            "Test Nia",
            "already answered (accepted)",
        ),
        (
            "of another node",
            |f| {
                by_hand(
                    f,
                    "ivo_uid",
                    "Test Nia",
                    "contract",
                    "open",
                    Some(("Test Nia", &key(74))),
                )
            },
            "Test Nia",
            "a node's file asks of its own node",
        ),
        (
            "someone else's",
            |f| {
                by_hand(
                    f,
                    UID,
                    "Test Nia",
                    "contract",
                    "open",
                    Some(("Test Nia", &key(74))),
                )
            },
            "Test Ivo",
            "it brings the request sw_activity_band-r1 by Test Nia: only Test Ivo's own",
        ),
    ];
    for (what, make, me, says) in cases {
        let (drive, copy) = world(&format!("request-{}", what.replace(' ', "-")));
        let f = make(sqlite::read(&copy.join(NODE)).unwrap());
        sqlite::write(&f, &copy.join(NODE)).unwrap();
        let r = pb(&copy, &drive, me, None).unwrap();
        let l = line(&r, NODE);
        assert!(!l.back && l.why.contains(says), "{what}: {l:?}");
        let before = std::fs::read(drive.join(NODE)).unwrap();
        assert!(
            pb(&copy, &drive, me, Some(vec![named(&copy, NODE)])).is_err(),
            "{what}"
        );
        assert_eq!(std::fs::read(drive.join(NODE)).unwrap(), before, "{what}");
        let _ = std::fs::remove_dir_all(drive.parent().unwrap());
    }
    // A request signed by someone else for the asker: not theirs.
    let (drive, copy) = world("request-signed-by-another");
    let f = by_hand(
        sqlite::read(&copy.join(NODE)).unwrap(),
        UID,
        "Test Nia",
        "contract",
        "open",
        Some(("Test Ivo", &key(75))),
    );
    sqlite::write(&f, &copy.join(NODE)).unwrap();
    let l = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&l, NODE);
    assert!(
        !l.back
            && l.why
                .contains("it brings a signature by Test Ivo of Test Nia's request"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_request_whose_key_is_revoked_on_the_drive_since_does_not_go_back() {
    let (drive, copy) = world("request-revoked");
    nia_asks(&copy);
    // On the drive, Ola revokes Nia's key from after she asked, before it
    // goes back.
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
            from: s("2026-10-16T00:00:00Z"),
        },
    )
    .unwrap();
    sqlite::write(&revoked, &drive.join(SOLAR)).unwrap();
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back
            && l.why
                .contains("does not count for them at 2026-10-16T09:00:00Z"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_request_on_the_drive_is_kept_as_it_was_asked() {
    type Change = fn(&mut File, &str);
    let cases: [(&str, Change, &str); 5] = [
        (
            "dropped",
            |f, _| f.requests.clear(),
            "changes or leaves out the request sw_activity_band-r1 by Test Nia",
        ),
        (
            "reworded",
            |f, _| f.requests[0].body.push('!'),
            "changes or leaves out the request sw_activity_band-r1 by Test Nia",
        ),
        (
            "answered by its asker",
            |f, _| f.requests[0].status = s("accepted"),
            "changes or leaves out the request sw_activity_band-r1 by Test Nia",
        ),
        (
            "unsigned",
            |f, uid| {
                f.signatures
                    .retain(|r| r.scope != chain::request_scope(uid))
            },
            "leaves out Test Nia's signature of the request sw_activity_band-r1",
        ),
        (
            "signed again",
            |f, uid| {
                let row = chain::sign(
                    f,
                    &chain::request_scope(uid),
                    "Test Nia",
                    &key(74),
                    NOW,
                    "ok",
                    "",
                )
                .unwrap();
                f.signatures.push(row)
            },
            "is signed once, when it is asked",
        ),
    ];
    for (what, change, says) in cases {
        let (drive, copy) = world(&format!("kept-{}", what.replace(' ', "-")));
        let uid = nia_asks(&copy);
        assert!(
            pb(&copy, &drive, "Test Nia", Some(vec![named(&copy, NODE)]))
                .unwrap()
                .written
        );
        // The drive holds her request; she signs her node, and then changes
        // the request by hand.
        nia_signs(&copy);
        let mut f = sqlite::read(&copy.join(NODE)).unwrap();
        change(&mut f, &uid);
        sqlite::write(&f, &copy.join(NODE)).unwrap();
        let r = pb(&copy, &drive, "Test Nia", None).unwrap();
        let l = line(&r, NODE);
        assert!(!l.back && l.why.contains(says), "{what}: {l:?}");
        let _ = std::fs::remove_dir_all(drive.parent().unwrap());
    }
    // Saved as a later revision and signed, the request is kept, and it goes
    // back.
    let (drive, copy) = world("kept-revision");
    let uid = nia_asks(&copy);
    assert!(
        pb(&copy, &drive, "Test Nia", Some(vec![named(&copy, NODE)]))
            .unwrap()
            .written
    );
    let g = opened(&copy).groups.remove(0);
    let mut edit = sqlite::read(&copy.join(NODE)).unwrap();
    edit.texts.push(Text {
        scope: s(UID),
        kind: s("note"),
        body: s("A line of Test Nia's own."),
    });
    let seen = opened_digest(&copy.join(NODE));
    save(&copy, NODE, &edit, &seen, "Test Nia", APP, &g).unwrap();
    nia_signs(&copy);
    let r = pb(&copy, &drive, "Test Nia", Some(vec![named(&copy, NODE)])).unwrap();
    assert!(r.written, "{:?}", r.lines);
    let f = sqlite::read(&drive.join(NODE)).unwrap();
    chain::check_request(&f, &uid, &opened(&drive).groups[0]).unwrap();
    assert_eq!(f.meta.get("revision").map(String::as_str), Some("2"));
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_request_carries_nothing_back_but_itself() {
    type Change = fn(&mut File);
    let cases: [(&str, Change, &str); 3] = [
        (
            "a text of the node",
            |f| {
                f.texts.push(Text {
                    scope: s(UID),
                    kind: s("note"),
                    body: s("A line nobody signed."),
                })
            },
            "differs from it in more than Test Nia's signature and requests",
        ),
        (
            "another writer",
            |f| {
                f.meta.insert(s("writer"), s("Test Ivo"));
            },
            "differs from it in more than Test Nia's signature and requests",
        ),
        (
            "a request not named as asking names one",
            |f| {
                let old = chain::request_scope(&f.requests[0].uid);
                f.requests[0].uid = s("r1");
                for row in f.signatures.iter_mut().filter(|r| r.scope == old) {
                    row.scope = chain::request_scope("r1");
                }
            },
            "is not named for the node it is on",
        ),
    ];
    for (what, change, says) in cases {
        let (drive, copy) = world(&format!("only-{}", what.replace(' ', "-")));
        nia_asks(&copy);
        let mut f = sqlite::read(&copy.join(NODE)).unwrap();
        change(&mut f);
        sqlite::write(&f, &copy.join(NODE)).unwrap();
        let r = pb(&copy, &drive, "Test Nia", None).unwrap();
        let l = line(&r, NODE);
        assert!(!l.back && l.why.contains(says), "{what}: {l:?}");
        let _ = std::fs::remove_dir_all(drive.parent().unwrap());
    }
}

#[test]
fn a_drive_signature_of_a_request_it_does_not_hold_is_never_dropped() {
    let (drive, copy) = world("orphan-signature");
    // The drive's file holds Nia's signature of a request, and not the
    // request: a copy taken now, in which she asks, would bring a request of
    // that name and lose the signature there.
    let g = opened(&drive).groups.remove(0);
    let (asked, uid) =
        chain::ask(&node(), UID, "contract", ASK, "Test Nia", &key(74), AT, &g).unwrap();
    let mut on_drive = sqlite::read(&drive.join(NODE)).unwrap();
    on_drive.signatures.extend(
        asked
            .signatures
            .iter()
            .filter(|r| r.scope == chain::request_scope(&uid))
            .cloned(),
    );
    sqlite::write(&on_drive, &drive.join(NODE)).unwrap();
    std::fs::remove_dir_all(&copy).unwrap();
    take_copy(&drive, &copy);
    assert_eq!(nia_asks(&copy), uid);
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back
            && l.why.contains(
                "holds a signature by Test Nia of a request of that name it does not hold"
            ),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_request_on_a_later_revision_goes_back_with_it_signed_and_says_so() {
    let (drive, copy) = world("later-unsigned");
    let g = opened(&copy).groups.remove(0);
    let mut edit = sqlite::read(&copy.join(NODE)).unwrap();
    edit.texts.push(Text {
        scope: s(UID),
        kind: s("note"),
        body: s("A line of Test Nia's own."),
    });
    let seen = opened_digest(&copy.join(NODE));
    save(&copy, NODE, &edit, &seen, "Test Nia", APP, &g).unwrap();
    nia_asks(&copy);
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back
            && l.why.contains("carries no current signature by Test Nia")
            && l.why.contains("Its requests go back by themselves only while the node is the drive's revision 1: sign revision 2"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_request_s_uid_is_its_own_whatever_the_file_s_table_allows() {
    let (drive, copy) = world("twice");
    let uid = nia_asks(&copy);
    // The file's table rebuilt by hand without its key, and a second row of
    // her signed request's name saying something she never asked.
    {
        let db = rusqlite::Connection::open(copy.join(NODE)).unwrap();
        db.execute_batch(&format!(
            "CREATE TABLE r2 AS SELECT * FROM request;
             DROP TABLE request;
             ALTER TABLE r2 RENAME TO request;
             INSERT INTO request SELECT uid, kind, scope, raised_by, at,
               'Something else entirely.', status, answer FROM request WHERE uid = '{uid}';"
        ))
        .unwrap();
    }
    let f = sqlite::read(&copy.join(NODE)).unwrap();
    assert_eq!(f.requests.len(), 2, "the reader takes both rows");
    let g = opened(&copy).groups.remove(0);
    let e = chain::check_request(&f, &uid, &g).unwrap_err();
    assert!(
        e.message().contains("more than one request"),
        "{}",
        e.message()
    );
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, NODE);
    assert!(
        !l.back
            && l.why
                .contains("it holds more than one request \"sw_activity_band-r1\""),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

// ── A group's release, sealed in the copy, goes back as a new file (F5) ──

/// Solar's release `version` on the drive and in a copy.
fn release_path(version: &str) -> String {
    format!("groups/l3_solar/releases/l3_solar-{version}.vleo")
}

/// Nia and Ivo sign their nodes in the copy, and Ola seals solar `version`
/// there: the path of the release in the copy.
fn sealed_in(copy: &Path, version: &str) -> String {
    let signs = |path: &str, who: &str, k: &SigningKey| {
        let g = opened(copy).groups.remove(0);
        let seen = opened_digest(&copy.join(path));
        match sign_in_place(copy, path, &seen, who, k, AT, APP, &g).unwrap() {
            Saved::Written { .. } => {}
            other => panic!("{who} did not sign: {other:?}"),
        }
    };
    // Ivo's node, written in the copy as a whole node, for him to sign.
    let mut ivo = renamed(node(), "l3_solar", "ivo_uid");
    ivo.blocks[0].id = s("ivo_node");
    ivo.meta.insert(s("writer"), s("Test Ivo"));
    sqlite::write(&ivo, &copy.join(IVO)).unwrap();
    signs(NODE, "Test Nia", &key(74));
    signs(IVO, "Test Ivo", &key(75));
    let o = opened(copy);
    let at = vleo_files::release::seal_in_place(
        copy,
        "l3_solar",
        version,
        "Test Ola",
        &key(73),
        AT,
        APP,
        &o.systems,
        &o.programme,
    )
    .unwrap();
    assert_eq!(at, release_path(version));
    at
}

#[test]
fn a_release_sealed_in_the_copy_goes_back_as_a_new_file_by_its_owner_alone() {
    let (drive, copy) = world("release");
    let rel = sealed_in(&copy, "1.0");
    // Nia's signed node is hers to put back, and the release is not.
    let r = pb(&copy, &drive, "Test Nia", None).unwrap();
    let l = line(&r, &rel);
    assert!(!l.back && l.why.contains("sealed by \"Test Ola\""), "{l:?}");
    // Ola's: the release goes back, and the nodes, signed by others, do not.
    let r = pb(&copy, &drive, "Test Ola", None).unwrap();
    assert_eq!(r.back(), std::slice::from_ref(&rel), "{:?}", r.lines);
    assert!(line(&r, &rel)
        .why
        .contains("l3_solar 1.0, sealed by Test Ola"));
    assert!(!drive.join("groups/l3_solar/releases").exists());
    let r = pb(&copy, &drive, "Test Ola", Some(vec![named(&copy, &rel)])).unwrap();
    assert!(r.written);
    assert_eq!(
        std::fs::read(drive.join(&rel)).unwrap(),
        std::fs::read(copy.join(&rel)).unwrap()
    );
    // On the drive it checks through the drive's chain.
    let o = opened(&drive);
    let f = sqlite::read(&drive.join(&rel)).unwrap();
    assert!(chain::check_release_in(&f, &o.systems, &o.programme)
        .unwrap()
        .holds());
    let left: Vec<_> = std::fs::read_dir(drive.join("groups/l3_solar/releases"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, ["l3_solar-1.0.vleo"]);
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_release_goes_back_only_after_the_drive_s_latest_and_is_never_written_over() {
    let (drive, copy) = world("release-race");
    let other = drive.parent().unwrap().join("other");
    take_copy(&drive, &other);
    // 1.0 sealed in the copy and put back.
    let rel = sealed_in(&copy, "1.0");
    pb(&copy, &drive, "Test Ola", Some(vec![named(&copy, &rel)])).unwrap();
    // In another copy taken before it, 1.0 sealed again, byte for byte the
    // drive's (the same people, keys and moment): said so, and nothing is
    // written. A 1.0 that differs is never written over (the test below).
    let on_drive = std::fs::read(drive.join(&rel)).unwrap();
    let again = sealed_in(&other, "1.0");
    let r = pb(&other, &drive, "Test Ola", None).unwrap();
    let l = line(&r, &again);
    assert!(
        !l.back && l.why.contains("already on the drive as it is in your copy"),
        "{l:?}"
    );
    assert_eq!(std::fs::read(drive.join(&rel)).unwrap(), on_drive);
    // 1.1, sealed beside the drive's 1.0 that copy never saw, does not go back.
    let beside = drive.parent().unwrap().join("beside");
    take_copy(&drive, &beside);
    std::fs::remove_dir_all(beside.join("groups/l3_solar/releases")).unwrap();
    let late = sealed_in(&beside, "1.1");
    let r = pb(&beside, &drive, "Test Ola", None).unwrap();
    let l = line(&r, &late);
    assert!(
        !l.back
            && l.why.contains(
                "it follows no release, and the drive's latest release of l3_solar is 1.0"
            ),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_release_changed_after_its_seal_does_not_go_back() {
    let (drive, copy) = world("release-changed");
    let rel = sealed_in(&copy, "1.0");
    let mut f = sqlite::read(&copy.join(&rel)).unwrap();
    let p = f.ports.iter_mut().find(|p| p.block_uid == UID).unwrap();
    p.symbol.push('x');
    sqlite::write(&f, &copy.join(&rel)).unwrap();
    let r = pb(&copy, &drive, "Test Ola", None).unwrap();
    let l = line(&r, &rel);
    assert!(
        !l.back && l.why.contains("does not check through the chain"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

// ── A group's answer to a release goes back as a new file (W7) ──

/// Pia answers solar's 1.0 for power, in the copy, once 1.0 is sealed and,
/// when `on_drive`, put back on the drive.
fn pia_answers(
    drive: &Path,
    copy: &Path,
    said: &vleo_files::answer::Said,
    on_drive: bool,
) -> String {
    let rel = sealed_in(copy, "1.0");
    if on_drive {
        let r = pb(copy, drive, "Test Ola", Some(vec![named(copy, &rel)])).unwrap();
        assert!(r.written, "{:?}", r.lines);
    }
    let o = opened(copy);
    vleo_files::answer::answer_in_place(
        copy,
        "l3_solar",
        "1.0",
        "l3_power",
        said,
        "Test Pia",
        &key(79),
        AT,
        APP,
        &o.systems,
        &o.programme,
    )
    .unwrap()
}

#[test]
fn an_answer_goes_back_as_a_new_file_by_the_person_who_wrote_it() {
    let (drive, copy) = world("answer");
    let rel = pia_answers(
        &drive,
        &copy,
        &vleo_files::answer::Said::Object(s("the array's margin falls below what power holds")),
        true,
    );
    // Not Ola's to put back.
    let r = pb(&copy, &drive, "Test Ola", None).unwrap();
    let l = line(&r, &rel);
    assert!(!l.back && l.why.contains("\"Test Pia\"'s answer"), "{l:?}");
    // Pia's: it goes back, a new folder and file in the drive's integration/.
    let r = pb(&copy, &drive, "Test Pia", None).unwrap();
    assert_eq!(r.back(), std::slice::from_ref(&rel), "{:?}", r.lines);
    assert!(line(&r, &rel)
        .why
        .contains("l3_power's answer to l3_solar 1.0"));
    let r = pb(&copy, &drive, "Test Pia", Some(vec![named(&copy, &rel)])).unwrap();
    assert!(r.written);
    assert_eq!(
        std::fs::read(drive.join(&rel)).unwrap(),
        std::fs::read(copy.join(&rel)).unwrap()
    );
    let o = opened(&drive);
    let f = sqlite::read(&drive.join(&rel)).unwrap();
    chain::check_answer(&f, &o.systems, &o.programme).unwrap();
    // One there already is never written over.
    let r = pb(&copy, &drive, "Test Pia", None).unwrap();
    assert!(!r.back().contains(&rel), "{:?}", r.lines);
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn an_answer_goes_back_only_to_a_release_the_drive_holds_into_its_integration_folder() {
    // Solar 1.0 sealed in the copy, never put back: there is nothing on the
    // drive it answers.
    let (drive, copy) = world("answer-no-release");
    let rel = pia_answers(&drive, &copy, &vleo_files::answer::Said::Fine, false);
    let r = pb(&copy, &drive, "Test Pia", None).unwrap();
    let l = line(&r, &rel);
    assert!(
        !l.back
            && l.why
                .contains("the drive holds no groups/l3_solar/releases/l3_solar-1.0.vleo"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
    // The drive's 1.0 is another release than the one answered.
    let (drive, copy) = world("answer-other-release");
    let rel = pia_answers(&drive, &copy, &vleo_files::answer::Said::Fine, true);
    let on_drive = drive.join("groups/l3_solar/releases/l3_solar-1.0.vleo");
    let mut other = sqlite::read(&on_drive).unwrap();
    other.texts.push(Text {
        scope: s(FILE),
        kind: s("note"),
        body: s("another release of that number"),
    });
    sqlite::write(&other, &on_drive).unwrap();
    let r = pb(&copy, &drive, "Test Pia", None).unwrap();
    let l = line(&r, &rel);
    assert!(
        !l.back && l.why.contains("it is not an answer to that release"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
    // A drive with no integration/: the system engineer makes it, not this.
    let (drive, copy) = world("answer-no-integration");
    let rel = pia_answers(&drive, &copy, &vleo_files::answer::Said::Fine, true);
    std::fs::remove_dir_all(drive.join("integration")).unwrap();
    let e = pb(&copy, &drive, "Test Pia", Some(vec![named(&copy, &rel)]));
    let said = format!("{e:?}");
    assert!(
        said.contains("the drive holds no integration/ folder"),
        "{said}"
    );
    assert!(!drive.join("integration").exists());
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn an_answer_changed_after_it_was_signed_does_not_go_back() {
    let (drive, copy) = world("answer-changed");
    let rel = pia_answers(&drive, &copy, &vleo_files::answer::Said::Fine, true);
    let mut f = sqlite::read(&copy.join(&rel)).unwrap();
    for t in f.texts.iter_mut().filter(|t| t.kind == "answer") {
        t.body = s("object");
    }
    f.texts.push(Text {
        scope: s(FILE),
        kind: s("because"),
        body: s("said by nobody who signed it"),
    });
    sqlite::write(&f, &copy.join(&rel)).unwrap();
    let r = pb(&copy, &drive, "Test Pia", None).unwrap();
    let l = line(&r, &rel);
    assert!(
        !l.back && l.why.contains("does not check through the chain"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_release_carrying_a_group_file_the_drive_does_not_hold_does_not_go_back() {
    let (drive, copy) = world("release-group");
    // Ola registers a person in her copy's group file, which the drive's
    // does not hold, and seals there.
    let mut g = sqlite::read(&copy.join(SOLAR)).unwrap();
    g.signatures.retain(|r| r.scope != FILE);
    register(
        &mut g,
        "Test Nia Two",
        "node engineer",
        &key(76),
        "Test Ola",
    );
    sqlite::write(&signed(g, "Test Ola", &key(73)), &copy.join(SOLAR)).unwrap();
    let rel = sealed_in(&copy, "1.0");
    let r = pb(&copy, &drive, "Test Ola", None).unwrap();
    let l = line(&r, &rel);
    assert!(
        !l.back
            && l.why
                .contains("its people are not the drive's group file's"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[test]
fn a_release_goes_back_only_as_one_sealed_whole_never_over_one_there() {
    let (drive, copy) = world("release-whole");
    let rel = sealed_in(&copy, "1.0");
    let sealed = sqlite::read(&copy.join(&rel)).unwrap();
    let dir = copy.join("groups/l3_solar/releases");
    let not_back = |name: &str, says: &str| {
        let r = pb(&copy, &drive, "Test Ola", None).unwrap();
        let l = line(&r, &format!("groups/l3_solar/releases/{name}"));
        assert!(
            !l.back && l.why.contains(says),
            "{name}: expected «{says}» in {l:?}"
        );
    };
    // Upgraded from before 1.0.
    let mut up = sealed.clone();
    up.meta.insert(s("upgraded_from"), s("format 1"));
    sqlite::write(&up, &dir.join("l3_solar-4.0.vleo")).unwrap();
    not_back("l3_solar-4.0.vleo", "a release from before 1.0");
    std::fs::remove_file(dir.join("l3_solar-4.0.vleo")).unwrap();
    // Another kind of file, and a name that is not the release's.
    sqlite::write(&node(), &dir.join("l3_solar-2.0.vleo")).unwrap();
    not_back("l3_solar-2.0.vleo", "releases/ holds releases only");
    std::fs::remove_file(dir.join("l3_solar-2.0.vleo")).unwrap();
    std::fs::copy(copy.join(&rel), dir.join("l3_solar-3.0.vleo")).unwrap();
    not_back("l3_solar-3.0.vleo", "so its name is l3_solar-1.0.vleo");
    std::fs::remove_file(dir.join("l3_solar-3.0.vleo")).unwrap();
    // Put back; then the copy's 1.0 changed: the drive's is never written over.
    pb(&copy, &drive, "Test Ola", Some(vec![named(&copy, &rel)])).unwrap();
    let mut changed = sealed.clone();
    changed.comments.push(Comment {
        scope: s(FILE),
        section: s(""),
        author: s("Test Ola"),
        at: s(AT),
        body: s("a word after the seal"),
        resolved: 0,
    });
    sqlite::write(&changed, &copy.join(&rel)).unwrap();
    let r = pb(&copy, &drive, "Test Ola", None).unwrap();
    let l = line(&r, &rel);
    assert!(
        !l.back && l.why.contains("a release is never written over"),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}

#[cfg(unix)]
#[test]
fn a_release_never_goes_back_through_a_link_on_the_drive() {
    let (drive, copy) = world("release-link");
    let rel = sealed_in(&copy, "1.0");
    let elsewhere = drive.parent().unwrap().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::os::unix::fs::symlink(&elsewhere, drive.join("groups/l3_solar/releases")).unwrap();
    let r = pb(&copy, &drive, "Test Ola", None).unwrap();
    let l = line(&r, &rel);
    assert!(!l.back && l.why.contains("through a link"), "{l:?}");
    let _ = std::fs::remove_dir_all(drive.parent().unwrap());
}
