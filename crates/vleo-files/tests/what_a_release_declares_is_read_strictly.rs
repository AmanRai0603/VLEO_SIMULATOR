//! What a group's release declares is read strictly: who helped with its
//! relation, its isolation results as cases in SI, its CSV as written, and its
//! seal as the page makes it.
//!
//! These are the library's own reading of a release, the same installed and
//! in the page. They were held by the developer's intake while it read
//! releases through them; with that intake retired, they are held here, at the
//! library that keeps reading them (`vleo_files::intake`, `vleo_files::seal`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use vleo_files::format_1::Folder;
use vleo_files::intake::{cases, declared_help, parse_csv};
use vleo_files::seal::fingerprint_of;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn declaration(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn a_transcription_is_taken_only_with_its_source_and_a_person_who_checked_it() {
    let root = repo();
    for a in ["none", "wording", "relation"] {
        assert_eq!(declared_help(&root, &declaration(&[("ai", a)])).ai, a);
    }
    // Silence, and a value nobody defined, are an assistant's relation.
    assert_eq!(declared_help(&root, &declaration(&[])).ai, "relation");
    assert_eq!(
        declared_help(&root, &declaration(&[("ai", "copied")])).ai,
        "relation"
    );
    let taken = declared_help(
        &root,
        &declaration(&[
            ("ai", "transcribed"),
            ("source", "crates/vleo-core/src/physics/solar.rs:120"),
            ("checked_by", "A. Person"),
        ]),
    );
    assert_eq!(taken.ai, "transcribed");
    let record = taken.transcribed.unwrap();
    assert!(
        record.contains("solar.rs:120") && record.contains("A. Person"),
        "{record}"
    );
    for (source, checker) in [
        ("", "A. Person"),
        ("x.rs:1", ""),
        ("x.rs:1", "  "),
        ("x.rs:1", "Claude"),
        ("x.rs:1", "claude/opus"),
        ("x.rs:1", "Copilot bot"),
    ] {
        let h = declared_help(
            &root,
            &declaration(&[
                ("ai", "transcribed"),
                ("source", source),
                ("checked_by", checker),
            ]),
        );
        assert_eq!(h.ai, "relation", "{source:?} / {checker:?}");
        assert!(h.not_taken.is_some() && h.transcribed.is_none());
    }
}

#[test]
fn csv_reads_quotes_commas_and_line_breaks() {
    let (h, r) = parse_csv("a,b\n\"x, \"\"y\"\"\",\"two\nlines\"\n1,2\n");
    assert_eq!(h, ["a", "b"]);
    assert_eq!(r[0], ["x, \"y\"", "two\nlines"]);
    assert_eq!(r[1], ["1", "2"]);
}

#[test]
fn the_seal_fingerprint_is_the_one_the_page_makes() {
    // Two files and the folder's record of itself; the expected value was
    // worked out with Python's hashlib, not with this code.
    let mut f = Folder::new();
    f.insert("group.csv".into(), b"id,name\nx,X\n".to_vec());
    f.insert("nodes/a/pseudocode.txt".into(), b"return 1\n".to_vec());
    f.insert(
        "reviews.csv".into(),
        b"name,scope\nsomebody,group\n".to_vec(),
    );
    assert_eq!(
        fingerprint_of(&f, "group"),
        "e7eda2b52f58711281855d3e2815334551728a2db3b862a724b7ba6f4dc93575"
    );
    // One byte changed after the seal, and it is another fingerprint.
    f.insert("nodes/a/pseudocode.txt".into(), b"return 2\n".to_vec());
    assert_ne!(
        fingerprint_of(&f, "group"),
        "e7eda2b52f58711281855d3e2815334551728a2db3b862a724b7ba6f4dc93575"
    );
}

#[test]
fn isolation_results_become_cases_in_si() {
    let c = cases(
        "lead [d],answer [d],tolerance,refuses,origin,says\n2,1,1e-9,no,hand,two days\nNaN,,,yes,hand,\n",
        "isolation.csv",
    )
    .unwrap();
    assert_eq!(c[0]["inputs"], "{ lead = 172800.0 }");
    assert_eq!(c[0]["expect"], "86400.0");
    assert_eq!(c[0]["label"], "two days");
    assert_eq!(c[1]["refuse"], "yes");
    assert_eq!(c[1]["inputs"], "{ lead = nan }");
    assert!(!c[1].contains_key("expect"));
    // Where each answer came from goes with it.
    assert_eq!(c[0]["origin"], "hand");
}

#[test]
fn each_published_member_is_answered_in_its_own_column() {
    let c = cases(
        "lead [d],answer [d],answer.Half [d],tolerance,refuses,origin\n2,1,0.5,1e-9,no,code\n-1,,,,yes,hand\n",
        "isolation.csv",
    )
    .unwrap();
    // A member's column is an answer, never an input.
    assert_eq!(c[0]["inputs"], "{ lead = 172800.0 }");
    assert_eq!(c[0]["also"], "{ Half = 43200.0 }");
    assert!(!c[1].contains_key("also"));
}
