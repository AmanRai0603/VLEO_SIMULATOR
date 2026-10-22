//! A preview of a release that moves another group's values, written by
//! the system engineer through the program, put back, and answered
//! (docs/briefs/F6.md, F6-c items 3, 4 and 10; W7).
//!
//! A drive of invented people: Test Pat (91), programme manager; Test Sam
//! (92), system engineer; Test Ola (93), who owns solar; Test Nia (94),
//! assigned solar's `sw_f107_design_long`; Test Uma (95), who owns the
//! orbit's environment, `l3_x_envorbit`, which reads solar's F10.7. Solar
//! 1.0 is sealed and put back, and design 2026.10.1, holding it, released
//! by Sam with the library and written to the drive as put-back writes it.
//! Then Nia gives the long design F10.7 a line its own cases do not look at,
//! as today's view's test gives it, and solar 1.1 is sealed and put back.
//! The tool opens today's design, which takes solar 1.1. Through the routes:
//!
//! - `GET /v1/design/previews` says solar 1.1 is not the release design
//!   2026.10.1 holds, and has no preview;
//! - a preview with no key open, by Ola, without the launch's word, or of a
//!   group today's design takes no release of, is refused by name, and
//!   nothing is written;
//! - Sam's is written in the copy: the release held whole, every value
//!   outside solar that the engine answers differently on the two designs,
//!   the orbit's environment's F10.7 among them, and the orbit's environment
//!   named to answer; the systems' own value shown and not named. Asked
//!   again, it is never written over, and written again, at its place with
//!   a sequence, only once the one there no longer opens;
//! - put back, Uma's My work says *Answer l3_solar 1.1*; her objection,
//!   written through `POST /v1/answer`, counts only once it is put back: in
//!   her copy, it is unanswered and My work says to put it back; on the
//!   drive, it is an objection with why, and My work says no more of it;
//! - Sam decides her objection, asking for solar's next version: an issue in
//!   his copy to Ola at the objection's place, the listing saying what was
//!   decided, and the issue a red mark on solar, open for Ola, as any issue
//!   to a group is; decided again, by Uma, as no decision, or of no
//!   objection, refused by name; as an allocation changed, it is not written
//!   there, and the route, as the listing does, says what a reallocation's
//!   why says to decide it;
//! - changed on the drive after she signed it, her answer is refused, and My
//!   work says it does not count;
//! - the preview copied under another design's name is listed with why, and
//!   said to Uma in My work.
//!
//! What moved is the engine's, on two graphs it built itself: the test
//! asserts that a value moved and where, never what it moved to.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use vleo_files::chain::{self, Registry, FILE};
use vleo_files::copied::{self, Copied};
use vleo_files::design::{self, Held, Part, Taken, Whole};
use vleo_files::keys::{SigningKey, MIN_ITERATIONS};
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::put_back::{digests, put_back, Chain};
use vleo_files::save::{sign_in_place, Saved};
use vleo_files::sqlite;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-15T12:00:00Z";
const LONG: &str = "sw_f107_design_long";
const NODE: &str = "groups/l3_solar/nodes/sw_f107_design_long.vnode";
const SOLAR: &str = "groups/l3_solar/l3_solar.vgroup";
const PASSPHRASE: &str = "a long enough passphrase";
const DESIGN: &str = "2026.10.1";

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
/// what solar's 1.1 fixture holds of it: its method in the method language,
/// its declaration and its cases, its inputs' defaults, and its units as the
/// method language writes them.
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
            .filter(|t| t.scope == LONG && t.kind == "pseudocode")
            .cloned(),
    );
    n.tables
        .extend(rel.tables.iter().filter(|t| t.scope == LONG).cloned());
    for p in n.ports.iter_mut() {
        let theirs = rel
            .ports
            .iter()
            .find(|q| {
                q.block_uid == LONG
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
    let scratch = std::env::temp_dir().join(format!("vleo-preview-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let (drive, copy) = (scratch.join("drive"), scratch.join("copy"));
    let mut programme = group("programme", "Test Pat");
    for (who, role, k) in [
        ("Test Pat", "programme manager", 91),
        ("Test Sam", "system engineer", 92),
        ("Test Ola", "subsystem engineer", 93),
        ("Test Uma", "subsystem engineer", 95),
    ] {
        register(&mut programme, who, role, &key(k), "Test Pat");
    }
    mount(&mut programme, "root", "systems", "Test Sam");
    let mut systems = group("systems", "Test Sam");
    mount(
        &mut systems,
        "sys_space_environment",
        "l3_solar",
        "Test Ola",
    );
    mount(&mut systems, "sys_orbit", "l3_x_envorbit", "Test Uma");
    let mut solar = design(SOLAR);
    solar.meta.insert(s("writer"), s("Test Ola"));
    register(
        &mut solar,
        "Test Nia",
        "node engineer",
        &key(94),
        "Test Ola",
    );
    solar.tables.push(Tbl {
        scope: s(FILE),
        path: s("versions.csv"),
        csv: s("version,date,by,believed,tested,learned,changed,risks,cost,rests_on,breaks_if\n\
             1.0,2026-10-15,Test Ola,,,,,,,the NOAA SWPC monthly means,a correction to those means\n\
             1.1,2026-10-16,Test Ola,The long design F10.7 held.,Read against its cases.,Below a centre of 95 sfu it reads another table.,One line of its method.,,,the NOAA SWPC monthly means,a correction to those means\n"),
    });
    solar.assignments.push(Assignment {
        block_uid: s(LONG),
        person: s("Test Nia"),
        contract_version: 1,
    });
    let mut node = brought_up(design(NODE));
    node.meta.insert(s("writer"), s("Test Nia"));
    for d in [
        "groups/programme",
        "groups/systems",
        "groups/l3_solar/nodes",
        "integration",
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
    take_copy(&drive, &copy);
    (drive, copy)
}

fn take_copy(drive: &Path, copy: &Path) {
    let files = digests(drive).unwrap();
    for path in files.keys() {
        let to = copy.join(path);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(drive.join(path), &to).unwrap();
    }
    let r = Copied {
        drive: std::fs::canonicalize(drive).unwrap().display().to_string(),
        copied_at: s("2026-10-15T09:00:00Z"),
        files,
    };
    std::fs::write(copy.join(copied::FILE_NAME), r.to_toml()).unwrap();
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

/// Nia signs the long design F10.7 as the copy holds it, changed by
/// `change`, as revision `revision`; Ola seals solar `version` and puts it
/// back on the drive. The release's bytes.
fn sealed_and_put_back(
    drive: &Path,
    copy: &Path,
    version: &str,
    revision: i64,
    change: impl Fn(&mut File),
) -> Vec<u8> {
    let mut n = sqlite::read(&copy.join(NODE)).unwrap();
    n.meta.insert(s("revision"), revision.to_string());
    n.blocks[0].revision = revision;
    change(&mut n);
    n.signatures.clear();
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
        version,
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
    let mut named = Vec::new();
    for p in [rel.clone(), s(NODE)] {
        named.push((
            p.clone(),
            copied::digest(&std::fs::read(copy.join(&p)).unwrap()),
        ));
    }
    let r = put_back(
        copy,
        drive,
        "Test Ola",
        "2026-10-16T09:00:00Z",
        &open,
        Some(&named[..1]),
    )
    .unwrap();
    assert!(r.written, "{:?}", r.lines);
    std::fs::read(drive.join(&rel)).unwrap()
}

/// Design 2026.10.1, holding solar 1.0 as the drive holds it, released by
/// Sam with the library on its integration checks, and written to the
/// drive's `design/` as put-back writes it.
fn released(drive: &Path, release: &[u8]) {
    let o = opened(drive);
    let rel = "groups/l3_solar/releases/l3_solar-1.0.vleo";
    let file = sqlite::read(&drive.join(rel)).unwrap();
    let i = design::integration(
        &[Part {
            group: "l3_solar",
            taken: Ok(Taken {
                version: "1.0",
                chain: "sealed by Test Ola, its owner as the systems' file mounts it",
                file: &file,
            }),
        }],
        "",
    );
    assert!(i.holds(), "{:?}", i.failing());
    let pb = std::fs::read(drive.join("groups/programme/programme.vgroup")).unwrap();
    let sb = std::fs::read(drive.join("groups/systems/systems.vgroup")).unwrap();
    let held = Held {
        releases: vec![(
            "l3_solar",
            "1.0",
            file.meta.get("fingerprint").map_or("", String::as_str),
            Whole {
                path: rel,
                bytes: release,
            },
        )],
        programme: Whole {
            path: "groups/programme/programme.vgroup",
            bytes: &pb,
        },
        systems: Whole {
            path: "groups/systems/systems.vgroup",
            bytes: &sb,
        },
    };
    let asked = design::Asked {
        version: DESIGN,
        previous: None,
        changed: "Solar's long design F10.7.",
        marks: &[],
        understood: &[],
        margins: &[],
        previews: &[],
        unread: &[],
        decided: &[],
    };
    let d = design::release(
        &i,
        &held,
        &asked,
        "Test Sam",
        &key(92),
        AT,
        APP,
        &o.programme,
    )
    .unwrap();
    std::fs::create_dir_all(drive.join("design")).unwrap();
    sqlite::write(&d, &drive.join(format!("design/design-{DESIGN}.vleo"))).unwrap();
}

/// The long design F10.7, its method answering from another table below a
/// centre of 95 sfu, where none of its own cases is.
fn long_changed(n: &mut File) {
    for t in n
        .texts
        .iter_mut()
        .filter(|t| t.kind == "pseudocode" || t.kind == "method")
    {
        assert!(t.body.contains("return sustained"));
        t.body = t.body.replace(
            "return sustained",
            "if central < 95 then\n  return central + spread * 14\nend\nreturn sustained",
        );
    }
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

/// The entries of the JSON array `key` in `json`, each as its text.
fn entries<'a>(json: &'a str, key: &str) -> Vec<&'a str> {
    let at = json
        .find(&format!("\"{key}\":["))
        .unwrap_or_else(|| panic!("no {key} in {json}"))
        + key.len()
        + 4;
    let (mut depth, mut start, mut out) = (0, at, Vec::new());
    for (i, c) in json[at..].char_indices() {
        match c {
            '{' => {
                if depth == 0 {
                    start = at + i;
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    out.push(&json[start..at + i + 1]);
                }
            }
            ']' if depth == 0 => break,
            _ => {}
        }
    }
    out
}

#[test]
fn the_system_engineer_previews_a_release_and_its_named_group_answers_it() {
    let (drive, copy) = world("routes");
    let first = sealed_and_put_back(&drive, &copy, "1.0", 2, |_| {});
    released(&drive, &first);
    sealed_and_put_back(&drive, &copy, "1.1", 3, long_changed);
    let scratch = drive.parent().unwrap().to_path_buf();
    let keys = scratch.join("keys");
    std::fs::create_dir_all(&keys).unwrap();
    for (file, who, k) in [
        ("test-sam.vkey", "Test Sam", 92),
        ("test-ola.vkey", "Test Ola", 93),
        ("test-uma.vkey", "Test Uma", 95),
    ] {
        write_key(&keys, file, who, &key(k));
    }
    // The reference data, verified, so the design runs whole.
    let mut store = vleo_data::Store::open(&scratch.join("data"));
    store
        .sync(&vleo_data::Source::Shipped(root().join("bundles")))
        .expect("the shipped bundles verify");
    std::env::set_var("VLEO_KEYS", &keys);
    std::env::set_var("VLEO_COPY", &copy);
    std::env::set_var("VLEO_DRIVE", &drive);
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    std::env::remove_var("VLEO_DESIGN");
    let port = vleo_server::serve(Some(root()), 18983, false, true).expect("no server");
    let previews = || ask(port, "GET", "/v1/design/previews", &[], false);
    let preview = |g: &str| post(port, "/v1/design/preview", &[("group", g)]);
    assert!(open_key(port, "test-ola.vkey").contains("\"open\":true"));
    post(port, "/v1/key/close", &[]);

    // Solar 1.1 is not the release design 2026.10.1 holds, and has no
    // preview yet.
    let p = previews();
    assert!(p.contains(&format!("\"previews_on\":\"{DESIGN}\"")), "{p}");
    assert_eq!(
        entries(&p, "pending"),
        ["{\"group\":\"l3_solar\",\"version\":\"1.1\",\"held\":\"1.0\",\"preview\":\"\"}"],
        "{p}"
    );
    assert!(entries(&p, "previews").is_empty(), "{p}");

    // Refused, by name, with nothing written.
    let r = preview("l3_solar");
    assert!(field(&r, "message").starts_with("no key is open"), "{r}");
    assert!(open_key(port, "test-ola.vkey").contains("\"open\":true"));
    let r = preview("l3_solar");
    assert!(
        field(&r, "message").contains("Test Ola may not write a preview"),
        "{r}"
    );
    assert!(open_key(port, "test-sam.vkey").contains("\"open\":true"));
    let r = ask(
        port,
        "POST",
        "/v1/design/preview",
        &[("group", "l3_solar")],
        false,
    );
    assert!(r.contains("\"ok\":false"), "{r}");
    let r = preview("l3_x_envorbit");
    assert!(
        field(&r, "message").contains("today's design takes no release of l3_x_envorbit"),
        "{r}"
    );
    assert!(!copy.join("integration/l3_solar-1.1").exists());

    // Sam's, in the copy.
    let r = preview("l3_solar");
    assert!(r.contains("\"ok\":true"), "{r}");
    let path = chain::preview_path("l3_solar", "1.1", DESIGN);
    assert_eq!(field(&r, "path"), path);
    assert!(r.contains("\"named\":[\"l3_x_envorbit\"]"), "{r}");
    let moved = entries(&r, "moved");
    for (value, group) in [
        ("env_f107", "l3_x_envorbit"),
        ("sys_space_environment_f10_7", "sys_space_environment"),
    ] {
        let m = moved
            .iter()
            .find(|m| m.contains(&format!("\"value\":\"{value}\"")))
            .unwrap_or_else(|| panic!("{value} moves: {r}"));
        assert!(m.contains(&format!("\"group\":\"{group}\"")), "{m}");
    }
    assert!(
        moved.iter().all(|m| !m.contains("\"group\":\"l3_solar\"")),
        "{r}"
    );
    // How each design ran is said beside what moved.
    assert!(
        r.contains("\"released\":{\"ran\":") && r.contains("\"with\":{\"ran\":"),
        "{r}"
    );
    assert!(
        field(&r, "message").contains("names l3_x_envorbit to answer"),
        "{r}"
    );
    let f = sqlite::read(&copy.join(&path)).unwrap();
    let o = opened(&copy);
    let back = design::open_preview(&f, &[&o.programme, &o.systems], &o.programme, APP)
        .expect("it opens through the chain");
    assert_eq!(back.named, ["l3_x_envorbit"]);
    assert_eq!(
        back.release.bytes,
        std::fs::read(drive.join("groups/l3_solar/releases/l3_solar-1.1.vleo")).unwrap()
    );
    assert!(
        !drive.join(&path).exists(),
        "the drive holds none of it yet"
    );
    let r = preview("l3_solar");
    assert!(
        field(&r, "message").contains(&format!("your copy holds {path} already")),
        "{r}"
    );

    // Listed, in the copy, unanswered; the integration checks list it too.
    let p = previews();
    assert!(p.contains(&format!("\"preview\":\"{path}\"")), "{p}");
    let listed = entries(&p, "previews");
    assert_eq!(listed.len(), 1, "{p}");
    assert!(
        listed[0].contains("\"in\":\"your copy, not put back yet\"")
            && listed[0].contains("\"stale\":false")
            && listed[0].contains(
                "\"answers\":[{\"group\":\"l3_x_envorbit\",\"said\":\"unanswered\",\"in_copy\":false}]"
            ),
        "{p}"
    );
    let i = ask(port, "GET", "/v1/design/integration", &[], false);
    assert!(i.contains(&format!("\"path\":\"{path}\"")), "{i}");

    // Put back by Sam.
    let files = format!(
        "{} {path}",
        copied::digest(&std::fs::read(copy.join(&path)).unwrap())
    );
    let r = post(port, "/v1/put-back", &[("files", &files)]);
    assert!(r.contains("\"written\":true"), "{r}");
    assert!(drive.join(&path).is_file());
    assert!(entries(&previews(), "previews")[0].contains("\"in\":\"the drive\""));

    // Uma's My work asks her to answer it; her objection is read back.
    assert!(open_key(port, "test-uma.vkey").contains("\"open\":true"));
    let w = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        w.contains(&format!(
            "\"kind\":\"answer\",\"group\":\"l3_x_envorbit\",\"node\":\"\",\"says\":\"Answer l3_solar 1.1: its preview on design {DESIGN} names l3_x_envorbit"
        )),
        "{w}"
    );
    let because = "Below 95 sfu the orbit's density is read from another table.";
    let r = post(
        port,
        "/v1/answer",
        &[
            ("group", "l3_solar"),
            ("version", "1.1"),
            ("answering", "l3_x_envorbit"),
            ("said", "object"),
            ("because", because),
        ],
    );
    assert!(r.contains("\"ok\":true"), "{r}");
    // In her copy only, it is not counted: unanswered, said to be in the
    // copy, and My work asks her to put it back.
    let p = previews();
    assert!(
        entries(&p, "previews")[0].contains(
            "\"answers\":[{\"group\":\"l3_x_envorbit\",\"said\":\"unanswered\",\"in_copy\":true}]"
        ),
        "{p}"
    );
    let w = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        w.contains("\"says\":\"Put back l3_x_envorbit's answer to l3_solar 1.1"),
        "{w}"
    );
    // Put back, it is her objection, with why, and My work says no more.
    let answer = vleo_files::answer::path_of("l3_solar", "1.1", "l3_x_envorbit");
    let files = format!(
        "{} {answer}",
        copied::digest(&std::fs::read(copy.join(&answer)).unwrap())
    );
    let r = post(port, "/v1/put-back", &[("files", &files)]);
    assert!(r.contains("\"written\":true"), "{r}");
    let p = previews();
    // What a reallocation's why says to decide it: the place, and the first
    // 12 characters of the answer's digest on the drive.
    let digest = chain::digest(&sqlite::read(&drive.join(&answer)).unwrap(), FILE).unwrap();
    assert!(
        entries(&p, "previews")[0].contains(&format!(
            "\"answers\":[{{\"group\":\"l3_x_envorbit\",\"said\":\"object\",\"because\":\"{because}\",\"objection\":\"objection:l3_solar-1.1.l3_x_envorbit\",\"decision\":\"undecided\",\"reallocation_names\":\"objection:l3_solar-1.1.l3_x_envorbit answer {}\",\"in_copy\":false}}]",
            &digest[..12]
        )),
        "{p}"
    );
    let w = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(!w.contains("\"kind\":\"answer\""), "{w}");
    // Her objection decided by Sam, through the program (F6-c3b).
    let decide = |decision: &str, answering: &str| {
        post(
            port,
            "/v1/design/decide",
            &[
                ("group", "l3_solar"),
                ("version", "1.1"),
                ("answering", answering),
                ("decision", decision),
            ],
        )
    };
    let r = decide("next", "l3_x_envorbit");
    assert!(
        field(&r, "message").contains("Test Uma may not decide an objection"),
        "{r}"
    );
    assert!(open_key(port, "test-sam.vkey").contains("\"open\":true"));
    let objection = "objection:l3_solar-1.1.l3_x_envorbit";
    for (d, a, why) in [
        (
            "allocation",
            "l3_x_envorbit",
            "an allocation changed decides objection:l3_solar-1.1.l3_x_envorbit from the drive, and nothing is written here: Reallocate in the systems' branch, its why saying \\\"objection:l3_solar-1.1.l3_x_envorbit answer ",
        ),
        (
            "maybe",
            "l3_x_envorbit",
            "is not a decision: next (ask for its next version)",
        ),
        (
            "next",
            "l3_solar",
            "no answer on the drive objects as objection:l3_solar-1.1.l3_solar",
        ),
    ] {
        let r = decide(d, a);
        assert!(r.contains("\"ok\":false") && r.contains(why), "{d}: {r}");
    }
    // An allocation changed: a reallocate row of the drive's systems' file,
    // its rows signed with it, its why saying what the listing named, decides
    // it with nothing in the copy. The file as it was, it is undecided again.
    let at = drive.join("groups/systems/systems.vgroup");
    let was = std::fs::read(&at).unwrap();
    let mut sy = sqlite::read(&at).unwrap();
    sy.changes.push(Change {
        at: s("2026-10-20T09:00:00Z"),
        who: s("Test Sam"),
        scope: s("sys_orbit"),
        what: format!(
            "reallocate: the orbit's range widened, deciding {objection} answer {}",
            &digest[..12]
        ),
        before: None,
        after: None,
    });
    chain::sign_rows(&mut sy);
    sy.signatures.retain(|r| r.scope != FILE);
    sqlite::write(&signed(sy, "Test Sam", &key(92)), &at).unwrap();
    let reallocated =
        "an allocation changed, by Test Sam at 2026-10-20T09:00:00Z, in the systems' file";
    let p = previews();
    assert!(
        entries(&p, "previews")[0].contains(&format!(
            "\"objection\":\"{objection}\",\"decision\":\"{reallocated}\""
        )),
        "{p}"
    );
    let r = decide("next", "l3_x_envorbit");
    assert!(
        field(&r, "message").contains(&format!("{objection} is decided already: {reallocated}")),
        "{r}"
    );
    std::fs::write(&at, was).unwrap();
    let r = decide("next", "l3_x_envorbit");
    assert!(r.contains("\"ok\":true"), "{r}");
    assert_eq!(field(&r, "objection"), objection);
    assert_eq!(field(&r, "addressed_to"), "Test Ola");
    let issue = field(&r, "file").to_string();
    let f = sqlite::read(&copy.join(&issue)).unwrap();
    assert_eq!(f.issues[0].place, objection);
    assert_eq!(f.issues[0].group_id, "l3_solar");
    let p = previews();
    assert!(
        entries(&p, "previews")[0].contains(&format!(
            "\"objection\":\"{objection}\",\"decision\":\"its next version, asked for by Test Sam in issue "
        )),
        "{p}"
    );
    let r = decide("anyway", "l3_x_envorbit");
    assert!(
        field(&r, "message").contains(&format!("{objection} is decided already: its next version")),
        "{r}"
    );
    // Its next version asked for is work for solar's owner: a red mark on
    // solar, as any open issue to a group is, never hidden as the
    // objection's.
    let i = ask(port, "GET", "/v1/design/integration", &[], false);
    assert!(i.contains("\"ok\":true"), "{i}");
    let mark = format!(
        "issue:{}",
        issue
            .trim_start_matches("issues/")
            .trim_end_matches(".vissue")
    );
    assert!(
        i.contains(&format!("{{\"id\":\"{mark}\",\"said\":\"issue 1 on l3_solar, open: l3_x_envorbit objects to l3_solar 1.1:")),
        "{mark}: {i}"
    );
    // Released anyway in its place: the programme manager's issue raises the
    // objection's mark, and is no red mark of its own.
    std::fs::remove_file(copy.join(&issue)).unwrap();
    let r = decide("anyway", "l3_x_envorbit");
    assert!(r.contains("\"ok\":true"), "{r}");
    let anyway = format!(
        "issue:{}",
        field(&r, "file")
            .trim_start_matches("issues/")
            .trim_end_matches(".vissue")
    );
    assert!(anyway.ends_with("-programme"), "{r}");
    let i = ask(port, "GET", "/v1/design/integration", &[], false);
    assert!(i.contains("\"ok\":true"), "{i}");
    assert!(!i.contains(&format!("{{\"id\":\"{anyway}\"")), "{i}");
    // Beside a preview that may be one and does not open, no objection
    // stands: the issue is said as it is.
    let odd = drive.join("integration/l3_solar-1.1/preview-of-something.vleo");
    std::fs::write(&odd, b"not a file").unwrap();
    let i = ask(port, "GET", "/v1/design/integration", &[], false);
    assert!(i.contains(&format!("{{\"id\":\"{anyway}\"")), "{i}");
    std::fs::remove_file(&odd).unwrap();
    assert!(open_key(port, "test-uma.vkey").contains("\"open\":true"));
    // Changed on the drive after she signed it: refused, never her answer,
    // and My work says it does not count.
    let mut f = sqlite::read(&drive.join(&answer)).unwrap();
    for t in f.texts.iter_mut().filter(|t| t.kind == "because") {
        t.body = s("Nothing moves that matters.");
    }
    sqlite::write(&f, &drive.join(&answer)).unwrap();
    let p = previews();
    let listed = entries(&p, "previews")[0];
    assert!(
        listed.contains("\"said\":\"refused\"") && listed.contains("has no signature that counts"),
        "{p}"
    );
    let w = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        w.contains("\"says\":\"l3_x_envorbit's answer to l3_solar 1.1 does not count"),
        "{w}"
    );
    // The preview under a name that is not its own: listed with why, and
    // said to Uma, whom it says it names.
    let wrong = "integration/l3_solar-1.1/preview-l3_solar-1.1-on-2026.10.9.vleo";
    std::fs::copy(drive.join(&path), drive.join(wrong)).unwrap();
    let p = previews();
    let planted = entries(&p, "previews")
        .into_iter()
        .find(|e| e.contains(wrong))
        .unwrap_or_else(|| panic!("{p}"));
    assert!(
        planted.contains("\"opens\":false") && planted.contains(&format!("so it is {path}")),
        "{planted}"
    );
    let w = ask(port, "GET", "/v1/my-work", &[], false);
    assert!(
        w.contains("\"kind\":\"preview refused\"") && w.contains(wrong),
        "{w}"
    );
    // Written again only once the one at its place no longer opens, as when
    // its writer stops being the system engineer: then at the next place,
    // with a sequence (F6's answer (a)).
    assert!(open_key(port, "test-sam.vkey").contains("\"open\":true"));
    let r = preview("l3_solar");
    assert!(
        field(&r, "message").contains(&format!("holds {path} already, and it opens")),
        "{r}"
    );
    for at in [copy.join(&path), drive.join(&path)] {
        std::fs::write(at, "no longer a preview").unwrap();
    }
    let r = preview("l3_solar");
    assert!(r.contains("\"ok\":true"), "{r}");
    let again = chain::preview_path_n("l3_solar", "1.1", DESIGN, 2);
    assert!(again.ends_with("-on-2026.10.1-2.vleo"), "{again}");
    assert_eq!(field(&r, "path"), again);
    let p = previews();
    assert!(
        entries(&p, "previews")
            .iter()
            .any(|e| e.contains(&again) && e.contains("\"opens\":true")),
        "{p}"
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
