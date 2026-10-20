//! A node engineer asks, in their node's file, for a contract change (W4) or
//! a breakdown (W13): a request signed on its own, so asking does not wait for
//! the node to pass its checks (docs/briefs/F3.md, F3-d).
//!
//! The node and the group are the signing tests' (`a_node_is_signed_through_
//! the_chain.rs`): solar's `sw_activity_band`, Test Nia assigned it and Test
//! Dee her deputy; Test Ivo is assigned nothing, and Test Zed is Ivo's deputy.

use std::path::{Path, PathBuf};
use vleo_files::chain::{self, Registry};
use vleo_files::copied::digest;
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::save::{ask_in_place, save, Asked};
use vleo_files::sqlite;

const NODE: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";
const UID: &str = "sw_activity_band";
const AT: &str = "2026-10-15T12:00:00Z";
const APP: &str = "vleo 1.0.0 (asking)";

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
        proof: vleo_files::people::proof(k, name),
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
    // A second node of hers, for a request moved from one node to another.
    g.assignments.push(Assignment {
        block_uid: s("b-two"),
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
    let c = std::env::temp_dir().join(format!("vleo-ask-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&c);
    std::fs::create_dir_all(c.join("groups/l3_solar/nodes")).unwrap();
    sqlite::write(f, &c.join(NODE)).unwrap();
    c
}

fn opened(c: &Path) -> String {
    digest(&std::fs::read(c.join(NODE)).unwrap())
}

const ASK: &str = "The band's floor reads F10.7 as daily: it needs the 81-day mean.";

fn asked(f: &File, who: &str, k: &SigningKey) -> (File, String) {
    chain::ask(f, UID, "contract", ASK, who, k, AT, &group("")).unwrap()
}

#[test]
fn a_request_asked_is_kept_signed_and_the_node_is_not_touched() {
    let g = group("");
    let signed = chain::sign_node(&node(), UID, "Test Nia", &key(74), AT, &g)
        .unwrap()
        .0;
    let (f, uid) =
        chain::ask(&signed, UID, "breakdown", ASK, "Test Nia", &key(74), AT, &g).unwrap();
    assert_eq!(
        f.requests,
        [Request {
            uid: uid.clone(),
            kind: s("breakdown"),
            scope: s(UID),
            raised_by: s("Test Nia"),
            at: s(AT),
            body: s(ASK),
            status: s("open"),
            answer: s(""),
        }]
    );
    chain::check_request(&f, &uid, &g).expect("the request counts");
    // The node is as it was: its revision, its content, and its signature.
    assert_eq!(f.blocks, signed.blocks);
    assert_eq!(f.meta, signed.meta);
    assert_eq!(
        chain::digest(&f, UID).unwrap(),
        chain::digest(&signed, UID).unwrap()
    );
    chain::check_node(&f, UID, &g).expect("the node's signature still holds");
    // A second request is a request of its own.
    let (two, second) = chain::ask(
        &f,
        UID,
        "contract",
        "And its ceiling.",
        "Test Nia",
        &key(74),
        AT,
        &g,
    )
    .unwrap();
    assert_ne!(second, uid);
    chain::check_request(&two, &uid, &g).unwrap();
    chain::check_request(&two, &second, &g).unwrap();
    // Read and written again, it is the same.
    let at = std::env::temp_dir().join(format!("vleo-ask-rw-{}.vnode", std::process::id()));
    sqlite::write(&two, &at).unwrap();
    let back = sqlite::read(&at).unwrap();
    chain::check_request(&back, &second, &g).unwrap();
    let _ = std::fs::remove_file(&at);
}

#[test]
fn a_node_that_cannot_be_signed_yet_still_asks() {
    let g = group("");
    let mut open = node();
    open.blocks[0].behaviour = s("open");
    let e = chain::sign_node(&open, UID, "Test Nia", &key(74), AT, &g).unwrap_err();
    assert!(e.message().contains("is open"), "{}", e.message());
    let (f, uid) = chain::ask(&open, UID, "breakdown", ASK, "Test Nia", &key(74), AT, &g).unwrap();
    chain::check_request(&f, &uid, &g).expect("asked, though it is open");
}

#[test]
fn only_its_node_engineer_or_their_deputy_asks_in_its_file() {
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
        for kind in ["contract", "code"] {
            let e = chain::ask(&node(), UID, kind, ASK, who, &k, AT, &g).unwrap_err();
            assert!(e.message().contains(says), "{who}, {kind}: {}", e.message());
        }
    }
    // A key revoked before the moment it asks asks nothing.
    let e = chain::ask(
        &node(),
        UID,
        "contract",
        ASK,
        "Test Nia",
        &key(74),
        AT,
        &group("2026-10-01"),
    )
    .unwrap_err();
    assert!(e.message().contains("revoked"), "{}", e.message());
    // Her deputy asks for her, and it counts.
    let (f, uid) = asked(&node(), "Test Dee", &key(76));
    chain::check_request(&f, &uid, &g).expect("the deputy's request counts");
}

#[test]
fn a_request_says_what_it_asks_and_is_of_the_contract_a_breakdown_or_the_code() {
    let g = group("");
    for (kind, body, says) in [
        ("contract", "", "says nothing"),
        ("breakdown", "  \n ", "says nothing"),
        ("code", " ", "says nothing"),
        ("wish", ASK, "\"wish\" is none of them"),
    ] {
        let e = chain::ask(&node(), UID, kind, body, "Test Nia", &key(74), AT, &g).unwrap_err();
        assert!(
            e.message().contains(says),
            "{kind} {body:?}: {}",
            e.message()
        );
    }
}

#[test]
fn a_request_changed_after_it_was_asked_does_not_count() {
    let g = group("");
    let (f, uid) = asked(&node(), "Test Nia", &key(74));
    type Change = fn(&mut Request);
    let cases: [(&str, Change); 5] = [
        ("its words", |r| r.body.push('!')),
        ("its kind", |r| r.kind = s("breakdown")),
        ("its moment", |r| r.at = s("2026-10-15T12:00:01Z")),
        ("its node", |r| r.scope = s("elsewhere")),
        ("who asked", |r| r.raised_by = s("Test Dee")),
    ];
    for (what, change) in cases {
        let mut g2 = f.clone();
        change(&mut g2.requests[0]);
        assert!(
            chain::check_request(&g2, &uid, &g).is_err(),
            "{what} changed, and it still counts"
        );
    }
    // Its answer is its group's, and not what was signed.
    let mut answered = f.clone();
    answered.requests[0].status = s("accepted");
    answered.requests[0].answer = s("Contract 2 reads the 81-day mean.");
    chain::check_request(&answered, &uid, &g).expect("an answer does not undo the asking");
    // Signed by someone else as theirs, it is not the asker's.
    let mut other = f.clone();
    other
        .signatures
        .retain(|r| r.scope != chain::request_scope(&uid));
    let row = chain::sign(
        &other,
        &chain::request_scope(&uid),
        "Test Dee",
        &key(76),
        AT,
        "ok",
        "",
    )
    .unwrap();
    other.signatures.push(row);
    let e = chain::check_request(&other, &uid, &g).unwrap_err();
    assert!(
        e.message().contains("Test Dee may not sign"),
        "{}",
        e.message()
    );
}

#[test]
fn asked_in_the_copy_the_request_is_written_and_a_second_writer_stops_it() {
    let g = group("");
    let c = copy("in-place", &node());
    let seen = opened(&c);
    let before = std::fs::read(c.join(NODE)).unwrap();
    match ask_in_place(
        &c,
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
        Asked::Written { uid, digest: d } => {
            assert_eq!(d, opened(&c));
            let f = sqlite::read(&c.join(NODE)).unwrap();
            chain::check_request(&f, &uid, &g).unwrap();
            assert_eq!(f.meta.get("written_by_app").map(String::as_str), Some(APP));
            assert_eq!(f.meta.get("revision").map(String::as_str), Some("1"));
            assert_eq!(f.meta.get("writer").map(String::as_str), Some("Test Nia"));
        }
        other => panic!("not asked: {other:?}"),
    }
    assert_ne!(std::fs::read(c.join(NODE)).unwrap(), before);
    // Asked again from what was first seen: someone wrote it since.
    let now = std::fs::read(c.join(NODE)).unwrap();
    match ask_in_place(
        &c,
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
        Asked::SecondWriter {
            writer, revision, ..
        } => {
            assert_eq!((writer.as_str(), revision), ("Test Nia", 1));
        }
        other => panic!("not stopped: {other:?}"),
    }
    assert_eq!(
        std::fs::read(c.join(NODE)).unwrap(),
        now,
        "nothing is written"
    );
    // What changed since it was seen is said before anything else: not
    // what is wrong with the asking.
    assert!(matches!(
        ask_in_place(
            &c,
            NODE,
            &seen,
            "Test Ivo",
            &key(75),
            AT,
            "wish",
            "",
            APP,
            &g
        )
        .unwrap(),
        Asked::SecondWriter { .. }
    ));
    // Refused, nothing is written either.
    let seen = opened(&c);
    let e = ask_in_place(
        &c,
        NODE,
        &seen,
        "Test Ivo",
        &key(75),
        AT,
        "contract",
        ASK,
        APP,
        &g,
    )
    .unwrap_err();
    assert!(
        e.message().contains("Test Ivo is not assigned"),
        "{}",
        e.message()
    );
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), now);
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn a_save_keeps_every_request_as_it_was_asked() {
    let g = group("");
    let (f, _) = asked(&node(), "Test Nia", &key(74));
    type Change = fn(&mut File);
    let cases: [(&str, Change); 3] = [
        ("dropped", |f| f.requests.clear()),
        ("changed", |f| f.requests[0].body.push('!')),
        ("added", |f| {
            let mut r = f.requests[0].clone();
            r.uid = s("by-hand");
            f.requests.push(r)
        }),
    ];
    for (what, change) in cases {
        let c = copy(&format!("save-{what}"), &f);
        let mut edit = sqlite::read(&c.join(NODE)).unwrap();
        change(&mut edit);
        let before = std::fs::read(c.join(NODE)).unwrap();
        let e = save(&c, NODE, &edit, &opened(&c), "Test Nia", APP, &g).unwrap_err();
        assert!(
            e.message().contains("a request is kept as it was asked"),
            "{what}: {}",
            e.message()
        );
        assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before, "{what}");
        let _ = std::fs::remove_dir_all(&c);
    }
    // An edit that keeps them is saved.
    let c = copy("save-kept", &f);
    let mut edit = sqlite::read(&c.join(NODE)).unwrap();
    edit.texts.push(Text {
        scope: s(UID),
        kind: s("note"),
        body: s("A line of Test Nia's own."),
    });
    save(&c, NODE, &edit, &opened(&c), "Test Nia", APP, &g).unwrap();
    let _ = std::fs::remove_dir_all(&c);
}

/// `f` with a request written by hand, `uid` on `scope`, signed by Nia at
/// `signed_at`.
fn by_hand(
    f: &File,
    uid: &str,
    scope: &str,
    kind: &str,
    body: &str,
    at: &str,
    signed_at: &str,
) -> File {
    let mut f = f.clone();
    f.requests.push(Request {
        uid: s(uid),
        kind: s(kind),
        scope: s(scope),
        raised_by: s("Test Nia"),
        at: s(at),
        body: s(body),
        status: s("open"),
        answer: s(""),
    });
    let row = chain::sign(
        &f,
        &chain::request_scope(uid),
        "Test Nia",
        &key(74),
        signed_at,
        "ok",
        "",
    )
    .unwrap();
    f.signatures.push(row);
    f
}

#[test]
fn a_request_is_the_only_one_of_its_name_and_named_for_its_node() {
    let g = group("");
    let (f, uid) = asked(&node(), "Test Nia", &key(74));
    // Two rows of one name: refused, never read as the first.
    let mut twice = f.clone();
    let mut again = twice.requests[0].clone();
    again.body = s("Something she never asked.");
    twice.requests.push(again);
    for e in [
        chain::check_request(&twice, &uid, &g).unwrap_err(),
        chain::digest(&twice, &chain::request_scope(&uid)).unwrap_err(),
    ] {
        assert!(
            e.message().contains("more than one request"),
            "{}",
            e.message()
        );
    }
    // Named otherwise than asking names one, signed as it is: refused.
    for name in [
        "r1",
        "sw_activity_band-r0",
        "sw_activity_band-r01",
        "sw_activity_band-rx",
        "elsewhere-r1",
    ] {
        let f = by_hand(&node(), name, UID, "contract", ASK, AT, AT);
        let e = chain::check_request(&f, name, &g).unwrap_err();
        assert!(
            e.message().contains("is not named for the node it is on"),
            "{name}: {}",
            e.message()
        );
    }
    let named = by_hand(&node(), "sw_activity_band-r7", UID, "contract", ASK, AT, AT);
    chain::check_request(&named, "sw_activity_band-r7", &g).expect("named as asking names one");
    // A block whose uid is a request's scope: neither is signed as the other.
    let mut odd = node();
    let mut b = odd.blocks[0].clone();
    b.uid = s("request:x");
    odd.blocks.push(b);
    odd.requests.push(Request {
        uid: s("x"),
        kind: s("contract"),
        scope: s(UID),
        raised_by: s("Test Nia"),
        at: s(AT),
        body: s(ASK),
        status: s("open"),
        answer: s(""),
    });
    let e = chain::digest(&odd, "request:x").unwrap_err();
    assert!(
        e.message().contains("names a block and a request"),
        "{}",
        e.message()
    );
}

#[test]
fn a_request_counts_only_as_it_was_asked() {
    let g = group("");
    for (what, f, uid, says) in [
        (
            "of a wish",
            by_hand(&node(), "sw_activity_band-r1", UID, "wish", ASK, AT, AT),
            "sw_activity_band-r1",
            "is of \"wish\"",
        ),
        (
            "saying nothing",
            by_hand(&node(), "sw_activity_band-r1", UID, "contract", " ", AT, AT),
            "sw_activity_band-r1",
            "says nothing",
        ),
        (
            "at no moment",
            by_hand(
                &node(),
                "sw_activity_band-r1",
                UID,
                "contract",
                ASK,
                "yesterday",
                AT,
            ),
            "sw_activity_band-r1",
            "neither a day",
        ),
        (
            "signed at another moment",
            by_hand(
                &node(),
                "sw_activity_band-r1",
                UID,
                "contract",
                ASK,
                AT,
                "2026-10-15T12:00:01Z",
            ),
            "sw_activity_band-r1",
            "signed it at another moment",
        ),
    ] {
        let e = chain::check_request(&f, uid, &g).unwrap_err();
        assert!(e.message().contains(says), "{what}: {}", e.message());
    }
    // Moved to another node of hers, it is not named for it: a request's
    // name is its node's.
    let mut two = node();
    let mut b = two.blocks[0].clone();
    b.uid = s("b-two");
    b.id = s("two");
    two.blocks.push(b);
    let (mut f, uid) =
        chain::ask(&two, UID, "contract", ASK, "Test Nia", &key(74), AT, &g).unwrap();
    chain::check_request(&f, &uid, &g).unwrap();
    f.requests[0].scope = s("b-two");
    let e = chain::check_request(&f, &uid, &g).unwrap_err();
    assert!(
        e.message().contains("is not named for the node it is on"),
        "{}",
        e.message()
    );
}

#[test]
fn nothing_is_asked_of_an_archived_node_a_mount_or_a_file_of_two_revisions() {
    let mut archived = node();
    archived.blocks[0].archived = 1;
    let e = chain::ask(
        &archived,
        UID,
        "contract",
        ASK,
        "Test Nia",
        &key(74),
        AT,
        &group(""),
    )
    .unwrap_err();
    assert!(e.message().contains("is archived"), "{}", e.message());
    let mut g = group("");
    g.file.mounts.push(Mount {
        block_uid: s(UID),
        group_id: s("l3_other"),
        ..Mount::default()
    });
    let e = chain::ask(&node(), UID, "contract", ASK, "Test Nia", &key(74), AT, &g).unwrap_err();
    assert!(
        e.message().contains("is where a group hangs"),
        "{}",
        e.message()
    );
    let mut two = node();
    two.meta.insert(s("revision"), s("2"));
    let c = copy("two-revisions", &two);
    let before = std::fs::read(c.join(NODE)).unwrap();
    let e = ask_in_place(
        &c,
        NODE,
        &opened(&c),
        "Test Nia",
        &key(74),
        AT,
        "contract",
        ASK,
        APP,
        &group(""),
    )
    .unwrap_err();
    assert!(e.message().contains("its block says 1"), "{}", e.message());
    assert_eq!(std::fs::read(c.join(NODE)).unwrap(), before);
    let _ = std::fs::remove_dir_all(&c);
}

#[test]
fn something_the_code_cannot_do_yet_is_asked_in_the_node_that_waits_on_it() {
    // W14: for the developer, signed by whoever may write the node, and it
    // counts as any request does.
    let g = group("");
    let (f, uid) = chain::ask(
        &node(),
        UID,
        "code",
        "The band needs a lookup of the NOAA scale by date, which the method language does not have.",
        "Test Nia",
        &key(74),
        AT,
        &g,
    )
    .expect("asked");
    chain::check_request(&f, &uid, &g).expect("a request of the code counts");
    assert_eq!(f.requests[0].kind, "code");
}
