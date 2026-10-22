//! An issue raised from a red mark (docs/briefs/F2.md, item 4 and its proof;
//! W11, its first half).
//!
//! A small drive of invented people, and solar's releases: 1.1, and a 1.2
//! whose hot edge of the sustained F10.7 band breaks the first solar closure,
//! both anchored by the programme manager. Today's design takes 1.2 and the
//! closure fails. Its node engineer copies the database, opens her key, and
//! raises an issue on the closure: the file is in the copy's `issues/`,
//! numbered, addressed to solar and the closure, signed by her, and carries
//! the engine's evidence — the closure failing, the node 1.2 changed as its
//! cause, and the design it was read on by its fingerprint. A second issue
//! takes the next number. An issue with no words, on no node, without the
//! launch's word or without a key open is refused, and nothing is written.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use vleo_files::chain::{self, FILE};
use vleo_files::keys::{SigningKey, MIN_ITERATIONS};
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::sqlite;

const APP: &str = "vleo 1.0.0 (test)";
const PASSPHRASE: &str = "a long enough passphrase";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn enc(v: &str) -> String {
    v.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// A request as the page makes it; `with_word` false is another program's.
fn ask(port: u16, method: &str, path: &str, pairs: &[(&str, &str)], with_word: bool) -> String {
    let mut body: Vec<String> = pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", enc(v)))
        .collect();
    if method == "POST" && with_word {
        body.insert(0, format!("session={}", vleo_server::session()));
    }
    let body = body.join("&");
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut raw = Vec::new();
    s.read_to_end(&mut raw).unwrap();
    let text = String::from_utf8_lossy(&raw).into_owned();
    text.split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or(text)
}

fn post(port: u16, path: &str, pairs: &[(&str, &str)]) -> String {
    ask(port, "POST", path, pairs, true)
}

fn field<'a>(json: &'a str, key: &str) -> &'a str {
    let k = format!("\"{key}\":\"");
    let at = json
        .find(&k)
        .unwrap_or_else(|| panic!("no {key} in {json}"))
        + k.len();
    &json[at..at + json[at..].find('"').unwrap()]
}

fn sql(file: &Path, statement: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    assert!(db.execute(statement, []).unwrap() > 0, "{statement}");
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

/// Solar's 1.2, from 1.1: the hot edge of the sustained band widened when the
/// cycle is low, which breaks the first solar closure; sealed again.
fn breaking_1_2(f: &Path) {
    sql(f, "UPDATE meta SET value = '1.2' WHERE key = 'version'");
    sql(
        f,
        "UPDATE tbl SET csv = csv || '1.2,2026-10-06,Test Ola,\
         The band needed a wider margin when the cycle is low.,\
         The multiplier was read again.,\
         A wider band is safer.,\
         The multiplier of the hot edge.,,,\
         the one-sided 90th percentile,a multiplier from another table\n' \
         WHERE scope = 'group' AND path = 'versions.csv'",
    );
    sql(
        f,
        "UPDATE doc SET body = replace(body, 'return sustained', \
         'if central < 95 then\n  return central + spread * 14\nend\nreturn sustained') \
         WHERE kind = 'pseudocode' AND scope = 'sw_f107_design_long'",
    );
    let r = vleo_files::intake::Release::open(f).unwrap();
    let fp = vleo_files::seal::fingerprint_of(&r.folder, "group");
    sql(
        f,
        &format!("UPDATE meta SET value = '{fp}' WHERE key = 'fingerprint'"),
    );
}

// Test Pat, the programme manager (91); Test Sam, the system engineer (92);
// Test Ola, who owns solar (93); Test Nia, solar's node engineer (94).
fn drive(at: &Path, keys: &Path) {
    let rel = at.join("groups/l3_solar/releases");
    std::fs::create_dir_all(&rel).unwrap();
    let fixture = root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo");
    std::fs::copy(&fixture, rel.join("l3_solar-1.1.vleo")).unwrap();
    std::fs::copy(&fixture, rel.join("l3_solar-1.2.vleo")).unwrap();
    breaking_1_2(&rel.join("l3_solar-1.2.vleo"));

    let mut programme = group("programme", "Test Pat");
    register(
        &mut programme,
        "Test Pat",
        "programme manager",
        &key(91),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Sam",
        "system engineer",
        &key(92),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Ola",
        "subsystem engineer",
        &key(93),
        "Test Pat",
    );
    programme.blocks.push(Block {
        uid: s("root"),
        id: s("root"),
        behaviour: s("children"),
        ..Block::default()
    });
    programme.mounts.push(Mount {
        block_uid: s("root"),
        group_id: s("systems"),
        release: s(""),
    });
    programme.assignments.push(Assignment {
        block_uid: s("root"),
        person: s("Test Sam"),
        contract_version: 1,
    });
    // The releases are from before 1.0: the programme manager anchors each,
    // and the anchors are part of what he signs.
    let how = vleo_files::upgrade::Upgrade {
        app: APP,
        at: "2026-10-01",
    };
    for v in ["1.1", "1.2"] {
        let r = sqlite::open(&rel.join(format!("l3_solar-{v}.vleo")), &how)
            .unwrap()
            .file;
        programme
            .signatures
            .push(vleo_files::upgrade::anchor(&r, "Test Pat", &key(91), "2026-10-07").unwrap());
    }
    std::fs::create_dir_all(at.join("groups/programme")).unwrap();
    sqlite::write(
        &signed(programme, "Test Pat", &key(91)),
        &at.join("groups/programme/programme.vgroup"),
    )
    .unwrap();

    let mut systems = group("systems", "Test Sam");
    systems.blocks.push(Block {
        uid: s("sys_space_environment"),
        id: s("sys_space_environment"),
        behaviour: s("children"),
        ..Block::default()
    });
    systems.mounts.push(Mount {
        block_uid: s("sys_space_environment"),
        group_id: s("l3_solar"),
        release: s(""),
    });
    systems.assignments.push(Assignment {
        block_uid: s("sys_space_environment"),
        person: s("Test Ola"),
        contract_version: 1,
    });
    std::fs::create_dir_all(at.join("groups/systems")).unwrap();
    sqlite::write(
        &signed(systems, "Test Sam", &key(92)),
        &at.join("groups/systems/systems.vgroup"),
    )
    .unwrap();

    let mut solar = group("l3_solar", "Test Ola");
    register(
        &mut solar,
        "Test Nia",
        "node engineer",
        &key(94),
        "Test Ola",
    );
    sqlite::write(
        &signed(solar, "Test Ola", &key(93)),
        &at.join("groups/l3_solar/l3_solar.vgroup"),
    )
    .unwrap();

    std::fs::write(
        at.join("START HERE.md"),
        format!(
            "# START HERE: an invented drive\n\nThe programme manager's key:\n\n    {}\n",
            key(91).public().fingerprint()
        ),
    )
    .unwrap();
    std::fs::create_dir_all(keys).unwrap();
    let locked = key(94)
        .lock("Test Nia", PASSPHRASE, [7; 16], [7; 12], MIN_ITERATIONS)
        .unwrap();
    let mut f = File::new(Kind::Key, APP);
    f.meta.insert(s("person"), s("Test Nia"));
    f.locked_keys.push(locked.to_row());
    sqlite::write(&f, &keys.join("test-nia.vkey")).unwrap();
}

/// Every issue file in the copy, by name.
fn issues(copy: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(copy.join("issues"))
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn an_issue_is_raised_from_a_red_mark() {
    let scratch = std::env::temp_dir().join(format!("vleo-issue-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let (drv, keys, copy) = (
        scratch.join("drive"),
        scratch.join("keys"),
        scratch.join("copy"),
    );
    drive(&drv, &keys);
    std::env::set_var("VLEO_KEYS", &keys);
    std::env::set_var("VLEO_COPY", &copy);
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    std::env::remove_var("VLEO_DESIGN");
    std::env::set_var("VLEO_DRIVE", &drv);
    let port = vleo_server::serve(Some(root()), 18991, false, true).expect("no server");
    std::env::remove_var("VLEO_DRIVE");

    // Today's design takes 1.2, checked through the chain, and the closure
    // fails.
    let today = ask(port, "GET", "/v1/today", &[], false);
    assert!(today.contains("\"version\":\"1.2\""), "{today}");
    assert!(
        today.contains("anchored by the programme manager"),
        "{today}"
    );
    let todays = field(&today, "fingerprint").to_string();

    let raise = |pairs: &[(&str, &str)]| post(port, "/v1/issues/raise", pairs);
    let closure = [
        ("node", "l3_solar_ach_01"),
        ("what", "The first solar closure fails since solar 1.2."),
        ("inputs", "defaults"),
    ];
    let early = raise(&closure);
    assert!(early.contains("copy it from the drive first"), "{early}");
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");
    let no_key = raise(&closure);
    assert!(no_key.contains("no key is open"), "{no_key}");
    let opened = post(
        port,
        "/v1/key/open",
        &[
            ("file", "test-nia.vkey"),
            ("passphrase", PASSPHRASE),
            ("accept_anchor", &key(91).public().fingerprint()),
        ],
    );
    assert!(opened.contains("\"open\":true"), "{opened}");

    // Refused, and nothing written: no words, no such node, no launch's word.
    for (pairs, says) in [
        (
            vec![("node", "l3_solar_ach_01"), ("what", "  ")],
            "say what is wrong",
        ),
        (
            vec![("node", "no_such_row"), ("what", "x")],
            "no node 'no_such_row'",
        ),
    ] {
        let r = raise(&pairs);
        assert!(r.contains("\"ok\":false") && r.contains(says), "{r}");
    }
    let bare = ask(port, "POST", "/v1/issues/raise", &closure, false);
    assert!(
        bare.contains("does not carry the word this launch gave its page"),
        "{bare}"
    );
    assert!(issues(&copy).is_empty(), "{:?}", issues(&copy));

    // Raised: in the copy's issues/, numbered, addressed, signed, evidenced.
    let r = raise(&closure);
    assert!(r.contains("\"ok\":true"), "{r}");
    assert_eq!(field(&r, "file"), "issues/1-l3_solar.vissue", "{r}");
    assert_eq!(issues(&copy), ["1-l3_solar.vissue"]);
    assert!(
        !drv.join("issues").exists(),
        "the drive is never written: the issue is in the person's copy"
    );
    let f = sqlite::read(&copy.join("issues/1-l3_solar.vissue")).unwrap();
    assert_eq!(
        f.meta.get("signed_rows").map(String::as_str),
        Some("issue change"),
        "its rows are signed with it (F6-d)"
    );
    assert_eq!(f.kind().unwrap(), Kind::Issue);
    assert_eq!(f.meta.get("number").map(String::as_str), Some("1"));
    assert_eq!(f.meta.get("group_id").map(String::as_str), Some("l3_solar"));
    assert_eq!(f.meta.get("writer").map(String::as_str), Some("Test Nia"));
    let i = &f.issues[0];
    assert_eq!((i.number, i.group_id.as_str()), (1, "l3_solar"));
    assert_eq!(i.place, "l3_solar_ach_01");
    assert_eq!(i.raised_by, "Test Nia");
    assert_eq!(i.what, "The first solar closure fails since solar 1.2.");
    assert!(
        i.addressed_to.starts_with("l3_solar")
            && i.addressed_to.ends_with("its node l3_solar_ach_01"),
        "{}",
        i.addressed_to
    );
    for says in [
        "l3_solar_ach_01 fails",
        "caused by sw_f107_design_long",
        "changed by l3_solar 1.2",
        &format!("fingerprint {todays}"),
        "by the engine kernel",
    ] {
        assert!(i.evidence.contains(says), "no {says:?} in:\n{}", i.evidence);
    }
    let sig = f
        .signatures
        .iter()
        .find(|r| r.scope == FILE)
        .expect("signed");
    assert_eq!(sig.signer, "Test Nia");
    assert_eq!(sig.public_key, key(94).public().to_hex());
    chain::check_row(&f, sig).expect("the signature checks");

    // The next issue takes the next number, on whichever node.
    let r = raise(&[
        ("node", "sw_f107_design_long"),
        ("what", "The hot edge widened in 1.2."),
    ]);
    assert_eq!(field(&r, "file"), "issues/2-l3_solar.vissue", "{r}");
    assert_eq!(issues(&copy), ["1-l3_solar.vissue", "2-l3_solar.vissue"]);

    // An issue already on the drive is counted too: the next number is past
    // it, in whichever group.
    std::fs::create_dir_all(drv.join("issues")).unwrap();
    std::fs::write(
        drv.join("issues/6-l3_power.vissue"),
        "an issue someone raised",
    )
    .unwrap();
    let r = raise(&[("node", "sw_f107_design_long"), ("what", "Again.")]);
    assert_eq!(field(&r, "file"), "issues/7-l3_solar.vissue", "{r}");

    post(port, "/v1/key/close", &[]);
    let _ = std::fs::remove_dir_all(&scratch);
}
