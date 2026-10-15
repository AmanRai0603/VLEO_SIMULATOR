//! A node's file opened from the working copy and saved back to it as the
//! next revision, through the program's routes (docs/briefs/F3.md, F3-a).
//!
//! A small drive of invented people, with the design's own `sw_ap_design`
//! in solar's nodes, assigned to Test Nia. Test Ivo is a node engineer of
//! solar assigned nothing. Through the routes: the node opened; saved as
//! revision 2 by Nia; a save from what she opened before that answered with
//! both versions and nothing written; and refused without a key, without the
//! launch's word, for someone not assigned, for an edit that is not a node
//! file, and for a node the copy does not hold.

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
const NODE: &str = "groups/l3_solar/nodes/sw_ap_design.vnode";

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

fn b64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16
            | (*c.get(1).unwrap_or(&0) as u32) << 8
            | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            out.push(if k <= c.len() {
                A[(n >> (18 - 6 * k) & 63) as usize] as char
            } else {
                '='
            });
        }
    }
    out
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

fn write_key(dir: &Path, file: &str, person: &str, k: &SigningKey) {
    let locked = k
        .lock(person, PASSPHRASE, [7; 16], [7; 12], MIN_ITERATIONS)
        .unwrap();
    let mut f = File::new(Kind::Key, APP);
    f.meta.insert(s("person"), s(person));
    f.locked_keys.push(locked.to_row());
    sqlite::write(&f, &dir.join(file)).unwrap();
}

// Test Pat (71), programme manager; Test Sam (72), system engineer; Test Ola
// (73), who owns solar; Test Nia (74), assigned sw_ap_design; Test Ivo (75),
// a node engineer of solar assigned nothing.
fn drive(at: &Path, keys: &Path) {
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
    std::fs::create_dir_all(at.join("groups/programme")).unwrap();
    sqlite::write(
        &signed(programme, "Test Pat", &key(71)),
        &at.join("groups/programme/programme.vgroup"),
    )
    .unwrap();
    let mut systems = group("systems", "Test Sam");
    mount(
        &mut systems,
        "sys_space_environment",
        "l3_solar",
        "Test Ola",
    );
    std::fs::create_dir_all(at.join("groups/systems")).unwrap();
    sqlite::write(
        &signed(systems, "Test Sam", &key(72)),
        &at.join("groups/systems/systems.vgroup"),
    )
    .unwrap();
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
        block_uid: s("sw_ap_design"),
        person: s("Test Nia"),
        contract_version: 1,
    });
    std::fs::create_dir_all(at.join("groups/l3_solar/nodes")).unwrap();
    sqlite::write(
        &signed(solar, "Test Ola", &key(73)),
        &at.join("groups/l3_solar/l3_solar.vgroup"),
    )
    .unwrap();
    std::fs::copy(root().join("design").join(NODE), at.join(NODE)).unwrap();
    std::fs::write(
        at.join("START HERE.md"),
        format!(
            "# START HERE: an invented drive\n\nThe programme manager's key:\n\n    {}\n",
            key(71).public().fingerprint()
        ),
    )
    .unwrap();
    std::fs::create_dir_all(keys).unwrap();
    write_key(keys, "test-nia.vkey", "Test Nia", &key(74));
    write_key(keys, "test-ivo.vkey", "Test Ivo", &key(75));
}

/// The node file as the program gave it, edited with a line of the writer's
/// own, as base64.
fn edited(copy: &Path, line: &str) -> String {
    let at = std::env::temp_dir().join(format!("vleo-node-edit-{}.vnode", std::process::id()));
    let mut f = sqlite::read(&copy.join(NODE)).unwrap();
    // One text of each kind for a block: the note is the writer's line.
    f.texts
        .retain(|t| !(t.scope == "sw_ap_design" && t.kind == "note"));
    f.texts.push(Text {
        scope: s("sw_ap_design"),
        kind: s("note"),
        body: s(line),
    });
    sqlite::write(&f, &at).unwrap();
    let out = b64(&std::fs::read(&at).unwrap());
    let _ = std::fs::remove_file(&at);
    out
}

fn open_key(port: u16, file: &str) -> String {
    post(
        port,
        "/v1/key/open",
        &[
            ("file", file),
            ("passphrase", PASSPHRASE),
            ("accept_anchor", &key(71).public().fingerprint()),
        ],
    )
}

#[test]
fn a_node_is_saved_in_the_copy_and_a_second_writer_stopped() {
    let scratch = std::env::temp_dir().join(format!("vleo-node-save-{}", std::process::id()));
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
    std::env::remove_var("VLEO_DRIVE");
    std::env::remove_var("VLEO_DESIGN");
    let port = vleo_server::serve(Some(root()), 18951, false, true).expect("no server");

    let early = ask(port, "GET", "/v1/node/sw_ap_design/file", &[], false);
    assert!(early.contains("copy it from the drive first"), "{early}");
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");

    let opened = ask(port, "GET", "/v1/node/sw_ap_design/file", &[], false);
    assert!(opened.contains("\"ok\":true"), "{opened}");
    assert_eq!(field(&opened, "path"), NODE);
    assert_eq!(field(&opened, "revision"), "1");
    let first = field(&opened, "opened").to_string();
    for (id, says) in [("no_such_row", "no node file"), ("../x", "not a node's id")] {
        let r = ask(port, "GET", &format!("/v1/node/{id}/file"), &[], false);
        assert!(r.contains("\"ok\":false") && r.contains(says), "{id}: {r}");
    }

    let mine = edited(&copy, "A line of Test Nia's own.");
    let save = |pairs: &[(&str, &str)]| post(port, "/v1/node/sw_ap_design/save", pairs);
    let before = std::fs::read(copy.join(NODE)).unwrap();
    let no_key = save(&[("opened", &first), ("file", &mine)]);
    assert!(no_key.contains("no key is open"), "{no_key}");
    let bare = ask(
        port,
        "POST",
        "/v1/node/sw_ap_design/save",
        &[("opened", &first), ("file", &mine)],
        false,
    );
    assert!(bare.contains("does not carry the word"), "{bare}");

    // Ivo is a node engineer of solar, and not assigned the node.
    assert!(open_key(port, "test-ivo.vkey").contains("\"open\":true"));
    let ivo = save(&[("opened", &first), ("file", &mine)]);
    assert!(
        ivo.contains("Test Ivo is not assigned sw_ap_design"),
        "{ivo}"
    );
    post(port, "/v1/key/close", &[]);
    assert_eq!(
        std::fs::read(copy.join(NODE)).unwrap(),
        before,
        "nothing was written"
    );

    assert!(open_key(port, "test-nia.vkey").contains("\"open\":true"));
    let junk = save(&[("opened", &first), ("file", "bm90IGEgbm9kZQ==")]);
    assert!(junk.contains("the edit is not a node file"), "{junk}");
    let r = save(&[("opened", &first), ("file", &mine)]);
    assert!(r.contains("\"ok\":true"), "{r}");
    assert!(r.contains("\"revision\":2"), "{r}");
    assert!(
        r.contains("is saved as revision 2, written by Test Nia"),
        "{r}"
    );
    let now = sqlite::read(&copy.join(NODE)).unwrap();
    assert_eq!(now.meta.get("revision").map(String::as_str), Some("2"));
    assert_eq!(now.meta.get("writer").map(String::as_str), Some("Test Nia"));
    assert!(
        !drv.join(NODE).exists()
            || std::fs::read(drv.join(NODE)).unwrap() != std::fs::read(copy.join(NODE)).unwrap(),
        "the drive is never written"
    );

    // A save from what she opened before her first save went in: a second
    // writer, answered with both, nothing written.
    let saved = std::fs::read(copy.join(NODE)).unwrap();
    let stale = edited(&copy, "Another line.");
    let r = save(&[("opened", &first), ("file", &stale)]);
    assert!(r.contains("\"conflict\":true"), "{r}");
    assert!(r.contains("Test Nia saved revision 2"), "{r}");
    assert_eq!(field(&r, "yours"), stale);
    assert_eq!(field(&r, "theirs"), b64(&saved));
    assert_eq!(
        std::fs::read(copy.join(NODE)).unwrap(),
        saved,
        "nothing was written"
    );

    post(port, "/v1/key/close", &[]);
    let _ = std::fs::remove_dir_all(&scratch);
}
