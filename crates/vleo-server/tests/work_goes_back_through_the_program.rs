//! A node engineer's signed work put back on the drive from their working
//! copy, through the program's routes, and nothing else (docs/briefs/F3.md,
//! F3-c).
//!
//! The small drive of invented people F3-b's test uses: solar's
//! `sw_activity_band`, assigned to Test Nia, whose deputy is Test Dee; Test
//! Ivo is assigned nothing. Through the routes: nothing goes back without a
//! key or the launch's word; a saved node that is not signed stays in the
//! copy, and so does one someone else changed on the drive since the copy;
//! the signed node goes back only when the files named are the report's, and
//! a copy taken again finds it signed; and a key revoked on the drive since
//! the copy puts nothing back.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::keys::{SigningKey, MIN_ITERATIONS};
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::people::{self, Change, Valve};
use vleo_files::sqlite;

const APP: &str = "vleo 1.0.0 (test)";
const PASSPHRASE: &str = "a long enough passphrase";
const UID: &str = "sw_activity_band";
const NODE: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";
const SIGN: &str = "/v1/node/sw_activity_band/sign";
const SOLAR: &str = "groups/l3_solar/l3_solar.vgroup";

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

/// The file at `path` in `copy`, opened as the program gives it: its digest.
fn opened(port: u16) -> String {
    let r = ask(port, "GET", "/v1/node/sw_activity_band/file", &[], false);
    assert!(r.contains("\"ok\":true"), "{r}");
    field(&r, "opened").to_string()
}

/// `path` in `copy` named as the report gives it: its digest, a space, its
/// path.
fn named(copy: &Path, path: &str) -> String {
    format!(
        "{} {path}",
        vleo_files::copied::digest(&std::fs::read(copy.join(path)).unwrap())
    )
}

/// On the drive, Ola revokes Nia's key from the day before the signing.
fn revoke_nia_on(drv: &Path) {
    let read = |p: &str| sqlite::read(&drv.join(p)).unwrap();
    let anchor = key(71).public().fingerprint();
    let programme =
        chain::open_programme(read("groups/programme/programme.vgroup"), &anchor).unwrap();
    let systems = chain::open_systems(&read("groups/systems/systems.vgroup"), &programme).unwrap();
    let (parent, _) = chain::mounted_by("l3_solar", &[&programme, &systems]).unwrap();
    let valve = Valve::Group {
        mounted_by: parent,
        programme: &programme,
    };
    let yesterday = vleo_units::calendar::Civil::from_unix(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - 86_400,
    )
    .to_string();
    let revoked = people::change(
        &read(SOLAR),
        &valve,
        "Test Ola",
        &key(73),
        &now(),
        &Change::Revoke {
            person: s("Test Nia"),
            public_key: key(74).public().to_hex(),
            from: yesterday[..10].to_string(),
        },
    )
    .unwrap();
    sqlite::write(&revoked, &drv.join(SOLAR)).unwrap();
}

#[test]
fn signed_work_goes_back_to_the_drive_and_nothing_else_does() {
    let scratch = std::env::temp_dir().join(format!("vleo-put-back-route-{}", std::process::id()));
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
    let port = vleo_server::serve(Some(root()), 18955, false, true).expect("no server");
    let early = post(port, "/v1/put-back", &[]);
    assert!(early.contains("no key is open"), "{early}");
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");
    let bare = ask(port, "POST", "/v1/put-back", &[], false);
    assert!(bare.contains("does not carry the word"), "{bare}");
    let on_drive = std::fs::read(drv.join(NODE)).unwrap();

    // Saved, not signed: it stays in the copy, and says why.
    assert!(open_key(port, "test-nia.vkey").contains("\"open\":true"));
    let r = post(
        port,
        "/v1/node/sw_activity_band/save",
        &[("opened", &opened(port)), ("file", &with_band(&copy, "1"))],
    );
    assert!(r.contains("\"revision\":2"), "{r}");
    let r = post(port, "/v1/put-back", &[]);
    assert!(
        r.contains("\"back\":[]") && r.contains("carries no current signature by Test Nia"),
        "{r}"
    );

    // Signed: it would go back, and nothing is written yet.
    let r = post(port, SIGN, &[("opened", &opened(port))]);
    assert!(r.contains("\"ok\":true"), "{r}");
    let r = post(port, "/v1/put-back", &[]);
    assert!(
        r.contains(&format!("\"back\":[\"{NODE}\"]")) && r.contains("\"written\":false"),
        "{r}"
    );
    assert_eq!(std::fs::read(drv.join(NODE)).unwrap(), on_drive);

    // Someone else writes it on the drive meanwhile: it is never written over.
    let mut theirs = on_drive.clone();
    theirs.extend_from_slice(b"someone else's");
    std::fs::write(drv.join(NODE), &theirs).unwrap();
    let r = post(port, "/v1/put-back", &[]);
    assert!(r.contains("changed on the drive since your copy"), "{r}");
    let r = post(port, "/v1/put-back", &[("files", &named(&copy, NODE))]);
    assert!(r.contains("nothing in your copy goes back"), "{r}");
    assert_eq!(std::fs::read(drv.join(NODE)).unwrap(), theirs);
    std::fs::write(drv.join(NODE), &on_drive).unwrap();

    // Only the files the report names are written.
    let r = post(
        port,
        "/v1/put-back",
        &[("files", &named(&copy, "START HERE.md"))],
    );
    assert!(r.contains("not the ones that go back now"), "{r}");
    let r = post(port, "/v1/put-back", &[("files", NODE)]);
    assert!(
        r.contains("does not name a file as the report gave it"),
        "{r}"
    );
    assert_eq!(std::fs::read(drv.join(NODE)).unwrap(), on_drive);
    let r = post(port, "/v1/put-back", &[("files", &named(&copy, NODE))]);
    assert!(r.contains("\"written\":true"), "{r}");
    assert_eq!(
        std::fs::read(drv.join(NODE)).unwrap(),
        std::fs::read(copy.join(NODE)).unwrap()
    );
    let state = ask(port, "GET", "/v1/copy", &[], false);
    assert!(state.contains("\"changed_here\":[]"), "{state}");

    // A copy taken again finds her node signed: My work no longer asks.
    let again = scratch.join("again");
    std::env::set_var("VLEO_COPY", &again);
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");
    let mine = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        mine.contains("\"ok\":true") && !mine.contains("\"kind\":\"unsigned\""),
        "{mine}"
    );
    let f = sqlite::read(&again.join(NODE)).unwrap();
    chain::check_node(&f, UID, &solar(&again)).expect("her signature counts on the drive");

    // Signed again in the new copy; on the drive, her key is revoked from
    // before it. The drive's chain is the one read: nothing goes back.
    let r = post(
        port,
        "/v1/node/sw_activity_band/save",
        &[("opened", &opened(port)), ("file", &with_band(&again, "1"))],
    );
    assert!(r.contains("\"ok\":true"), "{r}");
    let r = post(port, SIGN, &[("opened", &opened(port))]);
    assert!(r.contains("\"ok\":true"), "{r}");
    revoke_nia_on(&drv);
    let r = post(port, "/v1/put-back", &[]);
    assert!(r.contains("\"back\":[]") && r.contains("revoked"), "{r}");

    // Solar's file on the drive swapped for a link to one elsewhere: the chain
    // never reads a group's file through a link, and says so.
    #[cfg(unix)]
    {
        let elsewhere = scratch.join("elsewhere.vgroup");
        std::fs::rename(drv.join(SOLAR), &elsewhere).unwrap();
        std::os::unix::fs::symlink(&elsewhere, drv.join(SOLAR)).unwrap();
        let r = post(port, "/v1/put-back", &[]);
        assert!(r.contains("is reached through a link"), "{r}");
    }

    post(port, "/v1/key/close", &[]);
    let _ = std::fs::remove_dir_all(&scratch);
}
