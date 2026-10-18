//! The design is released by the system engineer, whole or not at all
//! (docs/briefs/F6.md, F6-b; W8).
//!
//! A small programme of invented people: Pat the programme manager, Sam the
//! system engineer and Sid their deputy, Ola who owns solar and Dee their
//! deputy, Nia a node engineer of solar. Solar 1.2 is sealed by Ola from
//! solar 1.1's `sw_activity_band`, signed by Nia, as the seal's own test has
//! it. Nothing here is computed: the checks read the release, and the
//! expected values are written in the test.
//!
//! - the integration checks of today's design, solar 1.2 taken, hold, a line
//!   for each of W8's five checks;
//! - each check fails by its own line: no release taken, a base that does
//!   not fit, a node whose checks refuse it, a relation an assistant
//!   supplied, a range with one end or its ends the wrong way round;
//! - Sam releases design 2026.10.1: a released design holding solar 1.2 and
//!   the programme's and the systems' files whole, by their bytes and their
//!   SHA-256, signed by Sam, opening through the chain; Sid releases it too;
//! - refused whole when a line fails, naming it; refused for Ola and Pat, for
//!   a version that is not one or does not come after the design released
//!   now, with nothing said of what changed, and under an assistant's name.

use std::path::Path;

use vleo_files::chain::{self, Registry, FILE};
use vleo_files::design::{self, Asked, Check, Held, Integration, Mark, Part, Taken, Whole};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::release;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-17T09:00:00Z";
const BAND: &str = "sw_activity_band";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn register(f: &mut File, name: &str, role: &str, deputy_for: &str, k: &SigningKey) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(deputy_for),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: k.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Pat Morgan"),
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

fn group_file(id: &str) -> File {
    let mut f = File::new(Kind::Group, APP);
    f.meta.insert(s("group_id"), s(id));
    f.meta.insert(s("writer"), s("whoever"));
    f.meta.insert(s("based_on"), s(""));
    f
}

fn signed(mut f: File, who: &str, k: &SigningKey) -> File {
    f.signatures.retain(|r| r.scope != FILE);
    let row = chain::sign(&f, FILE, who, k, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
    f
}

// Pat the programme manager (11), Sam the system engineer (12), Sid their
// deputy (17), Ola who owns solar (13), Dee their deputy (14), Nia a node
// engineer of solar (16).
fn programme() -> Registry {
    let mut f = group_file("programme");
    register(&mut f, "Pat Morgan", "programme manager", "", &key(11));
    register(&mut f, "Sam Engel", "system engineer", "", &key(12));
    register(&mut f, "Sid Side", "system engineer", "Sam Engel", &key(17));
    register(&mut f, "Ola Sun", "subsystem engineer", "", &key(13));
    register(
        &mut f,
        "Dee Puty",
        "subsystem engineer",
        "Ola Sun",
        &key(14),
    );
    mount(&mut f, "root", "systems", "Sam Engel");
    chain::open_programme(
        signed(f, "Pat Morgan", &key(11)),
        &key(11).public().fingerprint(),
    )
    .unwrap()
}

fn systems(pr: &Registry) -> Registry {
    let mut f = group_file("systems");
    mount(&mut f, "sys_space_environment", "l3_solar", "Ola Sun");
    chain::open_systems(&signed(f, "Sam Engel", &key(12)), pr).unwrap()
}

/// Solar's group file, unsigned: its heading, Nia, and the band assigned to
/// her at `contract`.
fn solar_unsigned(contract: i64) -> File {
    let mut f = group_file("l3_solar");
    f.blocks.push(Block {
        uid: s("l3_solar"),
        id: s("l3_solar"),
        behaviour: s("children"),
        question: s("What solar weather does the design face?"),
        ..Block::default()
    });
    f.people.push(Person {
        name: s("Nia Node"),
        role: s("node engineer"),
        deputy_for: s(""),
    });
    f.keys.push(PersonKey {
        person: s("Nia Node"),
        public_key: key(16).public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Ola Sun"),
        revoked_from: s(""),
        proof: vleo_files::people::proof(&key(16), "Nia Node"),
    });
    f.assignments.push(Assignment {
        block_uid: s(BAND),
        person: s("Nia Node"),
        contract_version: contract,
    });
    // The owner's record of which belief broke, a row for each version.
    f.tables.push(Tbl {
        scope: s(FILE),
        path: s("versions.csv"),
        csv: s("version,date,by,believed,tested,learned,changed,risks,cost,rests_on,breaks_if\n\
             1.0,2026-10-01,Ola Sun,,,,,,,the four NOAA SWPC activity levels,a correction to those levels\n\
             1.1,2026-10-05,Ola Sun,The bands held.,Read against their cases.,They held.,Nothing but words.,,,the four NOAA SWPC activity levels,a correction to those levels\n\
             1.2,2026-10-10,Ola Sun,The bands held.,Read again.,They held.,A method said plainly.,,,the four NOAA SWPC activity levels,a correction to those levels\n"),
    });
    f
}

fn solar() -> File {
    signed(solar_unsigned(1), "Ola Sun", &key(13))
}

struct World {
    pr: Registry,
    sy: Registry,
}

fn world() -> World {
    let pr = programme();
    let sy = systems(&pr);
    World { pr, sy }
}

/// `uid` from solar 1.1's release, in a node file of its own, unsigned.
fn from_release(uid: &str) -> File {
    let how = vleo_files::upgrade::Upgrade { app: APP, at: AT };
    let rel = vleo_files::sqlite::open(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/l3_solar-1.1.vleo"),
        &how,
    )
    .unwrap()
    .file;
    let mut n = File::new(Kind::Node, APP);
    for (k, v) in [
        ("group_id", "l3_solar"),
        ("block_uid", uid),
        ("revision", "1"),
        ("contract_version", "1"),
        ("writer", "Nia Node"),
    ] {
        n.meta.insert(s(k), s(v));
    }
    n.blocks = vec![rel.blocks.iter().find(|b| b.uid == uid).unwrap().clone()];
    n.blocks[0].parent_uid = s("l3_solar");
    n.ports = rel
        .ports
        .iter()
        .filter(|p| p.block_uid == uid)
        .cloned()
        .collect();
    n.wires = rel
        .wires
        .iter()
        .filter(|w| w.to_block == uid)
        .cloned()
        .collect();
    n.texts = rel
        .texts
        .iter()
        .filter(|t| t.scope == uid)
        .cloned()
        .collect();
    n.tables = rel
        .tables
        .iter()
        .filter(|t| t.scope == uid)
        .cloned()
        .collect();
    n
}

/// The band's node file, signed by Nia as it is.
fn band(w: &World) -> File {
    let reg = chain::open_group_in(&solar(), &w.sy, &w.pr).unwrap();
    chain::sign_node(&from_release(BAND), BAND, "Nia Node", &key(16), AT, &reg)
        .expect("Nia signs the band")
        .0
}

fn seal(
    w: &World,
    group: &File,
    nodes: &[File],
    version: &str,
    previous: Option<&str>,
    by: &str,
    k: u8,
) -> Result<File, String> {
    release::seal(
        group,
        nodes,
        version,
        previous,
        by,
        &key(k),
        AT,
        APP,
        &w.sy,
        &w.pr,
    )
    .map_err(|e| e.message().to_string())
}

const RELEASED: &str = "2026.10.1";

/// Solar 1.2, sealed by Ola from the band Nia signed.
fn solar_12(w: &World) -> File {
    seal(w, &solar(), &[band(w)], "1.2", Some("1.1"), "Ola Sun", 13).unwrap()
}

fn checks_of(release: Result<&File, &str>, released: &str) -> Integration {
    let taken = release.map(|f| Taken {
        version: "1.2",
        chain: "sealed by Ola Sun, its owner as the systems' file mounts it",
        file: f,
    });
    design::integration(
        &[Part {
            group: "l3_solar",
            taken,
        }],
        released,
    )
}

fn line(i: &Integration, c: Check) -> &design::Line {
    i.lines.iter().find(|l| l.check == c).unwrap()
}

/// `f` as the drive holds it: its bytes as a file of the one schema.
fn bytes(f: &File) -> Vec<u8> {
    let dir = std::env::temp_dir().join(format!(
        "vleo-design-bytes-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let at = dir.join("f.vleo");
    let _ = std::fs::remove_file(&at);
    let b = vleo_files::sqlite::written(f, &at).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    b
}

#[allow(clippy::too_many_arguments)]
fn release_it(
    w: &World,
    i: &Integration,
    r: &File,
    version: &str,
    previous: Option<&str>,
    changed: &str,
    by: &str,
    k: u8,
) -> Result<File, String> {
    release_marked(w, i, r, version, previous, changed, &[], &[], by, k)
}

#[allow(clippy::too_many_arguments)]
fn release_marked(
    w: &World,
    i: &Integration,
    r: &File,
    version: &str,
    previous: Option<&str>,
    changed: &str,
    marks: &[Mark],
    understood: &[String],
    by: &str,
    k: u8,
) -> Result<File, String> {
    let (rb, pb, sb) = (bytes(r), bytes(&w.pr.file), bytes(&w.sy.file));
    let held = Held {
        releases: vec![(
            "l3_solar",
            "1.2",
            r.meta.get("fingerprint").map_or("", String::as_str),
            Whole {
                path: "groups/l3_solar/releases/l3_solar-1.2.vleo",
                bytes: &rb,
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
    let asked = Asked {
        version,
        previous,
        changed,
        marks,
        understood,
    };
    design::release(i, &held, &asked, by, &key(k), AT, APP, &w.pr)
        .map_err(|e| e.message().to_string())
}

#[test]
fn today_s_design_is_checked_a_line_for_each_of_w8_s_five_checks() {
    let w = world();
    let r = solar_12(&w);
    let i = checks_of(Ok(&r), "");
    assert_eq!(i.lines.len(), 5);
    assert!(i.holds(), "{:?}", i.failing());
    assert!(line(&i, Check::Seal)
        .says
        .starts_with("l3_solar 1.2 is taken"));
    assert!(line(&i, Check::Base).says.contains("before the first"));
}

#[test]
fn each_check_fails_by_its_own_line() {
    let w = world();
    let r = solar_12(&w);
    // No release taken: the seal fails, and nothing else of it is checked.
    let i = checks_of(
        Err("its seal does not check through the chain: changed after the seal"),
        "",
    );
    assert!(!i.holds());
    assert!(!line(&i, Check::Seal).holds);
    assert!(line(&i, Check::Seal)
        .says
        .contains("changed after the seal"));
    assert!(i.lines.iter().all(|l| !l.holds));
    // A base that does not fit the design released now.
    let i = checks_of(Ok(&r), RELEASED);
    assert!(!line(&i, Check::Base).holds);
    assert!(
        line(&i, Check::Base)
            .says
            .contains("the design released now is 2026.10.1"),
        "{}",
        line(&i, Check::Base).says
    );
    assert!(
        i.lines.iter().filter(|l| !l.holds).count() == 1,
        "{:?}",
        i.failing()
    );
    // A node whose checks refuse it: its method gone.
    let mut bad = r.clone();
    bad.texts
        .retain(|t| !(t.scope == BAND && t.kind == "pseudocode"));
    let i = checks_of(Ok(&bad), "");
    assert!(!line(&i, Check::Nodes).holds, "{:?}", i.lines);
    assert!(line(&i, Check::Assistant).holds);
    // A relation an assistant supplied.
    let mut bad = r.clone();
    for t in bad
        .tables
        .iter_mut()
        .filter(|t| t.scope == BAND && t.path == "declaration.csv")
    {
        t.csv = "author,ai,date,source,checked_by\nNia Node,relation,2026-10-17,,\n".into();
    }
    let i = checks_of(Ok(&bad), "");
    assert!(!line(&i, Check::Assistant).holds, "{:?}", i.lines);
    assert!(
        line(&i, Check::Assistant)
            .says
            .contains("an assistant supplied the relation"),
        "{}",
        line(&i, Check::Assistant).says
    );
    assert!(
        line(&i, Check::Nodes).holds,
        "{}",
        line(&i, Check::Nodes).says
    );
    // A range declared by one end.
    let mut bad = r.clone();
    let p = bad
        .ports
        .iter_mut()
        .find(|p| p.block_uid == BAND && p.direction == "in")
        .unwrap();
    p.lower = s("0");
    p.upper = s("");
    let name = p.name.clone();
    let i = checks_of(Ok(&bad), "");
    assert!(!line(&i, Check::Ranges).holds);
    assert!(
        line(&i, Check::Ranges)
            .says
            .contains(&format!("{BAND}.{name} declares one end of its range only")),
        "{}",
        line(&i, Check::Ranges).says
    );
    // A range whose lower end is above its upper.
    let mut bad = r.clone();
    let p = bad
        .ports
        .iter_mut()
        .find(|p| p.block_uid == BAND && p.direction == "in")
        .unwrap();
    p.lower = s("2");
    p.upper = s("1");
    let i = checks_of(Ok(&bad), "");
    assert!(
        line(&i, Check::Ranges)
            .says
            .contains(&format!("{BAND}.{name} runs from 2 to 1")),
        "{}",
        line(&i, Check::Ranges).says
    );
}

#[test]
fn the_system_engineer_releases_the_design_whole_and_it_opens_through_the_chain() {
    let w = world();
    let r = solar_12(&w);
    let i = checks_of(Ok(&r), "");
    let d = release_it(
        &w,
        &i,
        &r,
        RELEASED,
        None,
        "Solar's bands, said plainly.",
        "Sam Engel",
        12,
    )
    .expect("Sam releases the design");
    assert_eq!(d.kind().unwrap(), Kind::Design);
    for (k, v) in [
        ("version", RELEASED),
        ("previous", ""),
        ("released_by", "Sam Engel"),
        ("oldest_application", "1.0.0"),
    ] {
        assert_eq!(d.meta.get(k).map(String::as_str), Some(v), "{k}");
    }
    // Each file whole, by its bytes and their SHA-256.
    let solar = d
        .media
        .iter()
        .find(|m| m.path == "groups/l3_solar/releases/l3_solar-1.2.vleo")
        .unwrap();
    assert_eq!(solar.bytes, bytes(&r));
    assert_eq!(
        solar.sha256,
        vleo_files::keys::hex(&vleo_files::keys::sha256(&bytes(&r)))
    );
    for p in [
        "groups/programme/programme.vgroup",
        "groups/systems/systems.vgroup",
    ] {
        assert!(d.media.iter().any(|m| m.path == p), "{p}");
    }
    let t = d.tables.iter().find(|t| t.path == "released.csv").unwrap();
    assert!(t.csv.contains("l3_solar,1.2,"), "{}", t.csv);
    assert!(d
        .texts
        .iter()
        .any(|t| t.kind == "changed" && t.body == "Solar's bands, said plainly."));
    chain::check_design(&d, &w.pr).expect("the design opens through the chain");
    assert_eq!(
        d.signatures
            .iter()
            .find(|r| r.scope == FILE)
            .unwrap()
            .signer,
        "Sam Engel"
    );
    // Sid, Sam's deputy, releases it too.
    release_it(&w, &i, &r, RELEASED, None, "Solar.", "Sid Side", 17)
        .expect("Sam's deputy releases");
}

#[test]
fn a_release_of_the_design_is_refused_whole_and_by_name() {
    let w = world();
    let r = solar_12(&w);
    // A line failing: refused whole, naming it.
    let i = checks_of(Ok(&r), RELEASED);
    let e = release_it(
        &w,
        &i,
        &r,
        "2026.10.2",
        Some(RELEASED),
        "Solar.",
        "Sam Engel",
        12,
    )
    .unwrap_err();
    assert!(
        e.starts_with(
            "the design is not released, refused whole: l3_solar, the base still fitting"
        ),
        "{e}"
    );
    let i = checks_of(Ok(&r), "");
    for (version, previous, changed, by, k, says) in [
        (
            RELEASED,
            None,
            "Solar.",
            "Ola Sun",
            13,
            "Ola Sun may not release the design",
        ),
        (
            RELEASED,
            None,
            "Solar.",
            "Pat Morgan",
            11,
            "Pat Morgan may not release the design",
        ),
        (
            "2026.13.1",
            None,
            "Solar.",
            "Sam Engel",
            12,
            "\"2026.13.1\" is not a design's version",
        ),
        (
            "1.2",
            None,
            "Solar.",
            "Sam Engel",
            12,
            "\"1.2\" is not a design's version",
        ),
        (
            RELEASED,
            Some(RELEASED),
            "Solar.",
            "Sam Engel",
            12,
            "2026.10.1 does not come after 2026.10.1",
        ),
        (
            "2026.9.4",
            Some(RELEASED),
            "Solar.",
            "Sam Engel",
            12,
            "does not come after",
        ),
        (RELEASED, None, "  ", "Sam Engel", 12, "say what changed"),
        (
            RELEASED,
            None,
            "Solar.",
            "Claude",
            12,
            "Claude is an assistant's name",
        ),
    ] {
        let e = release_it(&w, &i, &r, version, previous, changed, by, k).unwrap_err();
        assert!(e.contains(says), "{by} {version}: {e}");
    }
    // Sam's key under Sid's name signs nothing.
    let e = release_it(&w, &i, &r, RELEASED, None, "Solar.", "Sid Side", 12).unwrap_err();
    assert!(
        e.contains("is not one the programme's file registers"),
        "{e}"
    );
}

fn mark(id: &str, said: &str, raised: Option<i64>) -> Mark {
    Mark {
        id: s(id),
        said: s(said),
        raised,
    }
}

#[test]
fn every_red_mark_is_understood_or_raised_before_the_design_is_released() {
    let w = world();
    let r = solar_12(&w);
    let i = checks_of(Ok(&r), "");
    let marks = [
        mark(
            "closure:k_drag",
            "k_drag fails: the drag is above its bound",
            None,
        ),
        mark("closure:k_power", "k_power is blocked", Some(4)),
    ];
    // Neither marked: the one not raised refuses it, by what it says.
    let e = release_marked(
        &w,
        &i,
        &r,
        RELEASED,
        None,
        "Solar.",
        &marks,
        &[],
        "Sam Engel",
        12,
    )
    .unwrap_err();
    assert!(e.contains("every red mark is understood"), "{e}");
    assert!(
        e.contains("k_drag fails: the drag is above its bound"),
        "{e}"
    );
    assert!(!e.contains("k_power"), "{e}");
    // A mark today's design does not have is refused, never ignored.
    let e = release_marked(
        &w,
        &i,
        &r,
        RELEASED,
        None,
        "Solar.",
        &marks,
        &[s("closure:k_drag"), s("closure:gone")],
        "Sam Engel",
        12,
    )
    .unwrap_err();
    assert!(
        e.contains("closure:gone is marked understood, and today's design has no such red mark"),
        "{e}"
    );
    // Understood: released, and the record says which was which.
    let d = release_marked(
        &w,
        &i,
        &r,
        RELEASED,
        None,
        "Solar.",
        &marks,
        &[s("closure:k_drag")],
        "Sam Engel",
        12,
    )
    .expect("released with every red mark understood or raised");
    let t = d.tables.iter().find(|t| t.path == "red_marks.csv").unwrap();
    assert_eq!(
        t.csv,
        "mark,said,how\n\
         closure:k_drag,k_drag fails: the drag is above its bound,understood by Sam Engel\n\
         closure:k_power,k_power is blocked,raised as issue 4\n"
    );
}

#[test]
fn the_released_design_is_written_readable_as_csv() {
    let w = world();
    let r = solar_12(&w);
    let i = checks_of(Ok(&r), "");
    let d = release_it(&w, &i, &r, RELEASED, None, "Solar.", "Sam Engel", 12).unwrap();
    let files = design::readable(&d, &[("l3_solar", &r)]);
    let names: Vec<&str> = files.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        names,
        [
            "Groups.csv",
            "Nodes.csv",
            "Ports.csv",
            "Wires.csv",
            "Closures.csv",
            "Status.csv"
        ]
    );
    let of = |n: &str| &files.iter().find(|(x, _)| *x == n).unwrap().1;
    assert!(
        of("Groups.csv").contains("\nl3_solar,1.2,Ola Sun,"),
        "{}",
        of("Groups.csv")
    );
    assert!(
        of("Groups.csv").trim_end().ends_with(",2026.10.1"),
        "{}",
        of("Groups.csv")
    );
    assert!(
        of("Nodes.csv").contains(&format!("\nl3_solar,{BAND},l3_solar,")),
        "{}",
        of("Nodes.csv")
    );
    let band_ports = r.ports.iter().filter(|p| p.block_uid == BAND).count();
    assert_eq!(
        of("Ports.csv")
            .lines()
            .filter(|l| l.starts_with(&format!("l3_solar,{BAND},")))
            .count(),
        band_ports
    );
    // The status is the checks the design was released on, a line each.
    assert_eq!(
        of("Status.csv").lines().count(),
        1 + 5,
        "{}",
        of("Status.csv")
    );
    assert!(
        of("Status.csv").contains("l3_solar,the base still fitting,"),
        "{}",
        of("Status.csv")
    );
}

#[test]
fn a_design_s_version_follows_the_design_released_now() {
    for (version, previous, says) in [
        ("2026.10.1", None, None),
        ("2026.10.3", None, Some("2026.10.3 is not the first of its month")),
        ("2026.10.2", Some("2026.10.1"), None),
        ("2026.11.1", Some("2026.10.1"), None),
        ("2026.10.3", Some("2026.10.1"), Some("does not follow 2026.10.1, the design released now: the next in 2026.10 is 2026.10.2")),
        ("2026.11.2", Some("2026.10.1"), Some("the next in 2026.11 is 2026.11.1")),
        ("2026.9.1", Some("2026.10.1"), Some("does not come after")),
    ] {
        match (design::follows(version, previous), says) {
            (Ok(()), None) => {}
            (Err(e), Some(s)) => assert!(e.contains(s), "{version} after {previous:?}: {e}"),
            (got, _) => panic!("{version} after {previous:?}: {got:?}"),
        }
    }
    // And the release itself is held to it.
    let w = world();
    let r = solar_12(&w);
    let i = checks_of(Ok(&r), "");
    let e = release_it(
        &w,
        &i,
        &r,
        "2026.10.3",
        Some("2026.10.1"),
        "Solar.",
        "Sam Engel",
        12,
    )
    .unwrap_err();
    assert!(e.contains("does not follow 2026.10.1"), "{e}");
}

#[test]
fn a_released_design_holds_what_was_checked_and_nothing_beside() {
    let w = world();
    let r = solar_12(&w);
    // Checked for solar and power; held for solar only.
    let mut i = checks_of(Ok(&r), "");
    let mut power = i.lines.clone();
    for l in power.iter_mut() {
        l.group = s("l3_power");
    }
    i.lines.extend(power);
    let e = release_it(&w, &i, &r, RELEASED, None, "Solar.", "Sam Engel", 12).unwrap_err();
    assert!(
        e.contains("the releases held (l3_solar) are not the groups checked (l3_power, l3_solar)"),
        "{e}"
    );
}

#[test]
fn only_a_computed_node_declares_its_method_and_an_assistant_s_signature_is_the_assistant_s_line() {
    let w = world();
    let r = solar_12(&w);
    // A stated node with no declaration: sealing asks none of it, nor do the
    // checks.
    let mut stated = r.clone();
    stated.blocks.push(Block {
        uid: s("sw_stated"),
        id: s("sw_stated"),
        parent_uid: s("l3_solar"),
        behaviour: s("stated"),
        question: s("What is stated?"),
        contract_version: 1,
        ..Block::default()
    });
    let i = checks_of(Ok(&stated), "");
    assert!(
        line(&i, Check::Assistant).holds,
        "{}",
        line(&i, Check::Assistant).says
    );
    // A signature in an assistant's name is the assistant's line, not the
    // nodes'.
    let mut signed = r.clone();
    let mut row = signed
        .signatures
        .iter()
        .find(|x| x.scope == FILE)
        .unwrap()
        .clone();
    row.signer = s("Claude");
    signed.signatures.push(row);
    let i = checks_of(Ok(&signed), "");
    assert!(!line(&i, Check::Assistant).holds);
    assert!(
        line(&i, Check::Assistant)
            .says
            .contains("an assistant's name"),
        "{}",
        line(&i, Check::Assistant).says
    );
    assert!(
        line(&i, Check::Nodes).holds,
        "{}",
        line(&i, Check::Nodes).says
    );
}

#[test]
fn a_range_s_ends_are_finite_numbers() {
    let w = world();
    let mut r = solar_12(&w);
    let p = r
        .ports
        .iter_mut()
        .find(|p| p.block_uid == BAND && p.direction == "in")
        .unwrap();
    p.lower = s("0");
    p.upper = s("inf");
    let name = p.name.clone();
    let i = checks_of(Ok(&r), "");
    assert!(
        line(&i, Check::Ranges).says.contains(&format!(
            "{BAND}.{name}'s range, 0 to inf, is not two numbers"
        )),
        "{}",
        line(&i, Check::Ranges).says
    );
}

/// `d` signed again by Sam as it is: a released design whose signature
/// counts, whatever it holds, so what is read back is checked on its own.
fn resigned(mut d: File) -> File {
    d.signatures.retain(|r| r.scope != FILE);
    let row = chain::sign(&d, FILE, "Sam Engel", &key(12), AT, "ok", "").unwrap();
    d.signatures.push(row);
    d
}

#[test]
fn a_released_design_is_read_back_as_it_was_released() {
    let w = world();
    let r = solar_12(&w);
    let i = checks_of(Ok(&r), "");
    let d = release_it(&w, &i, &r, RELEASED, None, "Solar.", "Sam Engel", 12).unwrap();
    let o = design::open_held(&d, &w.pr, APP).expect("read back");
    assert_eq!(o.version, RELEASED);
    assert_eq!(o.releases.len(), 1);
    let h = &o.releases[0];
    assert_eq!((h.group.as_str(), h.version.as_str()), ("l3_solar", "1.2"));
    assert_eq!(h.back.bytes, bytes(&r));
    assert_eq!(
        h.back.file.meta.get("version").map(String::as_str),
        Some("1.2")
    );
    assert_eq!(
        o.programme.file.meta.get("group_id").map(String::as_str),
        Some("programme")
    );
    assert_eq!(
        o.systems.file.meta.get("group_id").map(String::as_str),
        Some("systems")
    );

    let solar = "groups/l3_solar/releases/l3_solar-1.2.vleo";
    let refused = |d: File| {
        design::open_held(&d, &w.pr, APP)
            .unwrap_err()
            .message()
            .to_string()
    };
    // Changed after it was signed: the chain refuses it.
    let mut x = d.clone();
    x.media
        .iter_mut()
        .find(|m| m.path == solar)
        .unwrap()
        .bytes
        .push(0);
    assert!(
        refused(x).contains("by the system engineer"),
        "a signature that does not count"
    );
    // Signed as it is, and each thing it holds wrong, by name.
    let mut x = d.clone();
    x.media
        .iter_mut()
        .find(|m| m.path == solar)
        .unwrap()
        .bytes
        .push(0);
    let e = refused(resigned(x));
    assert!(
        e.contains(&format!(
            "holds {solar} whose bytes are not the ones it was released with"
        )),
        "{e}"
    );
    let mut x = d.clone();
    let m = x.media.iter_mut().find(|m| m.path == solar).unwrap();
    m.bytes = bytes(&w.sy.file);
    m.sha256 = vleo_files::keys::hex(&vleo_files::keys::sha256(&m.bytes));
    let e = refused(resigned(x));
    assert!(
        e.contains(&format!("holds {solar}, which is not l3_solar's file")),
        "{e}"
    );
    let mut x = d.clone();
    x.media.retain(|m| m.path != solar);
    let e = refused(resigned(x));
    assert!(e.contains(&format!("does not hold {solar}")), "{e}");
    let mut x = d.clone();
    for t in x.tables.iter_mut().filter(|t| t.path == "released.csv") {
        t.csv = s("group,version,fingerprint,path,sha256\n");
    }
    let e = refused(resigned(x));
    assert!(
        e.contains(&format!("holds {solar}, which released.csv does not name")),
        "{e}"
    );
    let mut x = d.clone();
    for t in x.tables.iter_mut().filter(|t| t.path == "released.csv") {
        t.csv = t.csv.replace("l3_solar,1.2,", "l3_solar,1.3,");
    }
    let e = refused(resigned(x));
    assert!(
        e.contains(&format!(
            "holds {solar} as l3_solar 1.3, and it is l3_solar 1.2"
        )),
        "{e}"
    );
    let mut x = d.clone();
    let sha = x
        .media
        .iter()
        .find(|m| m.path == solar)
        .unwrap()
        .sha256
        .clone();
    for t in x.tables.iter_mut().filter(|t| t.path == "released.csv") {
        t.csv = t.csv.replace(&sha, &"0".repeat(64));
    }
    let e = refused(resigned(x));
    assert!(
        e.contains("whose SHA-256 is not the one released.csv names"),
        "{e}"
    );
    let mut x = d.clone();
    x.media
        .retain(|m| m.path != "groups/systems/systems.vgroup");
    let e = refused(resigned(x));
    assert!(
        e.contains("does not hold groups/systems/systems.vgroup"),
        "{e}"
    );
    let mut x = d.clone();
    let p = x
        .media
        .iter()
        .find(|m| m.path == "groups/programme/programme.vgroup")
        .unwrap()
        .clone();
    let sy = x
        .media
        .iter_mut()
        .find(|m| m.path == "groups/systems/systems.vgroup")
        .unwrap();
    sy.bytes = p.bytes;
    sy.sha256 = p.sha256;
    let e = refused(resigned(x));
    assert!(e.contains("which is not systems's file"), "{e}");
}

#[test]
fn every_released_design_on_the_drive_is_read_back_oldest_first() {
    let w = world();
    let r = solar_12(&w);
    let i = checks_of(Ok(&r), "");
    let dir = std::env::temp_dir().join(format!("vleo-released-in-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (v, prev) in [("2026.10.1", None), ("2026.10.2", Some("2026.10.1"))] {
        let d = release_it(&w, &i, &r, v, prev, "Solar.", "Sam Engel", 12).unwrap();
        vleo_files::sqlite::write(&d, &dir.join(format!("design-{v}.vleo"))).unwrap();
    }
    std::fs::write(dir.join("NOTES.md"), "# notes\n").unwrap();
    let all = design::released_in(&[&dir], &w.pr, APP).unwrap();
    let versions: Vec<&str> = all.iter().map(|o| o.version.as_str()).collect();
    assert_eq!(versions, ["2026.10.1", "2026.10.2"]);
    // A file named for no version is refused by name, never passed over.
    std::fs::write(dir.join("design-x.vleo"), b"not one").unwrap();
    let e = design::released_in(&[&dir], &w.pr, APP).unwrap_err();
    assert!(
        e.message()
            .contains("design-x.vleo is not named for a design's version"),
        "{}",
        e.message()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The programme's file as `programme` writes it, with Ola's key revoked
/// from `from`, signed again by Pat.
fn programme_revoking_ola(from: &str) -> Registry {
    let mut f = programme().file.clone();
    for k in f.keys.iter_mut().filter(|k| k.person == "Ola Sun") {
        k.revoked_from = s(from);
    }
    f.signatures.retain(|r| r.scope != FILE);
    chain::open_programme(
        signed(f, "Pat Morgan", &key(11)),
        &key(11).public().fingerprint(),
    )
    .unwrap()
}

#[test]
fn a_design_is_released_again_under_a_new_version_holding_what_it_held() {
    let w = world();
    let r = solar_12(&w);
    let i = checks_of(Ok(&r), "");
    let one = release_it(&w, &i, &r, "2026.10.1", None, "Solar.", "Sam Engel", 12).unwrap();
    let old = design::open_held(&one, &w.pr, APP).unwrap();
    let again =
        |version: &str, previous: Option<&str>, why: &str, by: &str, k: u8, pr: &Registry| {
            design::release_again(&old, version, previous, why, by, &key(k), AT, APP, pr)
                .map_err(|e| e.message().to_string())
        };
    // 2026.10.2 was released after it, and was wrong: .1 again, as .3.
    let d = again(
        "2026.10.3",
        Some("2026.10.2"),
        "2026.10.2's band was wrong.",
        "Sam Engel",
        12,
        &w.pr,
    )
    .expect("released again");
    assert_eq!(d.media, one.media, "the same files, byte for byte");
    for (k, v) in [
        ("version", "2026.10.3"),
        ("previous", "2026.10.2"),
        ("again_of", "2026.10.1"),
        ("released_by", "Sam Engel"),
    ] {
        assert_eq!(d.meta.get(k).map(String::as_str), Some(v), "{k}");
    }
    let o = design::open_held(&d, &w.pr, APP).expect("it reads back");
    assert_eq!(o.releases[0].back.bytes, old.releases[0].back.bytes);
    for t in ["released.csv", "integration.csv", "red_marks.csv"] {
        assert_eq!(
            d.tables.iter().find(|x| x.path == t).map(|x| &x.csv),
            one.tables.iter().find(|x| x.path == t).map(|x| &x.csv),
            "{t}"
        );
    }
    assert!(d.texts.iter().any(|t| t.kind == "changed"
        && t.body
            .contains("released again, as 2026.10.3: 2026.10.2's band was wrong.")));
    // Sid, Sam's deputy, too.
    again(
        "2026.10.3",
        Some("2026.10.2"),
        "Wrong.",
        "Sid Side",
        17,
        &w.pr,
    )
    .expect("the deputy");
    // Refused, by name.
    for (version, previous, why, by, k, says) in [
        (
            "2026.10.3",
            Some("2026.10.2"),
            "Wrong.",
            "Ola Sun",
            13,
            "Ola Sun may not release a design again",
        ),
        (
            "2026.10.2",
            Some("2026.10.1"),
            "Wrong.",
            "Sam Engel",
            12,
            "2026.10.1 is the design released now",
        ),
        (
            "2026.10.3",
            Some("2026.10.2"),
            "  ",
            "Sam Engel",
            12,
            "say why 2026.10.1 is released again",
        ),
        (
            "2026.10.5",
            Some("2026.10.2"),
            "Wrong.",
            "Sam Engel",
            12,
            "does not follow 2026.10.2",
        ),
        (
            "2026.10.3",
            Some("2026.10.2"),
            "Wrong.",
            "Claude",
            12,
            "Claude is an assistant's name",
        ),
    ] {
        let e = again(version, previous, why, by, k, &w.pr).unwrap_err();
        assert!(e.contains(says), "{by} {version}: {e}");
    }
    // The programme's file now says Ola's key was revoked before the seal
    // was made: the seal it holds does not check through the chain now.
    let revoked = programme_revoking_ola("2026-10-16");
    let e = again(
        "2026.10.3",
        Some("2026.10.2"),
        "Wrong.",
        "Sam Engel",
        12,
        &revoked,
    )
    .unwrap_err();
    assert!(
        e.contains(
            "design 2026.10.1 holds l3_solar 1.2, which does not check through the chain now"
        ) && e.contains("with a key revoked from 2026-10-16"),
        "{e}"
    );
    // A key revoked after the seal still counts for it, as the chain counts
    // every signature made while its key counted.
    let later = programme_revoking_ola("2026-10-18");
    again(
        "2026.10.3",
        Some("2026.10.2"),
        "Wrong.",
        "Sam Engel",
        12,
        &later,
    )
    .expect("a seal made before its key was revoked still counts");
}
