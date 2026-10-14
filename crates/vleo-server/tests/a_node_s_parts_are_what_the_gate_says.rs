//! A node's six parts are what the gate and the sheet say (docs/PLAN_F.md,
//! stage F3): every part named, in the order of the system model; a node not
//! decided yet missing how it answers; a node the gate refuses failing in the
//! part the refusal is about, with the gate's words; and a node that does not
//! exist refused by name.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let dest = to.join(p.file_name().unwrap());
        if p.is_dir() {
            copy(&p, &dest);
        } else {
            std::fs::copy(&p, &dest).unwrap();
        }
    }
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
        .map(|(_, b)| b.to_string())
        .unwrap_or(out)
}

/// The text of part `name` in a parts answer.
fn part<'a>(body: &'a str, name: &str) -> &'a str {
    let at = body
        .find(&format!("\"name\":\"{name}\""))
        .unwrap_or_else(|| panic!("no part {name} in {body}"));
    let end = body[at..].find("]}").map_or(body.len(), |e| at + e);
    &body[at..end]
}

#[test]
fn a_node_s_parts_are_the_gate_s_and_the_sheet_s() {
    let s = std::env::temp_dir().join(format!("vleo-parts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    std::fs::create_dir_all(&s).unwrap();
    std::env::set_var("VLEO_CASE", s.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", s.join("results"));
    std::env::set_var("VLEO_LOG", s.join("log"));
    std::env::set_var("VLEO_DATA", s.join("data"));
    std::env::remove_var("VLEO_DRIVE");

    // One fixture's expected value said to come from the code under test.
    let design = s.join("design");
    copy(&root().join("design"), &design);
    let node = design.join("groups/l3_solar/nodes/sw_ap_design.vnode");
    let db = rusqlite::Connection::open(&node).unwrap();
    let changed = db
        .execute(
            "UPDATE test_case SET provenance = 'self-snapshot' \
             WHERE rowid = (SELECT min(rowid) FROM test_case)",
            [],
        )
        .unwrap();
    assert_eq!(changed, 1, "the node holds no case");
    drop(db);
    std::env::set_var("VLEO_DESIGN", &design);
    let port = vleo_server::serve(Some(root()), 19511, false, true).unwrap();
    std::env::remove_var("VLEO_DESIGN");

    let p = get(port, "/v1/parts/sw_ap_design");
    assert!(p.contains("\"ok\":true"), "{p}");
    let names: Vec<usize> = vleo_sheet::parts::PARTS
        .iter()
        .map(|n| p.find(&format!("\"name\":\"{n}\"")).unwrap_or(usize::MAX))
        .collect();
    assert!(
        names.windows(2).all(|w| w[0] < w[1]) && names[5] != usize::MAX,
        "the six parts are not all there, in order: {p}"
    );
    let evidence = part(&p, "evidence");
    assert!(evidence.contains("\"state\":\"failing\""), "{evidence}");
    assert!(evidence.contains("provenance"), "{evidence}");
    assert!(part(&p, "identity").contains("\"state\":\"done\""), "{p}");

    // A node nobody has decided yet: how it answers is missing.
    let tree = vleo_files::convert::open(&root()).unwrap().0;
    let open = tree
        .ordered()
        .into_iter()
        .find(|sh| sh.behaviour() == "open")
        .expect("the design has an open node")
        .id
        .clone();
    let p = get(port, &format!("/v1/parts/{open}"));
    assert!(p.contains("\"behaviour\":\"open\""), "{p}");
    let behaviour = part(&p, "behaviour");
    assert!(behaviour.contains("\"state\":\"missing\""), "{behaviour}");
    assert!(behaviour.contains("how it answers"), "{behaviour}");

    let p = get(port, "/v1/parts/no_such_node");
    assert!(
        p.contains("\"ok\":false") && p.contains("no node 'no_such_node'"),
        "{p}"
    );
    let _ = std::fs::remove_dir_all(&s);
}
