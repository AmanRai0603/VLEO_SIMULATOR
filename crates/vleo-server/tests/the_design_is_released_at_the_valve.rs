//! The design released by the system engineer through the program, and
//! put back on the drive (docs/briefs/F6.md, F6-b; W8; `vleo_files::design`).
//!
//! The drive of today's design's own test: Test Pat (91), programme manager;
//! Test Sam (92), system engineer; Test Ola (93), who owns solar; Test Nia
//! (94), assigned solar's `sw_activity_band`. Solar 1.0 is sealed by Ola and
//! put back, and the tool opens today's design from the drive. Through
//! `GET /v1/design/integration`, `POST /v1/design/release` and
//! `POST /v1/put-back`:
//!
//! - the checks hold for solar, a line for each of W8's five, with no design
//!   released yet;
//! - a release with no key open, by Ola, without the launch's word, of a
//!   version that is not one, or with a red mark not understood (an issue
//!   open in the copy), is refused by name, and nothing is written;
//! - Sam releases it in the copy: the released design, its notes and
//!   `readable/`, and the drive holds none of it until it is put back;
//! - released again under the same version in the copy, it is never written
//!   over;
//! - put back, the drive holds it, and the checks then say solar's base no
//!   longer fits the design released now.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use vleo_files::chain::{self, Registry, FILE};
use vleo_files::copied::{self, Copied};
use vleo_files::keys::{SigningKey, MIN_ITERATIONS};
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::put_back::{digests, put_back, Chain};
use vleo_files::save::{sign_in_place, Saved};
use vleo_files::sqlite;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-15T12:00:00Z";
const UID: &str = "sw_activity_band";
const NODE: &str = "groups/l3_solar/nodes/sw_activity_band.vnode";
const SOLAR: &str = "groups/l3_solar/l3_solar.vgroup";
const SAID: &str = " Said again by its node engineer for solar 1.0.";
const PASSPHRASE: &str = "a long enough passphrase";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

fn design(path: &str) -> File {
    sqlite::read(&root().join("design").join(path)).unwrap()
}

/// The design's node as its node engineer brings it to 1.0's checks, from
/// what solar's 1.1 release holds of it: its method in the method language,
/// its declaration and its cases, and its input's default, and its units as the method language writes them.
fn brought_up(mut n: File) -> File {
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
    n.texts.extend(
        rel.texts
            .iter()
            .filter(|t| t.scope == UID && t.kind == "pseudocode")
            .cloned(),
    );
    n.tables
        .extend(rel.tables.iter().filter(|t| t.scope == UID).cloned());
    for p in n.ports.iter_mut() {
        let theirs = rel
            .ports
            .iter()
            .find(|q| {
                q.block_uid == UID
                    && q.direction == p.direction
                    && (q.name == p.name || p.direction == "out")
            })
            .unwrap();
        p.unit = theirs.unit.clone();
        if p.direction == "in" {
            p.value = theirs.value.clone();
            p.lower = theirs.lower.clone();
            p.upper = theirs.upper.clone();
        }
    }
    n
}

/// The drive, and a copy taken of it, under a scratch folder of `name`.
fn world(name: &str) -> (PathBuf, PathBuf) {
    let scratch =
        std::env::temp_dir().join(format!("vleo-today-sealed-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let (drive, copy) = (scratch.join("drive"), scratch.join("copy"));
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
    mount(&mut programme, "root", "systems", "Test Sam");
    let mut systems = group("systems", "Test Sam");
    mount(
        &mut systems,
        "sys_space_environment",
        "l3_solar",
        "Test Ola",
    );
    // Solar's group file and the node's, as the design holds them, with the
    // group's node engineer and her assignment.
    let mut solar = design(SOLAR);
    solar.meta.insert(s("writer"), s("Test Ola"));
    register(
        &mut solar,
        "Test Nia",
        "node engineer",
        &key(94),
        "Test Ola",
    );
    // Ola's record of which belief broke: solar's first release.
    solar.tables.push(Tbl {
        scope: s(FILE),
        path: s("versions.csv"),
        csv: s("version,date,by,believed,tested,learned,changed,risks,cost,rests_on,breaks_if\n\
             1.0,2026-10-15,Test Ola,,,,,,,the four NOAA SWPC activity levels,a correction to those levels\n"),
    });
    solar.assignments.push(Assignment {
        block_uid: s(UID),
        person: s("Test Nia"),
        contract_version: 1,
    });
    let mut node = brought_up(design(NODE));
    node.meta.insert(s("writer"), s("Test Nia"));
    for d in [
        "groups/programme",
        "groups/systems",
        "groups/l3_solar/nodes",
    ] {
        std::fs::create_dir_all(drive.join(d)).unwrap();
    }
    for (f, who, k, at) in [
        (
            programme,
            "Test Pat",
            key(91),
            "groups/programme/programme.vgroup",
        ),
        (
            systems,
            "Test Sam",
            key(92),
            "groups/systems/systems.vgroup",
        ),
        (solar, "Test Ola", key(93), SOLAR),
    ] {
        sqlite::write(&signed(f, who, &k), &drive.join(at)).unwrap();
    }
    sqlite::write(&node, &drive.join(NODE)).unwrap();
    std::fs::write(
        drive.join("START HERE.md"),
        format!("# START HERE\n\n    {}\n", key(91).public().fingerprint()),
    )
    .unwrap();
    let files = digests(&drive).unwrap();
    for path in files.keys() {
        let to = copy.join(path);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(drive.join(path), &to).unwrap();
    }
    let r = Copied {
        drive: std::fs::canonicalize(&drive).unwrap().display().to_string(),
        copied_at: s("2026-10-15T09:00:00Z"),
        files,
    };
    std::fs::write(copy.join(copied::FILE_NAME), r.to_toml()).unwrap();
    (drive, copy)
}

struct Opened {
    anchor: String,
    programme: Registry,
    systems: Registry,
    solar: Registry,
}

fn opened(at: &Path) -> Opened {
    let read = |p: &str| sqlite::read(&at.join(p)).unwrap();
    let anchor = key(91).public().fingerprint();
    let programme =
        chain::open_programme(read("groups/programme/programme.vgroup"), &anchor).unwrap();
    let systems = chain::open_systems(&read("groups/systems/systems.vgroup"), &programme).unwrap();
    let solar = chain::open_group_in(&read(SOLAR), &systems, &programme).unwrap();
    Opened {
        anchor,
        programme,
        systems,
        solar,
    }
}

/// Nia says the node again in the copy, as its second revision, and signs
/// it; Ola seals solar 1.0 there and puts it back on the drive.
fn sealed_and_put_back(drive: &Path, copy: &Path) -> String {
    let mut n = sqlite::read(&copy.join(NODE)).unwrap();
    n.meta.insert(s("revision"), s("2"));
    n.blocks[0].revision = 2;
    let t = n
        .texts
        .iter_mut()
        .find(|t| t.scope == UID && t.kind == "explain.simply")
        .unwrap();
    t.body.push_str(SAID);
    sqlite::write(&n, &copy.join(NODE)).unwrap();
    let seen = copied::digest(&std::fs::read(copy.join(NODE)).unwrap());
    let o = opened(copy);
    match sign_in_place(copy, NODE, &seen, "Test Nia", &key(94), AT, APP, &o.solar).unwrap() {
        Saved::Written { .. } => {}
        other => panic!("Nia did not sign: {other:?}"),
    }
    let rel = vleo_files::release::seal_in_place(
        copy,
        "l3_solar",
        "1.0",
        "Test Ola",
        &key(93),
        AT,
        APP,
        &o.systems,
        &o.programme,
    )
    .unwrap();
    let open = || {
        let o = opened(drive);
        Ok(Chain {
            anchor: o.anchor,
            programme: o.programme,
            systems: Some(o.systems),
            groups: vec![o.solar],
            left_out: Vec::new(),
        })
    };
    let digest = copied::digest(&std::fs::read(copy.join(&rel)).unwrap());
    let r = put_back(
        copy,
        drive,
        "Test Ola",
        "2026-10-16T09:00:00Z",
        &open,
        Some(&[(rel.clone(), digest)]),
    )
    .unwrap();
    assert!(r.written, "{:?}", r.lines);
    rel
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

fn write_key(dir: &Path, file: &str, person: &str, k: &SigningKey) {
    let locked = k
        .lock(person, PASSPHRASE, [7; 16], [7; 12], MIN_ITERATIONS)
        .unwrap();
    let mut f = File::new(Kind::Key, APP);
    f.meta.insert(s("person"), s(person));
    f.locked_keys.push(locked.to_row());
    sqlite::write(&f, &dir.join(file)).unwrap();
}

fn open_key(port: u16, file: &str) -> String {
    post(
        port,
        "/v1/key/open",
        &[
            ("file", file),
            ("passphrase", PASSPHRASE),
            ("accept_anchor", &key(91).public().fingerprint()),
        ],
    )
}

/// Every red mark's id in an integration's answer.
fn mark_ids(json: &str) -> Vec<String> {
    let marks = &json[json.find("\"marks\":[").expect("no marks")..];
    marks
        .match_indices("{\"id\":\"")
        .map(|(at, k)| {
            let rest = &marks[at + k.len()..];
            rest[..rest.find('"').unwrap()].to_string()
        })
        .collect()
}

#[test]
fn the_system_engineer_releases_the_design_and_it_goes_back_to_the_drive() {
    let (drive, copy) = world("design-released");
    let rel = sealed_and_put_back(&drive, &copy);
    // A newer release of solar on the drive whose seal does not check:
    // today's design refuses it and takes 1.0, its last good one.
    let mut newer = sqlite::read(&drive.join(&rel)).unwrap();
    newer.meta.insert(s("version"), s("1.1"));
    newer.meta.insert(s("previous"), s("1.0"));
    sqlite::write(
        &newer,
        &drive.join("groups/l3_solar/releases/l3_solar-1.1.vleo"),
    )
    .unwrap();
    let scratch = drive.parent().unwrap().to_path_buf();
    let keys = scratch.join("keys");
    std::fs::create_dir_all(&keys).unwrap();
    write_key(&keys, "test-sam.vkey", "Test Sam", &key(92));
    write_key(&keys, "test-ola.vkey", "Test Ola", &key(93));
    std::env::set_var("VLEO_KEYS", &keys);
    std::env::set_var("VLEO_COPY", &copy);
    std::env::set_var("VLEO_DRIVE", &drive);
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    std::env::remove_var("VLEO_DESIGN");
    let port = vleo_server::serve(Some(root()), 18981, false, true).expect("no server");
    let checks = || ask(port, "GET", "/v1/design/integration", &[], false);
    let on_drive = digests(&drive).unwrap();
    // An issue open in the copy: a red mark, to be marked understood.
    let mut issue = File::new(Kind::Issue, APP);
    for (k, v) in [
        ("number", "1"),
        ("group_id", "l3_solar"),
        ("writer", "Test Nia"),
    ] {
        issue.meta.insert(s(k), s(v));
    }
    issue.issues.push(Issue {
        number: 1,
        group_id: s("l3_solar"),
        raised_by: s("Test Nia"),
        at: s(AT),
        place: s(UID),
        what: s("The band's top end is read from a monthly mean."),
        evidence: s(""),
        addressed_to: s("Test Ola"),
        closed_by: s(""),
    });
    std::fs::create_dir_all(copy.join("issues")).unwrap();
    sqlite::write(&issue, &copy.join("issues/1-l3_solar.vissue")).unwrap();

    // Before this computer accepts the drive's anchor, nothing is checked.
    let i = checks();
    assert!(
        i.contains("this computer has not accepted this database's anchor"),
        "{i}"
    );
    // Ola opens their key, and the anchor with it; then closes it.
    assert!(open_key(port, "test-ola.vkey").contains("\"open\":true"));
    post(port, "/v1/key/close", &[]);

    // The checks hold for solar, a line for each of W8's five.
    let i = checks();
    assert!(
        i.contains("\"ok\":true") && i.contains("\"holds\":true"),
        "{i}"
    );
    assert_eq!(i.matches("\"group\":\"l3_solar\"").count(), 5, "{i}");
    assert!(i.contains("\"released\":\"\""), "{i}");
    assert!(
        i.contains("l3_solar 1.0 is taken: sealed by Test Ola"),
        "{i}"
    );
    let version = field(&i, "next_version").to_string();
    let understood = mark_ids(&i).join(",");
    let release = |pairs: &[(&str, &str)]| post(port, "/v1/design/release", pairs);
    let asked = |v: &str, u: &str| {
        release(&[
            ("version", v),
            ("changed", "Solar's band, said again."),
            ("understood", u),
        ])
    };

    // Refused, by name, with nothing written.
    let r = asked(&version, &understood);
    assert!(field(&r, "message").starts_with("no key is open"), "{r}");
    assert!(open_key(port, "test-ola.vkey").contains("\"open\":true"));
    let r = asked(&version, &understood);
    assert!(
        field(&r, "message").contains("Test Ola may not release the design"),
        "{r}"
    );
    assert!(open_key(port, "test-sam.vkey").contains("\"open\":true"));
    let r = ask(
        port,
        "POST",
        "/v1/design/release",
        &[
            ("version", &version),
            ("changed", "Solar."),
            ("understood", &understood),
        ],
        false,
    );
    assert!(r.contains("\"ok\":false"), "{r}");
    let r = asked("2026.13.1", &understood);
    assert!(r.contains("is not a design's version"), "{r}");
    let r = asked(&version, &format!("{understood},closure:no_such_node"));
    assert!(
        field(&r, "message").contains(
            "closure:no_such_node is marked understood, and today's design has no such red mark"
        ),
        "{r}"
    );
    assert_eq!(understood, "group:l3_solar,issue:1-l3_solar", "{i}");
    assert!(
        i.contains("its latest, 1.1, is refused, so its last good release is held"),
        "{i}"
    );
    let r = asked(&version, "");
    assert!(
        r.contains("every red mark is understood")
            && r.contains(
                "issue 1 on l3_solar, open: The band's top end is read from a monthly mean."
            ),
        "{r}"
    );
    // A copy of another drive than today's design is built from.
    let record = std::fs::read_to_string(copy.join(copied::FILE_NAME)).unwrap();
    let mut other = Copied::from_toml(&record).unwrap();
    other.drive = scratch.join("another drive").display().to_string();
    std::fs::write(copy.join(copied::FILE_NAME), other.to_toml()).unwrap();
    let r = asked(&version, &understood);
    assert!(
        r.contains("release it in a copy of the drive it is built from"),
        "{r}"
    );
    std::fs::write(copy.join(copied::FILE_NAME), record).unwrap();
    assert!(
        !copy.join("design").exists() && !copy.join("readable").exists(),
        "a refused release writes nothing"
    );

    // A case saved on this computer moves the health map; the design
    // released is read on the design as declared, never on it.
    assert!(post(
        port,
        "/v1/inputs",
        &[("set", "sw_ap_central_expectation:40")]
    )
    .contains("\"ok\":true"));
    let margin_of = |q: &str, id: &str| {
        let h = ask(port, "GET", q, &[], false);
        let at = h.find(&format!("\"id\":\"{id}\"")).unwrap();
        let m = &h[at..][h[at..].find("\"margin\":").unwrap() + 9..];
        m[..m.find('}').unwrap()].to_string()
    };
    let (moved, declared) = (1..=5)
        .map(|k| format!("l3_solar_ach_0{k}"))
        .find_map(|id| {
            let (d, saved) = (
                margin_of("/v1/health?inputs=defaults", &id),
                margin_of("/v1/health", &id),
            );
            (d != saved).then_some((id, d))
        })
        .expect("the case saved moves a solar closure's margin");

    // Sam releases it, in the copy.
    let r = asked(&version, &understood);
    assert!(r.contains("\"ok\":true"), "{r}");
    let path = format!("design/design-{version}.vleo");
    assert_eq!(field(&r, "path"), path);
    assert!(
        field(&r, "announce").starts_with(&format!(
            "Design {version} is released: Solar's band, said again"
        )),
        "{r}"
    );
    let d = sqlite::read(&copy.join(&path)).unwrap();
    assert_eq!(d.kind().unwrap(), Kind::Design);
    assert_eq!(
        d.meta.get("released_by").map(String::as_str),
        Some("Test Sam")
    );
    let held = d
        .media
        .iter()
        .find(|m| m.path == "groups/l3_solar/releases/l3_solar-1.0.vleo")
        .expect("solar 1.0 held whole");
    assert_eq!(held.bytes, std::fs::read(drive.join(&held.path)).unwrap());
    let t = d.tables.iter().find(|t| t.path == "red_marks.csv").unwrap();
    assert!(t.csv.contains("issue:1-l3_solar,"), "{}", t.csv);
    assert!(t.csv.contains("understood by Test Sam"), "{}", t.csv);
    let notes = std::fs::read_to_string(copy.join("design/NOTES.md")).unwrap();
    assert!(
        notes.contains(&format!("## {version}\n\nReleased by Test Sam")),
        "{notes}"
    );
    let health = std::fs::read_to_string(copy.join("readable/Health.csv")).unwrap();
    assert!(
        health.contains(&format!("\n{moved},l3_solar,closes,{declared},")),
        "{moved} at {declared}: {health}"
    );
    // Every file of readable/; solar holds no closure.
    for (n, rows) in [
        ("Groups", true),
        ("Nodes", true),
        ("Ports", true),
        ("Wires", true),
        ("Closures", false),
        ("Health", true),
        ("Status", true),
    ] {
        let csv = std::fs::read_to_string(copy.join(format!("readable/{n}.csv"))).unwrap();
        assert_eq!(csv.lines().count() > 1, rows, "{n}: {csv}");
    }
    assert_eq!(
        digests(&drive).unwrap(),
        on_drive,
        "the drive holds none of it yet"
    );
    // Another before it is put back, under its version or the next: it goes
    // back first, so two never follow the same one.
    let next = vleo_files::meta::DesignVersion::parse(&version)
        .map(|v| v.next(v.year, v.month).to_string())
        .unwrap();
    for v in [&version, &next] {
        let r = asked(v, &understood);
        assert!(
            r.contains(&format!(
                "your copy holds {path}, released and not put back on the drive yet"
            )),
            "{r}"
        );
    }

    // Put back: the drive holds it, and solar's base no longer fits.
    let r = post(port, "/v1/put-back", &[]);
    assert!(
        r.contains(&format!(
            "goes back: design {version}, released by Test Sam"
        )),
        "{r}"
    );
    let mut files = Vec::new();
    for p in std::iter::once(path.clone())
        .chain(["design/NOTES.md".to_string()])
        .chain(
            [
                "Groups", "Nodes", "Ports", "Wires", "Closures", "Health", "Status",
            ]
            .map(|n| format!("readable/{n}.csv")),
        )
    {
        files.push(format!(
            "{} {p}",
            copied::digest(&std::fs::read(copy.join(&p)).unwrap())
        ));
    }
    let r = post(port, "/v1/put-back", &[("files", &files.join("\n"))]);
    assert!(r.contains("\"written\":true"), "{r}");
    assert!(drive.join(&path).is_file());
    let i = checks();
    assert!(i.contains("\"holds\":false"), "{i}");
    assert!(i.contains(&format!("\"released\":\"{version}\"")), "{i}");
    assert!(
        i.contains(&format!("and the design released now is {version}")),
        "{i}"
    );
    // So the next is refused whole, by the line that fails, and writes
    // nothing.
    let readable = std::fs::read(copy.join("readable/Groups.csv")).unwrap();
    let r = asked(&next, &understood);
    assert!(
        r.contains("the design is not released, refused whole: l3_solar, the base still fitting"),
        "{r}"
    );
    assert!(!copy.join(format!("design/design-{next}.vleo")).exists());
    assert_eq!(
        std::fs::read(copy.join("readable/Groups.csv")).unwrap(),
        readable
    );

    // A group the systems' file mounts with no release on the drive fails
    // its line; it is never left out.
    let mut sy = sqlite::read(&drive.join("groups/systems/systems.vgroup")).unwrap();
    sy.signatures.retain(|r| r.scope != FILE);
    mount(&mut sy, "sys_radiation", "l3_radiation", "Test Ola");
    sqlite::write(
        &signed(sy, "Test Sam", &key(92)),
        &drive.join("groups/systems/systems.vgroup"),
    )
    .unwrap();
    let i = checks();
    assert!(
        i.contains("l3_radiation is mounted, and the drive holds no release of it"),
        "{i}"
    );
    // Solar's release changed on the drive since the tool opened: read
    // again, its seal does not check, and nothing of it is held.
    let rel = drive.join("groups/l3_solar/releases/l3_solar-1.0.vleo");
    let mut f = sqlite::read(&rel).unwrap();
    f.texts
        .iter_mut()
        .find(|t| t.scope == UID && t.kind == "explain.simply")
        .unwrap()
        .body
        .push_str(" And changed on the drive.");
    sqlite::write(&f, &rel).unwrap();
    let i = checks();
    assert!(
        i.contains("l3_solar 1.0 is not read again from the drive: its seal does not check through the chain"),
        "{i}"
    );

    // W16: the design released now is not released again; an earlier one is.
    let again = |of: &str, v: &str, why: &str| {
        post(
            port,
            "/v1/design/release-again",
            &[("of", of), ("version", v), ("why", why)],
        )
    };
    let r = again(&version, &next, "Solar's band was right before.");
    assert!(
        r.contains(&format!("{version} is the design released now")),
        "{r}"
    );
    // A later design is on the drive: the same files under the next
    // version, signed by Sam, so it is the earlier now.
    let held = sqlite::read(&drive.join(&path)).unwrap();
    let mut later = held.clone();
    later.meta.insert(s("version"), next.clone());
    later.meta.insert(s("previous"), version.clone());
    later.signatures.retain(|r| r.scope != FILE);
    let row = chain::sign(&later, FILE, "Test Sam", &key(92), AT, "ok", "").unwrap();
    later.signatures.push(row);
    sqlite::write(&later, &drive.join(format!("design/design-{next}.vleo"))).unwrap();
    let list = ask(port, "GET", "/v1/design/released", &[], false);
    assert!(list.contains(&format!("\"released\":\"{next}\"")), "{list}");
    assert!(list.contains("\"holds\":[\"l3_solar 1.0\"]"), "{list}");
    let third = vleo_files::meta::DesignVersion::parse(&next)
        .map(|v| v.next(v.year, v.month).to_string())
        .unwrap();
    for (of, v, why, says) in [
        (
            "2026.1.1",
            third.as_str(),
            "x",
            "the drive holds no released design",
        ),
        (
            version.as_str(),
            version.as_str(),
            "x",
            "does not come after",
        ),
        (version.as_str(), third.as_str(), " ", "say why"),
    ] {
        let r = again(of, v, why);
        assert!(r.contains(says), "{of} as {v}: {r}");
    }
    let r = again(&version, &third, "The later one was wrong.");
    assert!(r.contains("\"ok\":true"), "{r}");
    assert!(
        field(&r, "announce").starts_with(&format!(
            "Design {third} is released: design {version} again, because The later one was wrong"
        )),
        "{r}"
    );
    let path3 = format!("design/design-{third}.vleo");
    let d3 = sqlite::read(&copy.join(&path3)).unwrap();
    assert_eq!(d3.meta.get("again_of"), Some(&version));
    assert_eq!(d3.media, held.media, "the same files, byte for byte");
    let notes = std::fs::read_to_string(copy.join("design/NOTES.md")).unwrap();
    assert!(
        notes.starts_with(&format!(
            "# Released designs\n\nWhat changed in each, newest first.\n\n## {third}\n\nDesign {version} released again by Test Sam"
        )),
        "{notes}"
    );
    assert!(std::fs::read_to_string(copy.join("readable/Health.csv"))
        .unwrap()
        .contains("not computed"));
    // Another before it goes back: refused, as a release is.
    let r = again(&version, "2026.12.1", "Again.");
    assert!(
        r.contains("released and not put back on the drive yet"),
        "{r}"
    );
    // Put back: it goes, said as a design released again.
    let r = post(port, "/v1/put-back", &[]);
    assert!(
        r.contains(&format!(
            "goes back: design {third}, design {version} released again by Test Sam"
        )),
        "{r}"
    );
    let mut files = Vec::new();
    for p in std::iter::once(path3.clone())
        .chain(["design/NOTES.md".to_string()])
        .chain(
            [
                "Groups", "Nodes", "Ports", "Wires", "Closures", "Health", "Status",
            ]
            .map(|n| format!("readable/{n}.csv")),
        )
    {
        // Only what differs from the drive goes back.
        let mine = std::fs::read(copy.join(&p)).unwrap();
        if std::fs::read(drive.join(&p)).ok().as_deref() == Some(&mine[..]) {
            continue;
        }
        files.push(format!(
            "{} {p}",
            copied::digest(&std::fs::read(copy.join(&p)).unwrap())
        ));
    }
    let r = post(port, "/v1/put-back", &[("files", &files.join("\n"))]);
    assert!(r.contains("\"written\":true"), "{r}");
    let list = ask(port, "GET", "/v1/design/released", &[], false);
    assert!(
        list.contains(&format!("\"released\":\"{third}\"")),
        "{list}"
    );
    assert!(
        list.contains(&format!("\"version\":\"{third}\",\"previous\":\"{next}\",\"released_by\":\"Test Sam\",\"again_of\":\"{version}\"")),
        "{list}"
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
