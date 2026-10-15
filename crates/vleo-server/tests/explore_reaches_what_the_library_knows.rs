//! The Explore workspace's routes reach what the library already knows
//! (docs/PLAN_F.md, stage F2): which design is open and what built it, the
//! health map, a failing closure traced to the release that broke it, and two
//! releases compared block by block.
//!
//! One drive, two solar releases: 1.1 as accepted, and a 1.2 whose hot edge
//! of the sustained F10.7 band breaks the first solar closure. Today's design
//! takes 1.2, so the spacecraft fails, and the trace names the node 1.2
//! changed, with the release.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sql(file: &Path, statement: &str) {
    let db = rusqlite::Connection::open(file).unwrap();
    assert!(db.execute(statement, []).unwrap() > 0, "{statement}");
}

fn get(port: u16, path: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        s,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out.split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or(out)
}

/// The text of the object in `body` that starts at the first `"id":"<id>"`.
fn object_of<'a>(body: &'a str, id: &str) -> &'a str {
    let at = body
        .find(&format!("\"id\":\"{id}\""))
        .unwrap_or_else(|| panic!("no {id} in {body}"));
    let end = body[at..].find('}').map_or(body.len(), |e| at + e);
    &body[at..end]
}

#[test]
fn explore_answers_from_the_drive_s_releases() {
    let s = std::env::temp_dir().join(format!("vleo-explore-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    std::fs::create_dir_all(&s).unwrap();
    std::env::set_var("VLEO_CASE", s.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", s.join("results"));
    std::env::set_var("VLEO_LOG", s.join("log"));
    std::env::set_var("VLEO_DATA", s.join("data"));
    std::env::remove_var("VLEO_DESIGN");

    let rel = s.join("drive/groups/l3_solar/releases");
    std::fs::create_dir_all(&rel).unwrap();
    let fixture = root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo");
    std::fs::copy(&fixture, rel.join("l3_solar-1.1.vleo")).unwrap();
    let f = rel.join("l3_solar-1.2.vleo");
    std::fs::copy(&fixture, &f).unwrap();
    sql(&f, "UPDATE meta SET value = '1.2' WHERE key = 'version'");
    sql(
        &f,
        "UPDATE tbl SET csv = csv || '1.2,2026-10-06,Aman Rai,\
         The band needed a wider margin when the cycle is low.,\
         The multiplier was read again.,\
         A wider band is safer.,\
         The multiplier of the hot edge.,,,\
         the one-sided 90th percentile,a multiplier from another table\n' \
         WHERE scope = 'group' AND path = 'versions.csv'",
    );
    sql(
        &f,
        "UPDATE doc SET body = replace(body, 'return sustained', \
         'if central < 95 then\n  return central + spread * 14\nend\nreturn sustained') \
         WHERE kind = 'pseudocode' AND scope = 'sw_f107_design_long'",
    );
    let r = vleo_files::intake::Release::open(&f).unwrap();
    let fp = vleo_files::seal::fingerprint_of(&r.folder, "group");
    sql(
        &f,
        &format!("UPDATE meta SET value = '{fp}' WHERE key = 'fingerprint'"),
    );

    // The design's own files: not today's design, and it says so.
    std::env::remove_var("VLEO_DRIVE");
    let port = vleo_server::serve(Some(root()), 19411, false, true).unwrap();
    let today = get(port, "/v1/today");
    assert!(today.contains("\"today\":false"), "{today}");
    assert!(today.contains("\"groups\":[]"), "{today}");
    let refused = get(
        port,
        "/v1/compare?a=drive:groups/l3_solar/releases/l3_solar-1.1.vleo&b=drive:x",
    );
    assert!(
        refused.contains("\"ok\":false") && refused.contains("no drive"),
        "{refused}"
    );

    // Today's design, from the drive.
    std::env::set_var("VLEO_DRIVE", s.join("drive"));
    let port = vleo_server::serve(Some(root()), 19431, false, true).unwrap();
    std::env::remove_var("VLEO_DRIVE");

    let today = get(port, "/v1/today");
    assert!(today.contains("\"today\":true"), "{today}");
    let solar = &today[today.find("\"id\":\"l3_solar\"").expect(&today)..];
    assert!(solar.contains("\"version\":\"1.2\""), "{today}");
    assert!(solar.contains("sw_f107_design_long"), "{today}");
    assert!(
        solar.contains("not yet checked through the chain"),
        "{today}"
    );

    // The map: the spacecraft as bad as its worst group, and the closure 1.2
    // breaks among the nodes that fail.
    let map = get(port, "/v1/health?inputs=defaults");
    assert!(map.contains("\"ok\":true"), "{map}");
    assert!(map.contains("\"spacecraft\":\"fails\""), "{map}");
    let ach = object_of(&map, "l3_solar_ach_01");
    assert!(ach.contains("\"state\":\"fails\""), "{ach}");
    assert!(ach.contains("\"closure\":true"), "{ach}");

    // The trace names the node the release changed, with the release.
    let t = get(
        port,
        "/v1/health/trace?inputs=defaults&closure=l3_solar_ach_01",
    );
    assert!(t.contains("\"ok\":true"), "{t}");
    assert!(t.contains("\"state\":\"fails\""), "{t}");
    let causes = &t[t.find("\"causes\":[").expect(&t)..];
    let first = object_of(causes, "sw_f107_design_long");
    assert!(
        causes.starts_with("\"causes\":[{\"id\":\"sw_f107_design_long\""),
        "the nearest cause is not the node 1.2 changed: {t}"
    );
    assert!(first.contains("changed by l3_solar 1.2"), "{first}");
    assert!(first.contains("\"group\":\"l3_solar\""), "{first}");

    // A row that is not a closure, and one that does not exist, are refused
    // by name.
    let t = get(port, "/v1/health/trace?closure=sw_f107_design_long");
    assert!(
        t.contains("\"ok\":false") && t.contains("is not a closure"),
        "{t}"
    );
    let t = get(port, "/v1/health/trace?closure=no_such_row");
    assert!(t.contains("\"ok\":false") && t.contains("no node"), "{t}");

    // The two releases compared: the method 1.2 changed, and nothing else in
    // any block.
    let c = get(
        port,
        "/v1/compare?a=drive:groups/l3_solar/releases/l3_solar-1.1.vleo\
         &b=drive:groups/l3_solar/releases/l3_solar-1.2.vleo",
    );
    assert!(c.contains("\"ok\":true"), "{c}");
    assert!(c.contains("\"same_content\":false"), "{c}");
    let changed = object_of(
        c.split_once("\"blocks\":").expect(&c).1,
        "sw_f107_design_long",
    );
    assert!(changed.contains("\"status\":\"changed\""), "{changed}");

    // A file is the same as itself; a path out of the drive, a name with no
    // place, and a file that is not there are each refused by name.
    let same = get(
        port,
        "/v1/compare?a=drive:groups/l3_solar/releases/l3_solar-1.1.vleo\
         &b=drive:groups/l3_solar/releases/l3_solar-1.1.vleo",
    );
    assert!(same.contains("\"same_content\":true"), "{same}");
    for (a, says) in [
        ("drive:../escape.vleo", "not a path inside"),
        ("/etc/passwd", "names no file"),
        ("design:groups/nothing.vnode", "no such file"),
    ] {
        let c = get(port, &format!("/v1/compare?a={a}&b={a}"));
        assert!(c.contains("\"ok\":false") && c.contains(says), "{a}: {c}");
    }
    let _ = std::fs::remove_dir_all(&s);
}
