//! A key counts only with the proof, made with it, that its holder offered it
//! for its person, kept in its row (the review of pull request 172); and a
//! file from before the proof was kept, format 2, reads as this format with
//! every key's proof empty, a copy of it kept when it is written again: what
//! was signed over the whole of it, and every key in it, stops counting.
//!
//! Test Nia (74) is a node engineer of solar, assigned `b-nia`; Test Ivo (75)
//! is another. Their group file is made here, its key rows as each test needs.

use rusqlite::Connection;
use vleo_files::chain::{self, Registry};
use vleo_files::keys::SigningKey;
use vleo_files::meta::{Kind, FORMAT};
use vleo_files::model::*;
use vleo_files::people;
use vleo_files::sqlite;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-15T12:00:00Z";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

/// Solar's group file with Nia's key row carrying `proof`.
fn solar(proof: &str) -> Registry {
    let mut g = File::new(Kind::Group, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("writer", "Test Ola"),
        ("based_on", ""),
    ] {
        g.meta.insert(s(k), s(v));
    }
    g.people.push(Person {
        name: s("Test Nia"),
        role: s("node engineer"),
        deputy_for: s(""),
    });
    g.keys.push(PersonKey {
        person: s("Test Nia"),
        public_key: key(74).public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Test Ola"),
        revoked_from: s(""),
        proof: s(proof),
    });
    g.assignments.push(Assignment {
        block_uid: s("b-nia"),
        person: s("Test Nia"),
        contract_version: 1,
    });
    Registry {
        from: s("l3_solar's group file"),
        file: g,
    }
}

/// Nia's node, signed by her.
fn nia_s_node() -> File {
    let mut n = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("block_uid", "b-nia"),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Test Nia"),
    ] {
        n.meta.insert(s(k), s(v));
    }
    n.blocks.push(Block {
        uid: s("b-nia"),
        id: s("nia_node"),
        behaviour: s("method"),
        revision: 1,
        contract_version: 1,
        ..Block::default()
    });
    let row = chain::sign(&n, "b-nia", "Test Nia", &key(74), AT, "ok", "").unwrap();
    n.signatures.push(row);
    n
}

#[test]
fn a_key_signs_only_with_the_proof_its_holder_made_for_them() {
    let node = nia_s_node();
    let pk = key(74).public().to_hex();
    chain::check_node(&node, "b-nia", &solar(&people::proof(&key(74), "Test Nia")))
        .expect("her key, with her proof, counts");
    for (what, proof) in [
        ("no proof", String::new()),
        (
            "a proof made for someone else",
            people::proof(&key(74), "Test Ivo"),
        ),
        (
            "a proof made by another key",
            people::proof(&key(75), "Test Nia"),
        ),
        ("a proof that is not one", s("00ff")),
    ] {
        let g = solar(&proof);
        let e = g.key("Test Nia", &pk, AT).unwrap_err();
        assert!(
            e.message()
                .contains("carries no proof in l3_solar's group file"),
            "{what}: {}",
            e.message()
        );
        let e = chain::check_node(&node, "b-nia", &g).unwrap_err();
        assert!(
            e.message().contains("carries no proof"),
            "{what}: {}",
            e.message()
        );
    }
}

/// `at` as format 2 wrote it: no proof column, and format 2 in both places.
fn as_format_2(at: &std::path::Path) {
    let db = Connection::open(at).unwrap();
    db.execute_batch(
        "ALTER TABLE person_key DROP COLUMN proof;
         UPDATE meta SET value = '2' WHERE key = 'format';
         PRAGMA user_version = 2;",
    )
    .unwrap();
}

#[test]
fn a_file_from_before_the_proof_reads_as_this_format_and_is_kept_when_written() {
    let dir = std::env::temp_dir().join(format!("vleo-format-2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let at = dir.join("l3_solar.vgroup");
    sqlite::write(&solar(&people::proof(&key(74), "Test Nia")).file, &at).unwrap();
    as_format_2(&at);
    let was = std::fs::read(&at).unwrap();
    let f = sqlite::read(&at).unwrap();
    assert_eq!(
        f.meta.get("format").map(String::as_str),
        Some(FORMAT.to_string().as_str())
    );
    assert_eq!(f.keys.len(), 1);
    assert_eq!(
        f.keys[0].proof, "",
        "read with its proof empty, and so signing nothing"
    );
    assert_eq!(f.keys[0].person, "Test Nia");
    // The application opens it the same, and says it was format 2.
    let how = vleo_files::upgrade::Upgrade {
        app: APP,
        at: "2026-10-16",
    };
    let opened = sqlite::open(&at, &how).unwrap();
    assert_eq!(opened.format, vleo_files::meta::FORMAT_2);
    assert_eq!(opened.file, f);
    // Written again: this format, and the format-2 file kept beside it as it
    // was.
    sqlite::write(&f, &at).unwrap();
    let kept = sqlite::kept_copy(&at, vleo_files::meta::FORMAT_2);
    assert_eq!(std::fs::read(&kept).unwrap(), was, "kept as it was");
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["l3_solar.format-2.vgroup", "l3_solar.vgroup"]);
    let db = Connection::open(&at).unwrap();
    let v: i64 = db
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(v, FORMAT);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn what_was_signed_over_a_whole_format_2_file_stops_counting_once_it_is_read() {
    let dir = std::env::temp_dir().join(format!("vleo-format-2-signed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let at = dir.join("l3_solar.vgroup");
    let mut g = solar(&people::proof(&key(74), "Test Nia")).file;
    let row = chain::sign(&g, chain::FILE, "Test Ola", &key(73), AT, "ok", "").unwrap();
    g.signatures.push(row.clone());
    chain::check_row(&g, &row).expect("signed as it is");
    sqlite::write(&g, &at).unwrap();
    as_format_2(&at);
    let f = sqlite::read(&at).unwrap();
    // Its format and its columns are what the whole file's digest covers.
    let e = chain::check_row(&f, &f.signatures[0]).unwrap_err();
    assert_eq!(e.kind(), vleo_files::ErrorKind::Tampered, "{}", e.message());
    // A node's own signature covers the node, not the file, and still checks;
    // but the key that made it has no proof here, and counts for nothing.
    let node = nia_s_node();
    let reg = Registry {
        from: s("l3_solar's group file"),
        file: f,
    };
    chain::check_row(&node, &node.signatures[0]).expect("the node's signature is as it was");
    assert!(chain::check_node(&node, "b-nia", &reg).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_file_that_says_this_format_without_the_proof_column_is_refused() {
    let dir = std::env::temp_dir().join(format!("vleo-format-3-short-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let at = dir.join("l3_solar.vgroup");
    sqlite::write(&solar(&people::proof(&key(74), "Test Nia")).file, &at).unwrap();
    Connection::open(&at)
        .unwrap()
        .execute_batch("ALTER TABLE person_key DROP COLUMN proof;")
        .unwrap();
    let e = sqlite::read(&at).unwrap_err();
    assert_eq!(
        e.kind(),
        vleo_files::ErrorKind::Malformed,
        "{}",
        e.message()
    );
    assert!(e.message().contains("person_key"), "{}", e.message());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_page_reads_a_format_2_file_s_rows_as_the_program_does() {
    // The page's library takes a file as its tables (`File::from_tables`):
    // format 2's, as the page's SQLite gives them, widen the same.
    let g = solar(&people::proof(&key(74), "Test Nia")).file;
    let mut tables = g.to_tables();
    for t in tables.iter_mut() {
        if t.name == "person_key" {
            let at = t.columns.iter().position(|c| c == "proof").unwrap();
            t.columns.remove(at);
            for r in t.rows.iter_mut() {
                r.remove(at);
            }
        }
        if t.name == "meta" {
            for r in t.rows.iter_mut() {
                if r[0] == Cell::Text(s("format")) {
                    r[1] = Cell::Text(s("2"));
                }
            }
        }
    }
    let f = File::from_tables(tables).unwrap();
    assert_eq!(
        f.meta.get("format").map(String::as_str),
        Some(FORMAT.to_string().as_str())
    );
    assert_eq!(f.keys[0].proof, "");
    let mut expected = g.clone();
    expected.keys[0].proof = s("");
    assert_eq!(f, expected, "nothing else read differently");
}
