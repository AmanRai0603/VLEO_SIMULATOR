//! The one schema, held to itself and to the operating model.
//!
//! The Rust records and `schema.sql` say the same thing, column for column
//! and value for value; a file written is the file read back; every kind of
//! file says what section 10 and section 13 of docs/OPERATING_1_0.md say it
//! must; and a file opened as the wrong kind, or in a format this library does
//! not read, is refused by name.

use std::collections::{BTreeMap, BTreeSet};

use rusqlite::Connection;
use vleo_files::meta::{open_as, DesignVersion, Kind, ReleaseVersion, Rule};
use vleo_files::model::*;
use vleo_files::sqlite::{self, SCHEMA};
use vleo_files::ErrorKind;

fn schema_db() -> Connection {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch(SCHEMA).unwrap();
    db
}

#[test]
fn every_table_in_the_schema_is_a_record_with_the_same_columns() {
    let db = schema_db();
    let mut st = db
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .unwrap();
    let in_schema: BTreeSet<String> = st
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let tables = File::default().to_tables();
    let in_model: BTreeSet<String> = tables.iter().map(|t| t.name.clone()).collect();
    assert_eq!(in_schema, in_model, "the schema's tables and the model's");
    for t in tables {
        let mut st = db
            .prepare(&format!("PRAGMA table_info(\"{}\")", t.name))
            .unwrap();
        let cols: Vec<String> = st
            .query_map([], |r| r.get(1))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(cols, t.columns, "the columns of {}", t.name);
    }
}

/// Every `CHECK (column IN (…))` in the schema, as (table, column) → values.
fn checks_in_schema() -> BTreeMap<(String, String), BTreeSet<String>> {
    let mut out = BTreeMap::new();
    let mut table = String::new();
    let flat = SCHEMA.replace('\n', " ");
    for part in flat.split("CREATE TABLE ").skip(1) {
        table = part.split_whitespace().next().unwrap().to_string();
        let mut rest = part;
        while let Some(i) = rest.find("CHECK (") {
            let after = &rest[i + 7..];
            let column = after.split_whitespace().next().unwrap().to_string();
            let open = after.find("IN (").unwrap() + 4;
            let close = open + after[open..].find(')').unwrap();
            let values = after[open..close]
                .split(',')
                .map(|v| v.trim().trim_matches('\'').to_string())
                .collect();
            out.insert((table.clone(), column), values);
            rest = &after[close..];
        }
    }
    assert!(!table.is_empty());
    out
}

#[test]
fn the_values_the_library_allows_are_the_values_the_schema_allows() {
    let schema = checks_in_schema();
    let model: BTreeMap<(String, String), BTreeSet<String>> = ALLOWED
        .iter()
        .map(|(t, c, v)| {
            (
                (t.to_string(), c.to_string()),
                v.iter().map(|s| s.to_string()).collect(),
            )
        })
        .collect();
    assert_eq!(model, schema);
}

/// A file with a row in every table, every constrained column holding an
/// allowed value.
fn full_file() -> File {
    let mut f = File::new(Kind::Node, "vleo-files tests");
    for (k, v) in [
        ("group_id", "solar"),
        ("block_uid", "b-1"),
        ("revision", "3"),
        ("contract_version", "2"),
        ("writer", "Ana Example"),
    ] {
        f.meta.insert(k.into(), v.into());
    }
    let s = |x: &str| x.to_string();
    f.people.push(Person {
        name: s("Ana Example"),
        role: s("node engineer"),
        deputy_for: s(""),
    });
    f.keys.push(PersonKey {
        person: s("Ana Example"),
        public_key: s("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"),
        registered_at: s("2026-10-06"),
        registered_by: s("Aman Rai"),
        revoked_from: s(""),
    });
    f.assignments.push(Assignment {
        block_uid: s("b-1"),
        person: s("Ana Example"),
        contract_version: 2,
    });
    f.blocks.push(Block {
        uid: s("b-1"),
        id: s("sw_ap_design"),
        parent_uid: s(""),
        question: s("What Ap must the design survive?"),
        behaviour: s("method"),
        perspective: s("subsystem"),
        ord: 1,
        archived: 0,
        contract_version: 2,
        revision: 3,
    });
    f.ports.push(Port {
        block_uid: s("b-1"),
        direction: s("out"),
        name: s("ap"),
        symbol: s("Ap"),
        port_type: s("number"),
        unit: s("nT"),
        lower: s("0"),
        upper: s("400"),
        range_reason: s("Ap is defined on 0 to 400"),
        state: s("achieved"),
        maturity: s("calculated"),
        value: s(""),
        choices: s(""),
        bundle: s("drivers"),
        open_owner: s(""),
        open_due: s(""),
        says: s("the design Ap"),
        ord: 0,
    });
    f.wires.push(Wire {
        to_block: s("b-1"),
        to_port: s("cycle"),
        from_ref: s("case"),
    });
    f.mounts.push(Mount {
        block_uid: s("b-1"),
        group_id: s("drivers"),
        release: s("1.0"),
    });
    f.closures.push(Closure {
        uid: s("c-1"),
        block_uid: s("b-1"),
        required_ref: s("b-1.ap_required"),
        achieved_ref: s("b-1.ap"),
        sense: s("<="),
        says: s("the sky stays under what the design sustains"),
    });
    f.loops.push(Loop {
        uid: s("l-1"),
        block_uid: s("b-1"),
        members: s("b-2 b-3"),
        settles: s("b-2.temperature"),
        tolerance: s("0.01 K"),
        max_iterations: 50,
    });
    f.cases.push(TestCase {
        block_uid: s("b-1"),
        name: s("quiet sky"),
        inputs: s("cycle=0.2"),
        expected: s("12"),
        tolerance: s("1e-6"),
        provenance: s("published-source"),
        source: s("NOAA SWPC"),
    });
    f.texts.push(Text {
        scope: s("b-1"),
        kind: s("explanation"),
        body: s("Said simply: …"),
    });
    f.tables.push(Tbl {
        scope: s("b-1"),
        path: s("results/isolation.csv"),
        csv: s("a,b\n1,2\n"),
    });
    f.media.push(Media {
        scope: s("b-1"),
        path: s("figures/ap.png"),
        media_type: s("image/png"),
        sha256: s("0".repeat(64).as_str()),
        bytes: vec![0, 1, 2, 255],
    });
    f.signatures.push(SignatureRow {
        scope: s("b-1"),
        revision: s("3"),
        signer: s("Ana Example"),
        public_key: s("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"),
        digest: s("0".repeat(64).as_str()),
        signature: s("0".repeat(128).as_str()),
        signed_at: s("2026-10-06T08:00:00Z"),
        verdict: s("ok"),
        note: s(""),
    });
    f.changes.push(Change {
        at: s("2026-10-06T08:00:00Z"),
        who: s("Ana Example"),
        scope: s("b-1"),
        what: s("question"),
        before: None,
        after: Some(s("What Ap must the design survive?")),
    });
    f.requests.push(Request {
        uid: s("r-1"),
        kind: s("contract"),
        scope: s("b-1"),
        raised_by: s("Ana Example"),
        at: s("2026-10-06"),
        body: s("widen the range"),
        status: s("open"),
        answer: s(""),
    });
    f.issues.push(Issue {
        number: 7,
        group_id: s("solar"),
        raised_by: s("Ana Example"),
        at: s("2026-10-06"),
        place: s("sw_ap_design"),
        what: s("the margin is negative"),
        evidence: s("health map, 6 October"),
        addressed_to: s("propulsion"),
        closed_by: s(""),
    });
    f.comments.push(Comment {
        scope: s("b-1"),
        section: s("theory"),
        author: s("Ana Example"),
        at: s("2026-10-06"),
        body: s("check the 1989 storm"),
        resolved: 0,
    });
    f.locked_keys.push(LockedKeyRow {
        person: s("Ana Example"),
        public_key: s("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"),
        salt: s("00"),
        nonce: s("00"),
        iterations: 600_000,
        sealed: s("00"),
    });
    f
}

#[test]
fn a_file_written_is_the_file_read_back() {
    let f = full_file();
    assert!(
        f.to_tables().iter().all(|t| !t.rows.is_empty()),
        "a row in every table"
    );
    let dir = std::env::temp_dir().join(format!("vleo-files-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("b-1.vnode");
    sqlite::write(&f, &path).unwrap();
    let back = sqlite::read(&path).unwrap();
    assert_eq!(back, f);
    // And through the rows alone, as the page hands them.
    assert_eq!(File::from_tables(f.to_tables()).unwrap(), f);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_database_itself_refuses_a_value_the_schema_does_not_allow() {
    let db = schema_db();
    let e = db.execute(
        "INSERT INTO closure (uid, block_uid, required_ref, achieved_ref, sense) VALUES ('c', 'b', 'r', 'a', '=')",
        [],
    );
    assert!(e.is_err(), "a closure that binds neither way");
}

#[test]
fn the_library_refuses_a_value_the_schema_does_not_allow() {
    let mut f = full_file();
    f.closures[0].sense = "=".into();
    let e = File::from_tables(f.to_tables()).err().unwrap();
    assert_eq!(e.kind(), ErrorKind::Malformed);
    assert!(e.message().contains("closure.sense"), "{}", e.message());
}

#[test]
fn a_table_or_a_column_the_schema_does_not_have_is_refused() {
    let mut tables = full_file().to_tables();
    tables.push(Table {
        name: "extra".into(),
        columns: vec![],
        rows: vec![],
    });
    assert!(File::from_tables(tables)
        .err()
        .unwrap()
        .message()
        .contains("extra"));
    let mut tables = full_file().to_tables();
    let t = tables.iter_mut().find(|t| t.name == "wire").unwrap();
    t.columns.swap(0, 1);
    assert!(File::from_tables(tables)
        .err()
        .unwrap()
        .message()
        .contains("wire"));
}

#[test]
fn every_kind_says_what_it_must() {
    for kind in Kind::ALL {
        let f = File::new(kind, "tests");
        let e = f.check_meta().err().unwrap();
        for (key, _) in kind.required() {
            assert!(e.message().contains(key), "{kind}: {}", e.message());
        }
        assert_eq!(Kind::from_name(kind.name()), Some(kind));
    }
    assert_eq!(full_file().check_meta().unwrap(), Kind::Node);
}

#[test]
fn a_value_that_breaks_its_rule_is_named() {
    let mut f = full_file();
    f.meta.insert("revision".into(), "0".into());
    let e = f.check_meta().err().unwrap();
    assert!(e.message().contains("revision is \"0\""), "{}", e.message());
}

#[test]
fn a_group_release_is_never_taken_for_a_released_design() {
    let mut release = File::new(Kind::GroupRelease, "tests");
    for (k, v) in [
        ("group_id", "solar"),
        ("version", "1.2"),
        ("previous", "1.1"),
        ("based_on", "2026.10.1"),
        ("sealed_by", "Aman Rai"),
        ("fingerprint", &"a".repeat(64)[..]),
    ] {
        release.meta.insert(k.into(), v.into());
    }
    let e = open_as(release.clone(), Kind::Design).err().unwrap();
    assert_eq!(e.kind(), ErrorKind::WrongKind);
    assert_eq!(
        e.message(),
        "this is a group release, not a released design"
    );
    assert!(open_as(release, Kind::GroupRelease).is_ok());
}

#[test]
fn a_file_that_says_no_kind_or_an_unknown_one_is_refused() {
    let mut f = full_file();
    f.meta.remove("file_kind");
    assert!(f
        .check_meta()
        .err()
        .unwrap()
        .message()
        .contains("does not say what kind"));
    f.meta.insert("file_kind".into(), "spreadsheet".into());
    assert!(f
        .check_meta()
        .err()
        .unwrap()
        .message()
        .contains("spreadsheet"));
}

#[test]
fn a_file_newer_than_the_library_or_not_a_vleo_file_is_refused() {
    let dir = std::env::temp_dir().join(format!("vleo-files-fmt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let newer = dir.join("newer.vleo");
    let db = Connection::open(&newer).unwrap();
    db.execute_batch(SCHEMA).unwrap();
    db.execute_batch("PRAGMA user_version = 3;").unwrap();
    drop(db);
    let e = sqlite::read(&newer).err().unwrap();
    assert_eq!(e.kind(), ErrorKind::Format);
    assert!(e.message().contains("newer"));
    let older = dir.join("older.vleo");
    let db = Connection::open(&older).unwrap();
    db.execute_batch(
        "PRAGMA application_id = 1447838031; PRAGMA user_version = 1;
         CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO meta VALUES ('file_kind', 'release');",
    )
    .unwrap();
    drop(db);
    assert!(sqlite::read(&older)
        .err()
        .unwrap()
        .message()
        .contains("upgraded"));
    // One that does not say what it is is not guessed at.
    let unsaid = dir.join("unsaid.vleo");
    let db = Connection::open(&unsaid).unwrap();
    db.execute_batch("PRAGMA application_id = 1447838031; PRAGMA user_version = 1;")
        .unwrap();
    drop(db);
    let e = sqlite::read(&unsaid).err().unwrap();
    assert_eq!(e.kind(), ErrorKind::WrongKind);
    assert!(
        e.message().contains("does not say what kind it is"),
        "{}",
        e.message()
    );
    let other = dir.join("other.db");
    Connection::open(&other)
        .unwrap()
        .execute_batch("CREATE TABLE t (x);")
        .unwrap();
    assert_eq!(
        sqlite::read(&other).err().unwrap().kind(),
        ErrorKind::WrongKind
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

// ── Versions, section 13 ─────────────────────────────────────────────────────

#[test]
fn a_release_is_major_when_others_read_a_change_and_minor_otherwise() {
    let v = ReleaseVersion::parse("1.2").unwrap();
    assert_eq!(v.next(false).to_string(), "1.3");
    assert_eq!(v.next(true).to_string(), "2.0");
    for bad in ["", "1", "1.2.3", "01.2", "a.b", "0.0", "-1.2"] {
        assert!(ReleaseVersion::parse(bad).is_none(), "{bad:?}");
    }
}

#[test]
fn a_design_is_year_month_sequence() {
    let v = DesignVersion::parse("2026.11.2").unwrap();
    assert_eq!(v.next(2026, 11).to_string(), "2026.11.3");
    assert_eq!(v.next(2026, 12).to_string(), "2026.12.1");
    for bad in ["2026.13.1", "2026.11.0", "26.11.1", "2026.11", "2026.011.1"] {
        assert!(DesignVersion::parse(bad).is_none(), "{bad:?}");
    }
}

#[test]
fn the_rules_hold_what_they_say() {
    assert!(Rule::Count.holds("1") && !Rule::Count.holds("0") && !Rule::Count.holds("01"));
    assert!(
        Rule::Date.holds("2026-10-06")
            && !Rule::Date.holds("2026-13-06")
            && !Rule::Date.holds("6 Oct")
    );
    assert!(Rule::Application.holds("1.0.0") && !Rule::Application.holds("1.0"));
    assert!(Rule::Digest.holds(&"f".repeat(64)) && !Rule::Digest.holds("abc"));
    assert!(Rule::DesignOrNone.holds("") && !Rule::Design.holds(""));
}

#[test]
fn a_key_file_keeps_a_key_that_still_unlocks() {
    use vleo_files::keys::{LockedKey, SigningKey, MIN_ITERATIONS};
    let key = SigningKey::from_seed([3u8; 32]);
    let locked = key
        .lock(
            "Ana Example",
            "pass phrase",
            [4u8; 16],
            [5u8; 12],
            MIN_ITERATIONS,
        )
        .unwrap();
    let mut file = File::new(Kind::Key, "tests");
    file.meta.insert("person".into(), "Ana Example".into());
    file.locked_keys.push(locked.to_row());
    let dir = std::env::temp_dir().join(format!("vleo-files-key-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("ana.vkey");
    sqlite::write(&file, &path).unwrap();
    let back = open_as(sqlite::read(&path).unwrap(), Kind::Key).unwrap();
    let again = LockedKey::from_row(&back.locked_keys[0]).unwrap();
    assert_eq!(again, locked);
    assert_eq!(again.unlock("pass phrase").unwrap().public(), key.public());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn every_kind_the_schema_names_is_in_the_one_table_of_kinds() {
    use vleo_files::meta::FORMAT;
    for k in Kind::ALL {
        assert!(
            vleo_kinds::KINDS.iter().any(|t| t.name == k.name()
                && t.format() == Some(FORMAT as u32)
                && t.reader == vleo_kinds::Reader::Files),
            "{} is not in crates/vleo-kinds as format {FORMAT}, read by vleo-files",
            k.name()
        );
    }
    assert!(SCHEMA.contains(&format!(
        "PRAGMA application_id = {};",
        vleo_kinds::APPLICATION_ID
    )));
}
