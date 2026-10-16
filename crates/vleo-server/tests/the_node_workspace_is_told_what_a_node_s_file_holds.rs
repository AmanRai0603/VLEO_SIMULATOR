//! What the Node workspace shows of a node's file, said by the program, since
//! a page cannot read the file's bytes (docs/briefs/F3.md, F3-g1).
//!
//! The drive and the node are the asking test's (`a_node_asks_in_the_copy.rs`):
//! Test Nia assigned solar's `sw_activity_band`, Test Dee her deputy, Test Ivo
//! a node engineer of solar assigned nothing. Through `GET /v1/node/<id>/file`
//! and `GET /v1/my-work`:
//!
//! - to Nia it is hers to write, not signed, and the one next step is to sign
//!   it; to Dee it is hers to write as Nia's deputy; to Ivo it is not his, and
//!   the next step says whose it is; with no key open, the next step is to
//!   open one;
//! - a case saved that disagrees with the method is a finding before anyone
//!   signs, and the next step is to fix it, naming the finding;
//! - a request asked is listed, counting through the chain, and one written by
//!   hand, signed by nobody, listed as not counting, with why;
//! - signed, it says so, and the next step is to put it back;
//! - My work lists the node for Nia and for Dee, as her deputy, signed or not,
//!   and not for Ivo.

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
const UID: &str = "sw_activity_band";
const NODE: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";
const SIGN: &str = "/v1/node/sw_activity_band/sign";
const ASK: &str = "/v1/node/sw_activity_band/ask";

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

fn write_key(dir: &Path, file: &str, person: &str, k: &SigningKey) {
    let locked = k
        .lock(person, PASSPHRASE, [7; 16], [7; 12], MIN_ITERATIONS)
        .unwrap();
    let mut f = File::new(Kind::Key, APP);
    f.meta.insert(s("person"), s(person));
    f.locked_keys.push(locked.to_row());
    sqlite::write(&f, &dir.join(file)).unwrap();
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

/// The node file in the copy with its quietest case, 64 sfu, saying band
/// `band`, as base64: band 1 agrees with the method, band 2 does not.
fn with_band(copy: &Path, band: &str) -> String {
    let at = std::env::temp_dir().join(format!("vleo-ask-edit-{}.vnode", std::process::id()));
    let mut f = sqlite::read(&copy.join(NODE)).unwrap();
    let t = f
        .tables
        .iter_mut()
        .find(|t| t.scope == UID && t.path == "results/isolation.csv")
        .unwrap();
    let line = t
        .csv
        .lines()
        .find(|l| l.starts_with("64,"))
        .unwrap()
        .to_string();
    let mut cells: Vec<&str> = line.splitn(3, ',').collect();
    cells[1] = band;
    t.csv = t.csv.replacen(&line, &cells.join(","), 1);
    sqlite::write(&f, &at).unwrap();
    let out = b64(&std::fs::read(&at).unwrap());
    let _ = std::fs::remove_file(&at);
    out
}

// Test Pat (71), programme manager; Test Sam (72), system engineer; Test Ola
// (73), who owns solar; Test Nia (74), assigned sw_activity_band; Test Ivo
// (75), a node engineer of solar assigned nothing; Test Dee (76), Nia's
// deputy.
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
    // Test Sue (77), Ola's deputy.
    register(
        &mut programme,
        "Test Sue",
        "subsystem engineer",
        &key(77),
        "Test Pat",
    );
    programme.people.last_mut().unwrap().deputy_for = s("Test Ola");
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
    register(
        &mut solar,
        "Test Dee",
        "node engineer",
        &key(76),
        "Test Ola",
    );
    solar.people.last_mut().unwrap().deputy_for = s("Test Nia");
    solar.assignments.push(Assignment {
        block_uid: s(UID),
        person: s("Test Nia"),
        contract_version: 1,
    });
    std::fs::create_dir_all(at.join("groups/l3_solar/nodes")).unwrap();
    sqlite::write(
        &signed(solar, "Test Ola", &key(73)),
        &at.join("groups/l3_solar/l3_solar.vgroup"),
    )
    .unwrap();
    sqlite::write(&node(), &at.join(NODE)).unwrap();
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
    write_key(keys, "test-dee.vkey", "Test Dee", &key(76));
    write_key(keys, "test-ola.vkey", "Test Ola", &key(73));
    write_key(keys, "test-sue.vkey", "Test Sue", &key(77));
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
fn the_node_workspace_is_told_what_a_node_s_file_holds() {
    let scratch = std::env::temp_dir().join(format!("vleo-node-ws-{}", std::process::id()));
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
    let port = vleo_server::serve(Some(root()), 18962, false, true).expect("no server");
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");
    let file = || {
        let r = ask(port, "GET", "/v1/node/sw_activity_band/file", &[], false);
        assert!(r.contains("\"ok\":true"), "{r}");
        r
    };
    let mine = || ask(port, "GET", "/v1/my-work", &[], false);

    // Before any key is opened here, this computer has not accepted the
    // database's anchor: the file is read, and nothing counts yet.
    let r = file();
    assert!(r.contains("\"yours\":false"), "{r}");
    assert!(
        field(&r, "next")
            .starts_with("Nothing yet: this computer has not accepted this database's anchor"),
        "{r}"
    );
    assert_eq!(field(&r, "group"), "l3_solar");

    // Nia: hers, unsigned, and the next step is to sign it.
    assert!(open_key(port, "test-nia.vkey").contains("\"open\":true"));
    let r = file();
    assert!(
        r.contains("\"yours\":true") && r.contains("\"deputy_for\":\"\""),
        "{r}"
    );
    assert!(r.contains("\"signed\":{\"ok\":false"), "{r}");
    assert_eq!(field(&r, "next"), "Sign revision 1", "{r}");
    assert_eq!(field(&r, "assigned"), "Test Nia");
    let work = mine();
    assert!(
        work.contains("\"nodes\":[{\"group\":\"l3_solar\",\"block\":\"sw_activity_band\",\"id\":\"sw_activity_band\",\"revision\":\"1\",\"deputy_for\":\"\",\"signed\":false}]"),
        "{work}"
    );

    // A case that disagrees with the method is found before anyone signs.
    let original = b64(&std::fs::read(copy.join(NODE)).unwrap());
    let r = post(
        port,
        "/v1/node/sw_activity_band/save",
        &[
            ("opened", field(&file(), "opened")),
            ("file", &with_band(&copy, "2")),
        ],
    );
    assert!(r.contains("\"revision\":2"), "{r}");
    let r = file();
    assert!(r.contains("\"level\":\"error\""), "{r}");
    assert!(
        field(&r, "next").starts_with("Fix what its checks find, then sign: sw_activity_band: "),
        "{r}"
    );
    // Put back as it was, and asked of: the request is listed, and counts.
    let r = post(
        port,
        "/v1/node/sw_activity_band/save",
        &[("opened", field(&file(), "opened")), ("file", &original)],
    );
    assert!(r.contains("\"revision\":3"), "{r}");
    let r = post(
        port,
        ASK,
        &[
            ("opened", field(&file(), "opened")),
            ("kind", "breakdown"),
            ("body", "The band is two relations."),
        ],
    );
    assert!(r.contains("\"ok\":true"), "{r}");
    let r = file();
    assert!(
        r.contains("\"uid\":\"sw_activity_band-r1\",\"kind\":\"breakdown\",\"status\":\"open\",\"raised_by\":\"Test Nia\"")
            && r.contains("\"body\":\"The band is two relations.\",\"counts\":true,\"why\":\"\""),
        "{r}"
    );
    assert_eq!(field(&r, "next"), "Sign revision 3", "{r}");

    // Signed, it says so, and the next step is to put it back.
    let r = post(port, SIGN, &[("opened", field(&file(), "opened"))]);
    assert!(r.contains("\"ok\":true"), "{r}");
    let r = file();
    assert!(
        r.contains("\"signed\":{\"ok\":true,\"says\":\"revision 3 is signed as it is now\"}"),
        "{r}"
    );
    assert!(r.contains("\"signer\":\"Test Nia\""), "{r}");
    assert!(r.contains("\"changed_here\":true"), "{r}");
    assert_eq!(
        field(&r, "next"),
        "Put your work back on the drive, from My work"
    );
    assert!(mine().contains("\"revision\":\"3\",\"deputy_for\":\"\",\"signed\":true"));
    post(port, "/v1/key/close", &[]);
    // No key open, the anchor accepted: the next step is to open one.
    let r = file();
    assert!(r.contains("\"yours\":false"), "{r}");
    assert!(field(&r, "next").starts_with("Open your key"), "{r}");

    // A request written by hand, signed by nobody, does not count, and says why.
    let mut forged = sqlite::read(&copy.join(NODE)).unwrap();
    forged.requests.push(Request {
        uid: s("sw_activity_band-r2"),
        kind: s("contract"),
        scope: s(UID),
        raised_by: s("Test Nia"),
        at: s("2026-10-16T08:00:00Z"),
        body: s("By hand."),
        status: s("open"),
        answer: s(""),
    });
    sqlite::write(&forged, &copy.join(NODE)).unwrap();

    // Dee, Nia's deputy: hers to write, as Nia's deputy.
    assert!(open_key(port, "test-dee.vkey").contains("\"open\":true"));
    let r = file();
    assert!(
        r.contains("\"yours\":true") && r.contains("\"deputy_for\":\"Test Nia\""),
        "{r}"
    );
    assert!(
        r.contains("\"body\":\"By hand.\",\"counts\":false,\"why\":\"")
            && !r.contains("\"why\":\"\"}]"),
        "{r}"
    );
    assert!(mine()
        .contains("\"id\":\"sw_activity_band\",\"revision\":\"3\",\"deputy_for\":\"Test Nia\""));
    post(port, "/v1/key/close", &[]);

    // Ivo: not his, and the next step says whose it is. My work lists no node.
    assert!(open_key(port, "test-ivo.vkey").contains("\"open\":true"));
    let r = file();
    assert!(r.contains("\"yours\":false"), "{r}");
    assert!(
        field(&r, "next").starts_with("Nothing: sw_activity_band is Test Nia's to write"),
        "{r}"
    );
    assert!(mine().contains("\"nodes\":[]"));
    post(port, "/v1/key/close", &[]);
    let _ = std::fs::remove_dir_all(&scratch);
}
