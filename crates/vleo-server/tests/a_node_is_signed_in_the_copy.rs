//! A node signed as the day's work through the program's routes, checked at
//! once through the chain (docs/briefs/F3.md, F3-b).
//!
//! A small drive of invented people, with solar's `sw_activity_band`, as
//! solar's 1.1 release holds it (`crates/vleo-files/tests/fixtures`), in
//! solar's nodes, assigned to Test Nia. Test Dee is her deputy; Test Ivo is a
//! node engineer of solar assigned nothing. Through the routes: Ivo is refused
//! by name; Nia signs, and her signature checks through the chain; a signing
//! from what she opened before is stopped; she saves a case that disagrees,
//! and signing it is refused with the check named; Dee saves it put right and
//! signs it for her. Refused without the launch's word and without a key.

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
    let at = std::env::temp_dir().join(format!("vleo-sign-edit-{}.vnode", std::process::id()));
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
fn a_node_is_signed_by_its_engineer_or_their_deputy_and_by_no_one_else() {
    let scratch = std::env::temp_dir().join(format!("vleo-node-sign-{}", std::process::id()));
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
    let port = vleo_server::serve(Some(root()), 18953, false, true).expect("no server");
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");
    let on_drive = std::fs::read(drv.join(NODE)).unwrap();
    let opened = |port: u16| {
        let r = ask(port, "GET", "/v1/node/sw_activity_band/file", &[], false);
        assert!(r.contains("\"ok\":true"), "{r}");
        field(&r, "opened").to_string()
    };
    let first = opened(port);

    let no_key = post(port, SIGN, &[("opened", &first)]);
    assert!(no_key.contains("no key is open"), "{no_key}");
    let bare = ask(port, "POST", SIGN, &[("opened", &first)], false);
    assert!(bare.contains("does not carry the word"), "{bare}");

    // Ivo is a node engineer of solar, and not assigned the node.
    assert!(open_key(port, "test-ivo.vkey").contains("\"open\":true"));
    let ivo = post(port, SIGN, &[("opened", &first)]);
    assert!(
        ivo.contains("Test Ivo is not assigned sw_activity_band"),
        "{ivo}"
    );
    post(port, "/v1/key/close", &[]);

    // Nia signs what she opened, and it checks through the chain.
    assert!(open_key(port, "test-nia.vkey").contains("\"open\":true"));
    let (before, r, after) = (now(), post(port, SIGN, &[("opened", &first)]), now());
    assert!(r.contains("\"ok\":true"), "{r}");
    assert_eq!(field(&r, "signed_by"), "Test Nia");
    assert!(r.contains("\"revision\":1"), "{r}");
    assert!(r.contains("\"newer_on_drive\":[]"), "{r}");
    let signed = sqlite::read(&copy.join(NODE)).unwrap();
    chain::check_node(&signed, UID, &solar(&copy)).expect("her signature counts");
    // Signed at this moment, by the program's clock, as the row records it.
    let at = field(&r, "signed_at").to_string();
    assert!(before <= at && at <= after, "{before} {at} {after}");
    assert_eq!(signed.signatures.last().unwrap().signed_at, at);
    // What the next action is held to is the file as it now is.
    assert_eq!(
        field(&r, "opened"),
        vleo_files::copied::digest(&std::fs::read(copy.join(NODE)).unwrap())
    );
    // What she opened before her signing is not what is there now.
    let stale = post(port, SIGN, &[("opened", &first)]);
    assert!(stale.contains("\"conflict\":true"), "{stale}");

    // She saves a case that disagrees with the method; signing it is refused,
    // with the check named, and nothing is written.
    let r = post(
        port,
        "/v1/node/sw_activity_band/save",
        &[("opened", &opened(port)), ("file", &with_band(&copy, "2"))],
    );
    assert!(r.contains("\"revision\":2"), "{r}");
    let before = std::fs::read(copy.join(NODE)).unwrap();
    let r = post(port, SIGN, &[("opened", &opened(port))]);
    assert!(
        r.contains("sw_activity_band is not signed: its checks refuse it")
            && r.contains("does not reproduce"),
        "{r}"
    );
    assert_eq!(std::fs::read(copy.join(NODE)).unwrap(), before);
    post(port, "/v1/key/close", &[]);

    // A group file changed on the drive since the copy is said beside it.
    let solar_on_drive = drv.join("groups/l3_solar/l3_solar.vgroup");
    let mut changed = std::fs::read(&solar_on_drive).unwrap();
    changed.extend_from_slice(b"changed on the drive");
    std::fs::write(&solar_on_drive, changed).unwrap();

    // Dee, her deputy, puts it right and signs it for her.
    assert!(open_key(port, "test-dee.vkey").contains("\"open\":true"));
    let r = post(
        port,
        "/v1/node/sw_activity_band/save",
        &[("opened", &opened(port)), ("file", &with_band(&copy, "1"))],
    );
    assert!(r.contains("\"revision\":3"), "{r}");
    let r = post(port, SIGN, &[("opened", &opened(port))]);
    assert!(r.contains("\"ok\":true"), "{r}");
    assert_eq!(field(&r, "signed_by"), "Test Dee");
    assert!(
        r.contains("\"newer_on_drive\":[\"groups/l3_solar/l3_solar.vgroup\"]"),
        "{r}"
    );
    let signed = sqlite::read(&copy.join(NODE)).unwrap();
    chain::check_node(&signed, UID, &solar(&copy)).expect("the deputy's signature counts");
    assert_eq!(
        std::fs::read(drv.join(NODE)).unwrap(),
        on_drive,
        "the drive is never written"
    );

    post(port, "/v1/key/close", &[]);
    let _ = std::fs::remove_dir_all(&scratch);
}
