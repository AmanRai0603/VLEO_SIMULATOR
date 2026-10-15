//! A node's day's work signed with its node engineer's key, checked at once
//! through the chain, and refused when anyone else signs it or its checks fail
//! (docs/briefs/F3.md, F3-b).
//!
//! The node is solar's `sw_activity_band`, as solar's 1.1 release holds it
//! (`tests/fixtures/l3_solar-1.1.vleo`): its block, ports, wires, method and
//! cases, in a node file of its own. The group file is invented: Test Nia is
//! assigned the node and Test Dee is her deputy; Test Ivo is assigned nothing,
//! and Test Zed is Ivo's deputy, not Nia's.

use std::path::{Path, PathBuf};
use vleo_files::chain::{self, Registry};
use vleo_files::copied::digest;
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::save::{sign_in_place, Saved};
use vleo_files::sqlite;

const NODE: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";
const UID: &str = "sw_activity_band";
const AT: &str = "2026-10-15T12:00:00Z";
const APP: &str = "vleo 1.0.0 (signing)";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The node as its group's release holds it, in a node file of its own.
fn node() -> File {
    let how = vleo_files::upgrade::Upgrade {
        app: "vleo 1.0.0 (test)",
        at: "2026-10-01",
    };
    let rel = sqlite::open(
        &root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo"),
        &how,
    )
    .unwrap()
    .file;
    let mut n = File::new(Kind::Node, "vleo 1.0.0 (test)");
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("block_uid", UID),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Test Nia"),
    ] {
        n.meta.insert(s(k), s(v));
    }
    let b = rel.blocks.iter().find(|b| b.uid == UID).unwrap().clone();
    n.blocks = vec![b];
    // One revision: the file's and its block's.
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

fn person(f: &mut File, name: &str, deputy_for: &str, k: &SigningKey, revoked_from: &str) {
    f.people.push(Person {
        name: s(name),
        role: s("node engineer"),
        deputy_for: s(deputy_for),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: k.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Test Ola"),
        revoked_from: s(revoked_from),
    });
}

/// Solar's group file: Nia (74) assigned the node, Dee (76) her deputy, Ivo
/// (75) assigned nothing, Zed (77) Ivo's deputy. Nia's key is revoked from
/// `nia_revoked`, when it is not empty.
fn group(nia_revoked: &str) -> Registry {
    let mut g = File::new(Kind::Group, "vleo 1.0.0 (test)");
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("writer", "Test Ola"),
        ("based_on", ""),
    ] {
        g.meta.insert(s(k), s(v));
    }
    person(&mut g, "Test Nia", "", &key(74), nia_revoked);
    person(&mut g, "Test Dee", "Test Nia", &key(76), "");
    person(&mut g, "Test Ivo", "", &key(75), "");
    person(&mut g, "Test Zed", "Test Ivo", &key(77), "");
    g.assignments.push(Assignment {
        block_uid: s(UID),
        person: s("Test Nia"),
        contract_version: 1,
    });
    Registry {
        from: s("l3_solar's group file"),
        file: g,
    }
}

/// A copy holding `f` as the node's file.
fn copy(name: &str, f: &File) -> PathBuf {
    let c = std::env::temp_dir().join(format!("vleo-sign-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&c);
    std::fs::create_dir_all(c.join("groups/l3_solar/nodes")).unwrap();
    sqlite::write(f, &c.join(NODE)).unwrap();
    c
}

fn opened(c: &Path) -> String {
    digest(&std::fs::read(c.join(NODE)).unwrap())
}

#[test]
fn the_node_engineer_signs_and_the_signature_checks_through_the_chain() {
    let g = group("");
    assert!(
        vleo_files::checks::check_node(&node()).holds(),
        "the fixture's node holds its checks"
    );
    let signed = chain::sign_node(&node(), UID, "Test Nia", &key(74), AT, &g)
        .unwrap()
        .0;
    chain::check_node(&signed, UID, &g).expect("it checks through the chain");
    let row = signed.signatures.last().unwrap();
    assert_eq!(
        (row.signer.as_str(), row.verdict.as_str()),
        ("Test Nia", "ok")
    );
}

#[test]
fn only_the_assigned_node_engineer_or_their_deputy_signs() {
    let g = group("");
    for (who, k, says) in [
        (
            "Test Ivo",
            key(75),
            "Test Ivo is not assigned sw_activity_band",
        ),
        (
            "Test Zed",
            key(77),
            "Test Zed is not assigned sw_activity_band",
        ),
        (
            "Test Nia",
            key(75),
            "not one l3_solar's group file registers for them",
        ),
        ("Test Stranger", key(78), "Test Stranger is not registered"),
    ] {
        let e = chain::sign_node(&node(), UID, who, &k, AT, &g).unwrap_err();
        assert!(e.message().contains(says), "{who}: {}", e.message());
        // Refused before signing, not by the chain's check after it.
        assert!(
            !e.message().contains("no signature that counts"),
            "{who}: {}",
            e.message()
        );
    }
    // Her deputy signs for her, and it counts.
    let signed = chain::sign_node(&node(), UID, "Test Dee", &key(76), AT, &g)
        .unwrap()
        .0;
    chain::check_node(&signed, UID, &g).expect("the deputy's signature counts");
}

#[test]
fn a_key_revoked_before_the_signing_signs_nothing() {
    let g = group("2026-10-01T00:00:00Z");
    let e = chain::sign_node(&node(), UID, "Test Nia", &key(74), AT, &g).unwrap_err();
    assert!(
        e.message().contains("revoked") && !e.message().contains("no signature that counts"),
        "{}",
        e.message()
    );
}

#[test]
fn a_node_whose_case_disagrees_is_not_signed_and_nothing_is_written() {
    let mut bad = node();
    let t = bad
        .tables
        .iter_mut()
        .find(|t| t.path == "results/isolation.csv")
        .expect("the node carries its cases");
    // The quietest day, 64 sfu, is band 1; the case now says band 2.
    assert!(t.csv.contains("\n64,1,"), "{}", t.csv);
    t.csv = t.csv.replacen("\n64,1,", "\n64,2,", 1);
    let c = copy("disagrees", &bad);
    let before = std::fs::read(c.join(NODE)).unwrap();
    let e = sign_in_place(
        &c,
        NODE,
        &opened(&c),
        "Test Nia",
        &key(74),
        AT,
        APP,
        &group(""),
    )
    .unwrap_err();
    assert!(
        e.message()
            .contains("sw_activity_band is not signed: its checks refuse it")
            && e.message().contains("does not reproduce"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn signed_in_place_as_the_revision_it_is_and_a_second_writer_stopped() {
    let g = group("");
    let c = copy("signed", &node());
    let seen = opened(&c);
    match sign_in_place(&c, NODE, &seen, "Test Nia", &key(74), AT, APP, &g).unwrap() {
        Saved::Written { revision, .. } => {
            assert_eq!(revision, 1, "signing does not raise the revision")
        }
        other => panic!("not signed: {other:?}"),
    }
    let now = sqlite::read(&c.join(NODE)).unwrap();
    chain::check_node(&now, UID, &g).expect("the file in the copy is signed");
    assert_eq!(
        now.meta.get("written_by_app").map(String::as_str),
        Some(APP)
    );
    // What was signed is not what the person saw: stopped, nothing written.
    let signed_bytes = std::fs::read(c.join(NODE)).unwrap();
    match sign_in_place(&c, NODE, &seen, "Test Dee", &key(76), AT, APP, &g).unwrap() {
        Saved::SecondWriter { .. } => {}
        other => panic!("signed over what changed since it was seen: {other:?}"),
    }
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), signed_bytes);
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_file_changed_since_it_was_seen_is_a_second_writer_before_its_checks_are_read() {
    let g = group("");
    let c = copy("changed", &node());
    let seen = opened(&c);
    // Someone else's revision, whose case disagrees, lands after it was seen.
    let mut theirs = node();
    let t = theirs
        .tables
        .iter_mut()
        .find(|t| t.path == "results/isolation.csv")
        .unwrap();
    t.csv = t.csv.replacen("\n64,1,", "\n64,2,", 1);
    theirs.meta.insert(s("writer"), s("Test Dee"));
    sqlite::write(&theirs, &c.join(NODE)).unwrap();
    let before = std::fs::read(c.join(NODE)).unwrap();
    match sign_in_place(&c, NODE, &seen, "Test Nia", &key(74), AT, APP, &g) {
        Ok(Saved::SecondWriter { writer, .. }) => assert_eq!(writer, "Test Dee"),
        other => panic!("not stopped as a second writer: {other:?}"),
    }
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn signing_holds_the_copy_s_lock_and_reads_opened_as_a_digest() {
    let g = group("");
    let c = copy("guards", &node());
    let before = std::fs::read(c.join(NODE)).unwrap();
    let e = sign_in_place(&c, NODE, "", "Test Nia", &key(74), AT, APP, &g).unwrap_err();
    assert!(
        e.message()
            .contains("is not the SHA-256 of the node as it was opened"),
        "{}",
        e.message()
    );
    let held = vleo_files::save::lock(&c).unwrap();
    let e = sign_in_place(&c, NODE, &opened(&c), "Test Nia", &key(74), AT, APP, &g).unwrap_err();
    assert!(
        e.message()
            .contains("another program is copying the database or writing your copy"),
        "{}",
        e.message()
    );
    drop(held);
    // The node kept elsewhere than its group's folder is not signed there.
    let planted = "groups/l3_other/nodes/sw_activity_band.vnode";
    std::fs::create_dir_all(c.join("groups/l3_other/nodes")).unwrap();
    std::fs::copy(c.join(NODE), c.join(planted)).unwrap();
    let seen = digest(&std::fs::read(c.join(planted)).unwrap());
    let e = sign_in_place(&c, planted, &seen, "Test Nia", &key(74), AT, APP, &g).unwrap_err();
    assert!(
        e.message()
            .contains("is not where the node it holds is kept"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_signature_under_an_assistant_s_name_is_not_made() {
    let mut g = group("");
    person(&mut g.file, "Claude", "", &key(79), "");
    g.file.assignments[0].person = s("Claude");
    let c = copy("assistant", &node());
    let before = std::fs::read(c.join(NODE)).unwrap();
    let e = sign_in_place(&c, NODE, &opened(&c), "Claude", &key(79), AT, APP, &g).unwrap_err();
    assert!(
        e.message().contains("an assistant's name"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&c);
}

/// The node with its quietest case, 64 sfu, saying band 2: it disagrees.
fn disagreeing() -> File {
    let mut bad = node();
    let t = bad
        .tables
        .iter_mut()
        .find(|t| t.path == "results/isolation.csv")
        .unwrap();
    t.csv = t.csv.replacen("\n64,1,", "\n64,2,", 1);
    bad
}

#[test]
fn the_library_s_signing_holds_the_node_to_its_checks_whoever_calls_it() {
    let e =
        chain::sign_node(&disagreeing(), UID, "Test Nia", &key(74), AT, &group("")).unwrap_err();
    assert!(
        e.message().contains("is not signed: its checks refuse it")
            && e.message().contains("does not reproduce"),
        "{}",
        e.message()
    );
}

#[test]
fn a_node_with_nothing_a_check_reads_is_not_signed() {
    let g = group("");
    type Change = fn(&mut File);
    let cases: [(&str, Change, &str); 5] = [
        (
            "archived",
            |f| f.blocks[0].archived = 1,
            "is archived: an archived node is not signed",
        ),
        (
            "open",
            |f| f.blocks[0].behaviour = s("open"),
            "is open: nothing in it is decided or computed yet",
        ),
        (
            "lookup",
            |f| f.blocks[0].behaviour = s("lookup"),
            "nothing checks a lookup node yet",
        ),
        (
            "stated with no value",
            |f| {
                f.blocks[0].behaviour = s("stated");
                for p in f.ports.iter_mut() {
                    p.value = s("");
                }
            },
            "states no value that is a number",
        ),
        (
            "a case from the code under test",
            |f| {
                f.cases.push(TestCase {
                    block_uid: s(UID),
                    name: s("snapshot"),
                    inputs: s("f107=100"),
                    expected: s("2"),
                    tolerance: s("0"),
                    provenance: s("agent-generated"),
                    source: s(""),
                })
            },
            "an expected value never comes from the code under test",
        ),
    ];
    for (what, change, says) in cases {
        let mut f = node();
        change(&mut f);
        let c = copy(&what.replace(' ', "-"), &f);
        let before = std::fs::read(c.join(NODE)).unwrap();
        let e =
            sign_in_place(&c, NODE, &opened(&c), "Test Nia", &key(74), AT, APP, &g).unwrap_err();
        assert!(e.message().contains(says), "{what}: {}", e.message());
        assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before, "{what}");
        let _ = std::fs::remove_dir_all(&c);
    }
}

#[test]
fn a_signature_under_an_assistant_s_name_never_counts_whatever_made_it() {
    let mut g = group("");
    person(&mut g.file, "Claude", "", &key(79), "");
    g.file.assignments[0].person = s("Claude");
    let mut f = node();
    let row = chain::sign(&f, UID, "Claude", &key(79), AT, "ok", "").unwrap();
    f.signatures.push(row);
    let e = chain::check_node(&f, UID, &g).unwrap_err();
    assert!(
        e.message().contains("an assistant's name"),
        "{}",
        e.message()
    );
}

#[test]
fn a_node_is_signed_only_against_its_own_group_s_file() {
    let mut other = group("");
    other.file.meta.insert(s("group_id"), s("l3_power"));
    let c = copy("other-group", &node());
    let e =
        sign_in_place(&c, NODE, &opened(&c), "Test Nia", &key(74), AT, APP, &other).unwrap_err();
    assert!(
        e.message().contains("the group file given is l3_power's"),
        "{}",
        e.message()
    );
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn the_revision_said_is_the_one_its_signature_names() {
    let g = group("");
    let mut f = node();
    f.blocks[0].revision = 4;
    f.meta.insert(s("revision"), s("4"));
    let c = copy("revision", &f);
    match sign_in_place(&c, NODE, &opened(&c), "Test Nia", &key(74), AT, APP, &g).unwrap() {
        Saved::Written { revision, .. } => assert_eq!(revision, 4),
        other => panic!("not signed: {other:?}"),
    }
    let now = sqlite::read(&c.join(NODE)).unwrap();
    assert_eq!(now.signatures.last().unwrap().revision, "4");
    let _ = std::fs::remove_dir_all(&c);
    // The file says one revision and its block another: not signed.
    f.blocks[0].revision = 3;
    let c = copy("two-revisions", &f);
    let before = std::fs::read(c.join(NODE)).unwrap();
    let e = sign_in_place(&c, NODE, &opened(&c), "Test Nia", &key(74), AT, APP, &g).unwrap_err();
    assert!(
        e.message()
            .contains("says it is revision 4, and its block says 3"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&c);
}
