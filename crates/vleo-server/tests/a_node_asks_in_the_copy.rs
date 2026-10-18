//! A node engineer asks, through the program's routes, for a contract change
//! or a breakdown, written as a request in the node's file in the copy and
//! signed by them (docs/briefs/F3.md, F3-d).
//!
//! The drive and the node are the signing test's (`a_node_is_signed_in_the_
//! copy.rs`): Test Nia assigned solar's `sw_activity_band`, Test Dee her
//! deputy, Test Ivo assigned nothing, Test Ola solar's subsystem engineer.
//! Through the routes: Ivo is refused by name; Nia's node fails a case, so she
//! cannot sign it, and she asks all the same; a request of the code, or one
//! that says nothing, is refused; asking from what she opened before is
//! stopped. Ola's My work then holds the request, and a request written in the
//! copy by hand, signed by nobody, is said as refused. Refused without the
//! launch's word and without a key.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use vleo_files::chain::{self, Registry, FILE};
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

/// Now, as the program writes a moment (`results::now_utc`).
fn now() -> String {
    vleo_units::calendar::Civil::from_unix(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    )
    .to_string()
}

/// Solar's group file in the copy, as the chain reads it for this test.
fn solar(copy: &Path) -> Registry {
    Registry {
        from: s("l3_solar's group file"),
        file: sqlite::read(&copy.join("groups/l3_solar/l3_solar.vgroup")).unwrap(),
    }
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
fn a_node_engineer_asks_in_the_node_s_file_and_its_subsystem_engineer_sees_it() {
    let scratch = std::env::temp_dir().join(format!("vleo-node-ask-{}", std::process::id()));
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
    let port = vleo_server::serve(Some(root()), 18957, false, true).expect("no server");
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");
    let on_drive = std::fs::read(drv.join(NODE)).unwrap();
    let opened = |port: u16| {
        let r = ask(port, "GET", "/v1/node/sw_activity_band/file", &[], false);
        assert!(r.contains("\"ok\":true"), "{r}");
        field(&r, "opened").to_string()
    };
    let words = "The band's floor reads F10.7 as daily: it needs the 81-day mean.";
    let asking = |port: u16, opened: &str, kind: &str, body: &str| {
        post(
            port,
            ASK,
            &[("opened", opened), ("kind", kind), ("body", body)],
        )
    };

    let no_key = asking(port, &opened(port), "contract", words);
    assert!(no_key.contains("no key is open"), "{no_key}");
    let bare = ask(
        port,
        "POST",
        ASK,
        &[
            ("opened", &opened(port)),
            ("kind", "contract"),
            ("body", words),
        ],
        false,
    );
    assert!(bare.contains("does not carry the word"), "{bare}");

    // Ivo is a node engineer of solar, and not assigned the node.
    assert!(open_key(port, "test-ivo.vkey").contains("\"open\":true"));
    let ivo = asking(port, &opened(port), "contract", words);
    assert!(
        ivo.contains("Test Ivo is not assigned sw_activity_band"),
        "{ivo}"
    );
    post(port, "/v1/key/close", &[]);

    // Nia saves a case that disagrees with the method: her node cannot be
    // signed, and she asks all the same.
    assert!(open_key(port, "test-nia.vkey").contains("\"open\":true"));
    let r = post(
        port,
        "/v1/node/sw_activity_band/save",
        &[("opened", &opened(port)), ("file", &with_band(&copy, "2"))],
    );
    assert!(r.contains("\"revision\":2"), "{r}");
    let r = post(port, SIGN, &[("opened", &opened(port))]);
    assert!(r.contains("its checks refuse it"), "{r}");
    let first = opened(port);
    for (kind, body, says) in [
        ("wish", words, "is none of them"),
        ("contract", "  ", "says nothing"),
    ] {
        let r = asking(port, &first, kind, body);
        assert!(r.contains(says), "{kind}: {r}");
    }
    let (before, r, after) = (now(), asking(port, &first, "contract", words), now());
    assert!(r.contains("\"ok\":true"), "{r}");
    assert_eq!(field(&r, "asked_by"), "Test Nia");
    assert_eq!(field(&r, "kind"), "contract");
    let uid = field(&r, "request").to_string();
    let at = field(&r, "asked_at").to_string();
    assert!(before <= at && at <= after, "{before} {at} {after}");
    let f = sqlite::read(&copy.join(NODE)).unwrap();
    chain::check_request(&f, &uid, &solar(&copy)).expect("her request counts");
    assert_eq!(f.requests[0].at, at);
    assert_eq!(f.requests[0].body, words);
    assert_eq!(
        f.meta.get("revision").map(String::as_str),
        Some("2"),
        "asking is not a revision"
    );
    assert_eq!(
        field(&r, "opened"),
        vleo_files::copied::digest(&std::fs::read(copy.join(NODE)).unwrap())
    );
    // What she opened before is not what is there now.
    let stale = asking(port, &first, "breakdown", words);
    assert!(stale.contains("\"conflict\":true"), "{stale}");
    post(port, "/v1/key/close", &[]);

    // A request written by hand in the copy, in Nia's name, signed by nobody.
    let mut forged = sqlite::read(&copy.join(NODE)).unwrap();
    forged.requests.push(Request {
        uid: s("by-hand"),
        kind: s("breakdown"),
        scope: s(UID),
        raised_by: s("Test Nia"),
        at: s("2026-10-16T08:00:00Z"),
        body: s("Split it in two."),
        status: s("open"),
        answer: s(""),
    });
    sqlite::write(&forged, &copy.join(NODE)).unwrap();

    // Ola, solar's subsystem engineer, finds Nia's request in My work, and
    // the forged one said as refused.
    assert!(open_key(port, "test-ola.vkey").contains("\"open\":true"));
    let work = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        work.contains("\"kind\":\"request\"")
            && work.contains("Test Nia asks for a contract change of sw_activity_band")
            && work.contains("81-day mean"),
        "{work}"
    );
    assert!(
        work.contains("\"kind\":\"request refused\"")
            && work.contains("the request \\\"by-hand\\\" is not named for the node"),
        "{work}"
    );
    assert_eq!(
        work.matches("\"kind\":\"request\"").count(),
        1,
        "the forged one is not listed as asked: {work}"
    );
    post(port, "/v1/key/close", &[]);
    // Sue, Ola's deputy, finds it too.
    assert!(open_key(port, "test-sue.vkey").contains("\"open\":true"));
    let work = ask(port, "GET", "/v1/my-work", &[], false);
    assert_eq!(work.matches("\"kind\":\"request\"").count(), 1, "{work}");
    post(port, "/v1/key/close", &[]);
    // Said to be declined by hand, Nia's request is not taken as answered:
    // no answer to it is signed, so it is said as refused, not left out.
    let mut said = sqlite::read(&copy.join(NODE)).unwrap();
    for r in said.requests.iter_mut().filter(|r| r.uid == uid) {
        r.status = s("declined");
    }
    sqlite::write(&said, &copy.join(NODE)).unwrap();
    assert!(open_key(port, "test-ola.vkey").contains("\"open\":true"));
    let work = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        !work.contains("\"kind\":\"request\"")
            && work.contains("says it is declined, and no answer to it is signed yet"),
        "{work}"
    );
    post(port, "/v1/key/close", &[]);
    // Nia is not solar's owner: her My work holds no request.
    assert!(open_key(port, "test-nia.vkey").contains("\"open\":true"));
    let work = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(!work.contains("\"kind\":\"request"), "{work}");
    post(port, "/v1/key/close", &[]);
    assert_eq!(
        std::fs::read(drv.join(NODE)).unwrap(),
        on_drive,
        "the drive is never written"
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
