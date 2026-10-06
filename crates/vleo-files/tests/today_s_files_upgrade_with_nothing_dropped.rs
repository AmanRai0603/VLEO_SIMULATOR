//! A group's file from before 1.0 opens in 1.0 with nothing dropped.
//!
//! docs/PLAN_1_0.md, phase C: "Solar 1.1 upgrades with nothing dropped, from a
//! copy committed as a test fixture, and its pre-1.0 sign-offs are anchored by
//! fingerprint". The fixture `tests/fixtures/l3_solar-1.1.vleo` is the release
//! file as the solar group sealed it in the group application, byte for byte.
//!
//! The expected fingerprint is not computed by this library: it is the one the
//! group application sealed with and the developer's intake recorded, in
//! `acceptances/l3_solar-1.1.toml`. Every other expectation here is a count or
//! a value read from the fixture by SQLite itself, not by the code under test.

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use vleo_files::format_1::{self, Old};
use vleo_files::keys::SigningKey;
use vleo_files::meta::{open_as, Kind};
use vleo_files::sqlite;
use vleo_files::upgrade::{self, Upgrade};
use vleo_files::ErrorKind;

/// The fingerprint Solar 1.1 was sealed with (acceptances/l3_solar-1.1.toml).
const SOLAR_1_1: &str = "0a9e8b038e926892dd30bf61e04aed61db76d09dddc01821cb5cc575f5acd9e1";

const HOW: Upgrade = Upgrade {
    app: "vleo 1.0.0 (test)",
    at: "2026-10-06T09:00:00Z",
};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/l3_solar-1.1.vleo")
}

/// A scratch copy of the fixture, for a test that changes it.
fn copy(name: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("vleo-upgrade-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("l3_solar-1.1.vleo");
    std::fs::copy(fixture(), &path).unwrap();
    (dir, path)
}

/// A count SQLite gives of the fixture itself.
fn count(sql: &str) -> usize {
    let db = Connection::open(fixture()).unwrap();
    db.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap() as usize
}

#[test]
fn the_fixture_is_solar_1_1_as_it_was_sealed() {
    let old = sqlite::read_format_1(&fixture()).unwrap();
    assert_eq!(old.meta["file_kind"], "release");
    assert_eq!(old.meta["version"], "1.1");
    assert_eq!(old.meta["fingerprint"], SOLAR_1_1);
    // The folder the file holds gives the fingerprint the page sealed it with:
    // the library's folder and fingerprint are the page's.
    let folder = old.folder();
    assert_eq!(format_1::fingerprint(&folder, None), SOLAR_1_1);
    // And every sign-off was given for exactly what the folder holds.
    assert_eq!(old.reviews.len(), count("SELECT count(*) FROM review"));
    for r in &old.reviews {
        let node = (r.scope != "group").then_some(r.scope.as_str());
        assert_eq!(
            format_1::fingerprint(&folder, node),
            r.fingerprint,
            "{}'s sign-off of {}",
            r.name,
            r.scope
        );
    }
}

#[test]
fn solar_1_1_upgrades_with_nothing_dropped() {
    let opened = sqlite::open(&fixture(), &HOW).unwrap();
    assert_eq!(opened.format, 1);
    let f = open_as(opened.file, Kind::GroupRelease).unwrap();

    // What it says of itself, by section 13.
    assert_eq!(f.meta["group_id"], "l3_solar");
    assert_eq!(f.meta["version"], "1.1");
    assert_eq!(f.meta["previous"], "1.0", "from the group's versions.csv");
    assert_eq!(f.meta["based_on"], "", "no design was released before 1.0");
    assert_eq!(f.meta["sealed_by"], "Aman Rai");
    assert_eq!(f.meta["fingerprint"], SOLAR_1_1);

    // Every row has a place.
    let nodes = count("SELECT count(*) FROM node");
    let inputs = count("SELECT count(*) FROM input");
    assert_eq!(f.blocks.len(), nodes);
    assert_eq!(
        f.ports.len(),
        nodes + inputs,
        "an output per node, and every input"
    );
    assert_eq!(f.wires.len(), inputs);
    assert_eq!(f.texts.len(), count("SELECT count(*) FROM doc"));
    assert_eq!(f.tables.len(), count("SELECT count(*) FROM tbl"));
    assert_eq!(f.media.len(), count("SELECT count(*) FROM media"));
    assert_eq!(f.signatures.len(), count("SELECT count(*) FROM review"));
    assert_eq!(f.changes.len(), count("SELECT count(*) FROM change"));
    assert_eq!(f.people.len(), count("SELECT count(*) FROM member"));
    assert_eq!(f.people[0].role, "subsystem engineer", "the group's owner");
    assert!(
        f.signatures.iter().all(|s| s.public_key.is_empty()),
        "names, not keys"
    );

    // Nothing dropped: given back, it is the file that was upgraded, row for
    // row, and its folder gives the fingerprint the people signed.
    let before = sqlite::read_format_1(&fixture()).unwrap();
    assert_eq!(upgrade::to_format_1(&f).unwrap(), before);
    assert_eq!(upgrade::pre_1_0_fingerprint(&f).unwrap().1, SOLAR_1_1);
    assert!(
        opened.said.iter().any(|s| s.contains("sign-off")),
        "{:?}",
        opened.said
    );
}

#[test]
fn the_upgraded_file_is_written_and_read_back_as_format_2() {
    let (dir, path) = copy("written");
    let f = sqlite::open(&path, &HOW).unwrap().file;
    let out = dir.join("again.vleo");
    sqlite::write(&f, &out).unwrap();
    let back = sqlite::read(&out).unwrap();
    assert_eq!(back, f);
    assert_eq!(upgrade::pre_1_0_fingerprint(&back).unwrap().1, SOLAR_1_1);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn saving_over_a_file_from_before_1_0_keeps_the_old_one_beside_it() {
    let (dir, path) = copy("kept");
    let original = std::fs::read(&path).unwrap();
    let f = sqlite::open(&path, &HOW).unwrap().file;
    // Opening touches nothing.
    assert_eq!(std::fs::read(&path).unwrap(), original);
    sqlite::write(&f, &path).unwrap();
    let kept = sqlite::kept_copy(&path, 1);
    assert_eq!(kept, dir.join("l3_solar-1.1.format-1.vleo"));
    assert_eq!(
        std::fs::read(&kept).unwrap(),
        original,
        "the old file, byte for byte"
    );
    assert_eq!(
        sqlite::read(&path).unwrap(),
        f,
        "and the file is format 2 now"
    );
    // Saved again, the kept copy is as it was.
    sqlite::write(&f, &path).unwrap();
    assert_eq!(std::fs::read(&kept).unwrap(), original);

    // A different file kept there already is never written over, and neither
    // is the file being saved.
    let (dir2, path2) = copy("kept-other");
    std::fs::write(sqlite::kept_copy(&path2, 1), b"something else").unwrap();
    let e = sqlite::write(&f, &path2).err().unwrap();
    assert_eq!(e.kind(), ErrorKind::Io);
    assert_eq!(std::fs::read(&path2).unwrap(), original);
    std::fs::remove_dir_all(&dir).unwrap();
    std::fs::remove_dir_all(&dir2).unwrap();
}

/// The copy no longer sealed: what a release is while it is assembled, so a
/// test may change what the seal covers.
fn unsealed(path: &Path) {
    Connection::open(path)
        .unwrap()
        .execute_batch("DELETE FROM meta WHERE key IN ('sealed', 'sealed_by', 'fingerprint');")
        .unwrap();
}

/// The fixture with what Solar 1.1 happens not to hold added to it: a
/// reviewer, a comment, an issue raised in the group application, a request,
/// two node engineers on one node, and a stage.
fn with_everything_a_group_writes(path: &Path) {
    unsealed(path);
    let db = Connection::open(path).unwrap();
    db.execute_batch(
        "INSERT INTO member (name, role) VALUES ('Ada Lovelace', 'reviewer'), ('Bo Chen', 'author');
         UPDATE node SET author = 'Bo Chen,Ada Lovelace', stage = 'draft' WHERE uid = 'sw_regime';
         INSERT INTO comment (scope, section, author, at, body, resolved)
           VALUES ('sw_regime', 'method', 'Ada Lovelace', '2026-10-04T10:00:00Z', 'Why 3 bands?', 0);
         INSERT INTO comment (scope, section, author, at, body, resolved)
           VALUES ('group', 'issue', 'Bo Chen', '2026-10-04T11:00:00Z',
                   '# The flux plot\n\n- raised by: Bo Chen\n- on: 2026-10-04\n- folder version: 1.1\n- where: figures/speed\n\nThe axis has no unit.\n', 1);
         INSERT INTO request (node_uid, author, at, body, status, answer)
           VALUES ('sw_regime', 'Bo Chen', '2026-10-04T12:00:00Z', 'An input for the season', 'declined', 'Not in 1.1');",
    )
    .unwrap();
}

#[test]
fn comments_issues_requests_and_every_member_are_kept() {
    let (dir, path) = copy("everything");
    with_everything_a_group_writes(&path);
    let before = sqlite::read_format_1(&path).unwrap();
    let f = sqlite::open(&path, &HOW).unwrap().file;

    assert_eq!(f.comments.len(), 1);
    assert_eq!(f.issues.len(), 1, "the issue is an issue in format 2");
    assert_eq!(f.issues[0].place, "figures/speed");
    assert_eq!(f.issues[0].addressed_to, "Aman Rai", "the group's owner");
    assert!(!f.issues[0].closed_by.is_empty(), "it was resolved");
    assert_eq!(f.requests.len(), 1);
    assert_eq!(f.requests[0].kind, "contract");
    let ada = f.people.iter().find(|p| p.name == "Ada Lovelace").unwrap();
    assert_eq!(ada.role, "node engineer");
    let on_regime: Vec<&str> = f
        .assignments
        .iter()
        .filter(|a| a.block_uid == "sw_regime")
        .map(|a| a.person.as_str())
        .collect();
    assert_eq!(on_regime, ["Bo Chen", "Ada Lovelace"]);

    // Given back, every row is the one it was — the reviewer still a
    // reviewer, the author list as it was spelt, the issue a comment again.
    let mut back = upgrade::to_format_1(&f).unwrap();
    let mut was: Old = before;
    let key = |c: &format_1::Comment| (c.scope.clone(), c.section.clone(), c.at.clone());
    back.comments.sort_by_key(key);
    was.comments.sort_by_key(key);
    assert_eq!(back, was);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn what_cannot_come_back_is_not_upgraded() {
    // A comment resolved as 2: format 2 keeps an issue open or closed, so the
    // upgrade would lose the 2, and refuses rather than lose it.
    let (dir, path) = copy("lossy");
    Connection::open(&path)
        .unwrap()
        .execute_batch(
            "INSERT INTO comment (scope, section, author, at, body, resolved)
               VALUES ('group', 'issue', 'Bo Chen', '2026-10-04T11:00:00Z', 'x', 2);",
        )
        .unwrap();
    let e = sqlite::open(&path, &HOW).err().unwrap();
    assert_eq!(e.kind(), ErrorKind::Malformed);
    assert!(
        e.message()
            .contains("without losing its comments and issues"),
        "{e}"
    );

    // An input from `<a node of this file>.<anything>`: format 2 would read it
    // as that node's port.
    let (dir2, path2) = copy("ambiguous");
    unsealed(&path2);
    Connection::open(&path2)
        .unwrap()
        .execute_batch("UPDATE input SET source = 'sw_regime.other' WHERE rowid = 1;")
        .unwrap();
    let e = sqlite::open(&path2, &HOW).err().unwrap();
    assert!(e.message().contains("would read as a port"), "{e}");
    std::fs::remove_dir_all(&dir).unwrap();
    std::fs::remove_dir_all(&dir2).unwrap();
}

#[test]
fn a_release_changed_after_its_seal_is_not_upgraded() {
    let (dir, path) = copy("tampered");
    Connection::open(&path)
        .unwrap()
        .execute_batch(
            "UPDATE doc SET body = body || ' ' WHERE scope = 'sw_regime' AND kind = 'pseudocode';",
        )
        .unwrap();
    let e = sqlite::open(&path, &HOW).err().unwrap();
    assert_eq!(e.kind(), ErrorKind::Tampered);
    assert!(
        e.message()
            .contains("does not give the fingerprint it was sealed with"),
        "{e}"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_programme_manager_anchors_a_release_from_before_1_0_by_its_fingerprint() {
    let f = sqlite::open(&fixture(), &HOW).unwrap().file;
    let pm = SigningKey::from_seed([7; 32]);
    let row = upgrade::anchor(&f, "Pat Morgan", &pm, "2026-12-01").unwrap();
    assert_eq!(row.digest, SOLAR_1_1);
    assert_eq!(row.scope, "release l3_solar");
    assert_eq!(row.revision, "1.1");
    upgrade::check_anchor(&f, std::slice::from_ref(&row), &pm.public()).unwrap();

    // Not anchored: nothing signed, or signed by someone else's key.
    let e = upgrade::check_anchor(&f, &[], &pm.public()).err().unwrap();
    assert_eq!(e.kind(), ErrorKind::Signature);
    let other = SigningKey::from_seed([8; 32]);
    let e = upgrade::check_anchor(&f, std::slice::from_ref(&row), &other.public())
        .err()
        .unwrap();
    assert_eq!(e.kind(), ErrorKind::Signature);

    // An anchor whose signature is not the programme manager's over this
    // fingerprint does not check.
    let mut forged = row.clone();
    forged.signature = other
        .sign(&upgrade::anchor_message("l3_solar", "1.1", SOLAR_1_1))
        .to_hex();
    let e = upgrade::check_anchor(&f, &[forged], &pm.public())
        .err()
        .unwrap();
    assert_eq!(e.kind(), ErrorKind::Signature);

    // The release changed after the upgrade: the anchor no longer holds.
    let mut changed = f.clone();
    let t = changed
        .texts
        .iter_mut()
        .find(|t| t.kind == "pseudocode")
        .unwrap();
    t.body.push(' ');
    let e = upgrade::check_anchor(&changed, std::slice::from_ref(&row), &pm.public())
        .err()
        .unwrap();
    assert_eq!(e.kind(), ErrorKind::Tampered);
    assert_eq!(
        upgrade::anchor(&changed, "Pat Morgan", &pm, "2026-12-01")
            .err()
            .unwrap()
            .kind(),
        ErrorKind::Tampered
    );

    // Only a release is anchored.
    let mut group = f.clone();
    group.meta.insert("file_kind".into(), "group".into());
    let e = upgrade::anchor(&group, "Pat Morgan", &pm, "2026-12-01")
        .err()
        .unwrap();
    assert_eq!(e.kind(), ErrorKind::WrongKind);
}

#[test]
fn a_file_in_this_format_opens_as_it_is_and_is_not_given_back_as_format_1() {
    let (dir, path) = copy("current");
    let f = sqlite::open(&path, &HOW).unwrap().file;
    let mut plain = vleo_files::model::File::new(Kind::Daily, "vleo 1.0.0 (test)");
    plain.meta.insert("date".into(), "2026-10-06".into());
    plain.meta.insert("based_on".into(), String::new());
    let out = dir.join("daily.vleo");
    sqlite::write(&plain, &out).unwrap();
    let opened = sqlite::open(&out, &HOW).unwrap();
    assert_eq!(opened.format, 2);
    assert!(opened.said.is_empty());
    assert_eq!(opened.file, plain);
    let e = upgrade::to_format_1(&plain).err().unwrap();
    assert_eq!(e.kind(), ErrorKind::WrongKind);
    // A format-2 file read as format 1 is refused by name.
    sqlite::write(&f, &out).unwrap();
    assert_eq!(
        sqlite::read_format_1(&out).err().unwrap().kind(),
        ErrorKind::Format
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_computed_node_has_its_inputs_table_even_with_no_inputs() {
    // The page writes inputs.csv for a node with inputs, and for a computed
    // node with none, its header alone (web/js/gdb.js, folderFromDb); a
    // declared node with none has no inputs.csv. Solar 1.1 has neither case.
    let (dir, path) = copy("no-inputs");
    unsealed(&path);
    Connection::open(&path)
        .unwrap()
        .execute_batch(
            "INSERT INTO node (uid, id, kind, output, ord) VALUES ('lonely', 'lonely', 'computed', 'x', 99),
                                                               ('stated', 'stated', 'declared', 'y', 100);",
        )
        .unwrap();
    let folder = sqlite::read_format_1(&path).unwrap().folder();
    assert_eq!(
        folder
            .get("nodes/lonely/inputs.csv")
            .map(|b| String::from_utf8_lossy(b).into_owned()),
        Some("name,from,unit,default,min,max,says\n".to_string())
    );
    assert!(!folder.contains_key("nodes/stated/inputs.csv"));
    // And it upgrades, and comes back, as every file does.
    let f = sqlite::open(&path, &HOW).unwrap().file;
    assert_eq!(upgrade::to_format_1(&f).unwrap().folder(), folder);
    std::fs::remove_dir_all(&dir).unwrap();
}
