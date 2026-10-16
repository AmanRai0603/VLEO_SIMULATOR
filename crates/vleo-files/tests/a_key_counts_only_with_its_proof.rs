//! A key counts only with the proof, made with it, that its holder offered it
//! for its person, kept in its row (the review of pull request 172); and a
//! file from before the proof was kept, format 2, reads as this format with
//! every key's proof empty, nothing lost and no copy kept.
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

#[test]
fn a_file_from_before_the_proof_reads_as_this_format_and_is_kept_no_copy_of() {
    let dir = std::env::temp_dir().join(format!("vleo-format-2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let at = dir.join("l3_solar.vgroup");
    sqlite::write(&solar(&people::proof(&key(74), "Test Nia")).file, &at).unwrap();
    // As format 2 wrote it: no proof column, and format 2 in both places.
    {
        let db = Connection::open(&at).unwrap();
        db.execute_batch(
            "ALTER TABLE person_key DROP COLUMN proof;
             UPDATE meta SET value = '2' WHERE key = 'format';
             PRAGMA user_version = 2;",
        )
        .unwrap();
    }
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
    // Written again: this format, and no copy of the format-2 file kept.
    sqlite::write(&f, &at).unwrap();
    let names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["l3_solar.vgroup"], "nothing lost, so nothing kept");
    let db = Connection::open(&at).unwrap();
    let v: i64 = db
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(v, FORMAT);
    let _ = std::fs::remove_dir_all(&dir);
}
