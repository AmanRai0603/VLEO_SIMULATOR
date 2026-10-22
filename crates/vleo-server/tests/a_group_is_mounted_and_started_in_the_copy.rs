//! A group is mounted and given what it is allocated by the system engineer,
//! and started by the owner the mount names, through the program
//! (docs/briefs/F6.md, F6-a; W1, W13; `vleo_files::frame`, `start`).
//!
//! Invented people: Test Pat the programme manager, Test Sam the system
//! engineer, Test Ola who owns solar, Test Rui a subsystem engineer who owns
//! nothing yet. The systems' file holds solar's mount and a leaf,
//! `sys_radiation`, with the output the new group answers; solar's group
//! file holds a leaf of its own, `sw_activity_count`. Through
//! `GET /v1/branch`, `POST /v1/branch/mount`, `POST /v1/branch/allocate`,
//! `POST /v1/group/start` and `GET /v1/my-work`:
//!
//! - to Rui, the systems' branch may not be mounted on, and Rui's mount is
//!   refused with nothing written;
//! - Sam mounts `l3_radiation` on `sys_radiation` for Rui; the systems'
//!   branch then says it below, not started; allocated nothing, My work says
//!   *Allocate l3_radiation* to Sam, and to Rui it is not yet Rui's to
//!   start, and a start is refused (F6's question 43); an allocation with no
//!   sense is refused, and one with `<=` is made;
//! - to Sam, `l3_radiation` is mounted for Rui, not started, with its
//!   allocation and its sense, and the system engineer above it; Sam's start
//!   is refused, and My work says it is Rui's step;
//! - to Rui, a folder holding something is said as refused, not as Start;
//!   emptied, My work says *Start l3_radiation*; Rui starts it, and its
//!   branch is then started and opens through the chain; started again, it
//!   is refused;
//! - W13: Sam mounts `l3_activity` on solar's `sw_activity_count` for Rui;
//!   to Ola, the leaf carries it, and Ola's fill of it is refused naming the
//!   mount; the valve above `l3_activity` is Ola;
//! - a start without the launch's word, or with no key open, is refused;
//! - a file of the copy that cannot be read: the frame says so, and why;
//! - the drive holds what it held when it was copied; then Sam puts the
//!   systems' file back, and the drive's holds both mounts (F6-e).

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
const SYSTEMS: &str = "groups/systems/systems.vgroup";
const SOLAR: &str = "groups/l3_solar/l3_solar.vgroup";
const NEW: &str = "groups/l3_radiation/l3_radiation.vgroup";
const WHY: &str = "Radiation is a subsystem's question now: its dose is shielded by mass.";

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

fn drive(at: &Path, keys: &Path) {
    let mut programme = group("programme", "Test Pat");
    for (name, role, k) in [
        ("Test Pat", "programme manager", 81),
        ("Test Sam", "system engineer", 82),
        ("Test Ola", "subsystem engineer", 83),
        ("Test Rui", "subsystem engineer", 87),
    ] {
        register(&mut programme, name, role, &key(k), "Test Pat");
    }
    mount(&mut programme, "root", "systems", "Test Sam");
    std::fs::create_dir_all(at.join("groups/programme")).unwrap();
    sqlite::write(
        &signed(programme, "Test Pat", &key(81)),
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
    systems.blocks.push(Block {
        uid: s("sys_radiation"),
        id: s("sys_radiation"),
        behaviour: s("open"),
        question: s("What dose does the payload take over the mission?"),
        contract_version: 1,
        ..Block::default()
    });
    systems.ports.push(Port {
        block_uid: s("sys_radiation"),
        direction: s("out"),
        name: s("shield_mass"),
        port_type: s("number"),
        unit: s("kg"),
        state: s("achieved"),
        ..Port::default()
    });
    std::fs::create_dir_all(at.join("groups/systems")).unwrap();
    sqlite::write(&signed(systems, "Test Sam", &key(82)), &at.join(SYSTEMS)).unwrap();
    let mut solar = group("l3_solar", "Test Ola");
    solar.blocks.push(Block {
        uid: s("l3_solar"),
        id: s("l3_solar"),
        behaviour: s("children"),
        question: s("What solar weather does the design face?"),
        ..Block::default()
    });
    solar.blocks.push(Block {
        uid: s("sw_activity_count"),
        id: s("sw_activity_count"),
        parent_uid: s("l3_solar"),
        behaviour: s("open"),
        question: s("How many active days a year?"),
        contract_version: 1,
        ..Block::default()
    });
    std::fs::create_dir_all(at.join("groups/l3_solar")).unwrap();
    sqlite::write(&signed(solar, "Test Ola", &key(83)), &at.join(SOLAR)).unwrap();
    std::fs::write(
        at.join("START HERE.md"),
        format!(
            "# START HERE: an invented drive\n\nThe programme manager's key:\n\n    {}\n",
            key(81).public().fingerprint()
        ),
    )
    .unwrap();
    std::fs::create_dir_all(keys).unwrap();
    write_key(keys, "test-sam.vkey", "Test Sam", &key(82));
    write_key(keys, "test-ola.vkey", "Test Ola", &key(83));
    write_key(keys, "test-rui.vkey", "Test Rui", &key(87));
}

fn open_key(port: u16, file: &str) -> String {
    post(
        port,
        "/v1/key/open",
        &[
            ("file", file),
            ("passphrase", PASSPHRASE),
            ("accept_anchor", &key(81).public().fingerprint()),
        ],
    )
}

#[test]
fn a_group_is_mounted_and_started_in_the_copy() {
    let scratch = std::env::temp_dir().join(format!("vleo-valve-{}", std::process::id()));
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
    let port = vleo_server::serve(Some(root()), 18973, false, true).expect("no server");
    let tree = |g: &str| ask(port, "GET", &format!("/v1/branch?group={g}"), &[], false);
    let opened = || vleo_files::copied::digest(&std::fs::read(copy.join(SYSTEMS)).unwrap());
    let frame = |edit: &str, pairs: &[(&str, &str)]| {
        let o = opened();
        let mut all = vec![("group", "systems"), ("why", WHY), ("opened", o.as_str())];
        all.extend_from_slice(pairs);
        post(port, &format!("/v1/branch/{edit}"), &all)
    };
    let systems_bytes = || std::fs::read(copy.join(SYSTEMS)).unwrap();
    let copied = post(port, "/v1/copy", &[("drive", &drv.display().to_string())]);
    assert!(copied.contains("\"ok\":true"), "{copied}");
    let on_drive: Vec<Vec<u8>> = [SYSTEMS, SOLAR]
        .iter()
        .map(|p| std::fs::read(drv.join(p)).unwrap())
        .collect();

    // A start with no key open is refused.
    let r = post(port, "/v1/group/start", &[("group", "l3_radiation")]);
    assert!(field(&r, "message").starts_with("no key is open"), "{r}");

    // Rui: the systems' branch is not Rui's to mount on; Rui's mount is refused.
    assert!(open_key(port, "test-rui.vkey").contains("\"open\":true"));
    let r = tree("systems");
    assert!(r.contains("\"may_mount\":false"), "{r}");
    let before = systems_bytes();
    let r = frame(
        "mount",
        &[
            ("block", "sys_radiation"),
            ("mounts", "l3_radiation"),
            ("owner", "Test Rui"),
        ],
    );
    assert!(
        field(&r, "message").contains("may not change systems's group file"),
        "{r}"
    );
    assert_eq!(systems_bytes(), before, "a refused mount writes nothing");

    // Sam mounts radiation for Rui, and allocates it.
    assert!(open_key(port, "test-sam.vkey").contains("\"open\":true"));
    let r = tree("systems");
    assert!(r.contains("\"may_mount\":true"), "{r}");
    assert!(r.contains("\"group\":\"l3_solar\",\"block\":\"sys_space_environment\",\"owner\":\"Test Ola\",\"started\":true"), "{r}");
    let r = frame(
        "mount",
        &[
            ("block", "sys_radiation"),
            ("mounts", "l3_radiation"),
            ("owner", "Test Rui"),
        ],
    );
    assert!(
        r.contains("\"ok\":true") && field(&r, "edit") == "mount",
        "{r}"
    );
    let r = tree("systems");
    assert!(r.contains("\"group\":\"l3_radiation\",\"block\":\"sys_radiation\",\"owner\":\"Test Rui\",\"started\":false"), "{r}");
    // Allocated nothing yet: the system engineer's step first (F6's question
    // 43). To Sam, My work says Allocate; to Rui, it is not Rui's to start
    // yet, and a start is refused.
    let wanted = "the systems' file allocates l3_radiation nothing yet";
    let w = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        w.contains("\"kind\":\"allocate\",\"group\":\"l3_radiation\",\"node\":\"sys_radiation\"")
            && w.contains(&format!(
                "Allocate l3_radiation: it is mounted for Test Rui, and {wanted}"
            )),
        "{w}"
    );
    let r = tree("l3_radiation");
    assert!(field(&r, "next").contains(wanted), "{r}");
    assert!(open_key(port, "test-rui.vkey").contains("\"open\":true"));
    let r = tree("l3_radiation");
    assert!(
        r.contains("\"yours\":false")
            && r.contains("\"allocated\":false")
            && field(&r, "why_not").starts_with(&format!("Not yet: {wanted}")),
        "{r}"
    );
    let w = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        w.contains("\"kind\":\"not started\",\"group\":\"l3_radiation\"")
            && !w.contains("\"kind\":\"start\""),
        "{w}"
    );
    let r = post(port, "/v1/group/start", &[("group", "l3_radiation")]);
    assert!(field(&r, "message").contains(wanted), "{r}");
    assert!(!copy.join(NEW).exists());
    assert!(open_key(port, "test-sam.vkey").contains("\"open\":true"));
    let alloc = |sense: &str| {
        frame(
            "allocate",
            &[
                ("block", "sys_radiation"),
                ("name", "shield_mass_allocated"),
                ("type", "number"),
                ("unit", "kg"),
                ("value", "12"),
                ("bounds", "shield_mass"),
                ("sense", sense),
                (
                    "closure_says",
                    "the shield stays within the mass it is given",
                ),
            ],
        )
    };
    let before = systems_bytes();
    let r = alloc("");
    assert!(
        field(&r, "message").contains("say which way shield_mass_allocated binds shield_mass"),
        "{r}"
    );
    assert_eq!(
        systems_bytes(),
        before,
        "an allocation with no sense writes nothing"
    );
    let r = alloc("<=");
    assert!(r.contains("\"ok\":true"), "{r}");

    // To Sam, radiation is mounted for Rui and not started, with what it is
    // allocated and who is above it; Sam does not start it.
    let r = tree("l3_radiation");
    assert!(
        r.contains("\"started\":false") && r.contains("\"yours\":false"),
        "{r}"
    );
    assert_eq!(
        field(&r, "next"),
        "Nothing yet: l3_radiation is mounted for Test Rui, who starts its file"
    );
    assert!(
        r.contains("\"valve_above\":{\"branch\":\"systems\",\"owner\":\"Test Sam\"}"),
        "{r}"
    );
    assert!(
        r.contains("\"name\":\"shield_mass_allocated\",\"value\":\"12\",\"lower\":\"\",\"upper\":\"\",\"unit\":\"kg\",\"bounds\":\"shield_mass\",\"sense\":\"<=\""),
        "{r}"
    );
    let r = post(port, "/v1/group/start", &[("group", "l3_radiation")]);
    assert!(field(&r, "message").contains("its owner Test Rui"), "{r}");
    assert!(!copy.join(NEW).exists());
    // To the system engineer, My work says whose step it is.
    let w = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        w.contains("\"kind\":\"not started\",\"group\":\"l3_radiation\"")
            && w.contains("l3_radiation is mounted for Test Rui, who starts its file"),
        "{w}"
    );

    // Rui: My work says Start, and Rui starts it.
    assert!(open_key(port, "test-rui.vkey").contains("\"open\":true"));
    // A folder that holds something already is not a group to start: said
    // as the refusal it is, never offered as Start.
    std::fs::create_dir_all(copy.join("groups/l3_radiation")).unwrap();
    std::fs::write(copy.join("groups/l3_radiation/notes.txt"), "kept").unwrap();
    let w = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        !w.contains("\"kind\":\"start\"")
            && w.contains("\"kind\":\"refused\",\"group\":\"l3_radiation\""),
        "{w}"
    );
    let r = tree("l3_radiation");
    assert!(r.contains("\"ok\":false"), "{r}");
    std::fs::remove_file(copy.join("groups/l3_radiation/notes.txt")).unwrap();
    let w = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        w.contains("\"kind\":\"start\",\"group\":\"l3_radiation\""),
        "{w}"
    );
    let r = tree("l3_radiation");
    assert!(
        r.contains("\"yours\":true")
            && r.contains("\"allocated\":true")
            && field(&r, "next").starts_with("Start l3_radiation"),
        "{r}"
    );
    // Without the launch's word, nothing is started.
    let r = ask(
        port,
        "POST",
        "/v1/group/start",
        &[("group", "l3_radiation")],
        false,
    );
    assert!(r.contains("\"ok\":false"), "{r}");
    assert!(!copy.join(NEW).exists());
    let r = post(port, "/v1/group/start", &[("group", "l3_radiation")]);
    assert!(r.contains("\"ok\":true"), "{r}");
    assert_eq!(
        field(&r, "opened"),
        vleo_files::copied::digest(&std::fs::read(copy.join(NEW)).unwrap())
    );
    let r = tree("l3_radiation");
    assert!(
        r.contains("\"started\":true") && r.contains("\"yours\":true"),
        "{r}"
    );
    assert!(r.contains("\"signed\":{\"ok\":true"), "{r}");
    assert!(r.contains("\"uid\":\"l3_radiation\""), "{r}");
    let r = post(port, "/v1/group/start", &[("group", "l3_radiation")]);
    assert!(
        field(&r, "message").contains("l3_radiation is started already"),
        "{r}"
    );

    // W13: on solar's leaf, for Rui; Ola above it, and Ola's fill refused.
    assert!(open_key(port, "test-sam.vkey").contains("\"open\":true"));
    let r = frame(
        "mount",
        &[
            ("block", "sw_activity_count"),
            ("mounts", "l3_activity"),
            ("owner", "Test Rui"),
        ],
    );
    assert!(r.contains("\"ok\":true"), "{r}");
    let r = tree("l3_activity");
    assert!(
        r.contains("\"valve_above\":{\"branch\":\"l3_solar\",\"owner\":\"Test Ola\"}"),
        "{r}"
    );
    assert!(open_key(port, "test-ola.vkey").contains("\"open\":true"));
    let r = tree("l3_solar");
    assert!(r.contains("\"carries\":\"l3_activity\""), "{r}");
    let o = field(&r, "opened").to_string();
    let r = post(
        port,
        "/v1/branch/fill",
        &[
            ("group", "l3_solar"),
            ("why", WHY),
            ("opened", &o),
            ("block", "sw_activity_count"),
            ("question", "How many quiet days?"),
        ],
    );
    assert!(
        field(&r, "message")
            .contains("sw_activity_count carries l3_activity, mounted on it in systems's file"),
        "{r}"
    );

    // A file of the copy that cannot be read: the frame says so, and why,
    // never drawn as a frame with nothing in it.
    std::fs::create_dir_all(copy.join("groups/l3_solar/nodes")).unwrap();
    std::fs::write(
        copy.join("groups/l3_solar/nodes/broken.vnode"),
        "not a file",
    )
    .unwrap();
    let r = tree("systems");
    assert!(
        field(&r, "frame_refused").starts_with("a file of your copy cannot be read")
            && r.contains("\"valve_above\":null")
            && r.contains("\"below\":[]"),
        "{r}"
    );
    std::fs::remove_file(copy.join("groups/l3_solar/nodes/broken.vnode")).unwrap();
    let r = tree("systems");
    assert_eq!(field(&r, "frame_refused"), "", "{r}");

    let after: Vec<Vec<u8>> = [SYSTEMS, SOLAR]
        .iter()
        .map(|p| std::fs::read(drv.join(p)).unwrap())
        .collect();
    assert_eq!(after, on_drive, "the drive is never written");
    assert!(!drv.join(NEW).exists());

    // Put back by Sam, the systems' file goes over the drive's, as Sam signed
    // it last, its history kept (F6-e); the group Rui started stays in the
    // copy.
    assert!(open_key(port, "test-sam.vkey").contains("\"open\":true"));
    let r = post(port, "/v1/put-back", &[]);
    assert!(r.contains("the systems' file, by Test Sam, with"), "{r}");
    let files = format!(
        "{} {SYSTEMS}",
        vleo_files::copied::digest(&std::fs::read(copy.join(SYSTEMS)).unwrap())
    );
    let r = post(port, "/v1/put-back", &[("files", &files)]);
    assert!(r.contains("\"written\":true"), "{r}");
    let back = sqlite::read(&drv.join(SYSTEMS)).unwrap();
    assert!(
        back.mounts.iter().any(|m| m.group_id == "l3_radiation")
            && back.mounts.iter().any(|m| m.group_id == "l3_activity"),
        "{:?}",
        back.mounts
    );
    assert!(!drv.join(NEW).exists());
    let _ = std::fs::remove_dir_all(&scratch);
}
