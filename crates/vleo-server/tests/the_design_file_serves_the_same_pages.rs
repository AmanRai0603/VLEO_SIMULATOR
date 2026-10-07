//! A team's tool reads the design from one file; a developer's reads the
//! folders. The two must serve the same design.
//!
//! The kit carries `design.vleo` in place of the node folders, and the daemon
//! reads it through the interface it reads the folders through. So one server
//! is started on the folders and one on the file, and every page a reader
//! opens is asked of both: each node's fragment and what its folder holds, the
//! index, the de-risking record, a node form and its check. Any difference is a
//! page a team would see and a developer would not.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn get(port: u16, path: &str) -> (u16, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).expect("the server did not accept");
    write!(
        s,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut raw = Vec::new();
    s.read_to_end(&mut raw).unwrap();
    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    (status, body.to_string())
}

#[test]
fn the_file_and_the_folders_serve_the_same_pages() {
    let scratch = std::env::temp_dir().join(format!("vleo-design-serve-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    let file = scratch.join("design.vleo");
    vleo_design::write(&root(), &file, &vleo_design::Stamp::default())
        .expect("the design file was not written");

    // One server on the folders, then one on the file. Each reads where the
    // design is from as it starts, so the variable is set between the two.
    std::env::remove_var("VLEO_DESIGN");
    let folders = vleo_server::serve(Some(root()), 18911, false, true).expect("no server");
    std::env::set_var("VLEO_DESIGN", &file);
    let from_file = vleo_server::serve(Some(root()), 18941, false, true).expect("no server");
    std::env::remove_var("VLEO_DESIGN");

    // Each says where it reads the design from: the checkout's design/ and
    // the design file are both the design's files.
    let (_, v) = get(folders, "/v1/version");
    assert!(v.contains("\"from\":\"file\""), "{v}");
    let (_, v) = get(from_file, "/v1/version");
    assert!(v.contains("\"from\":\"file\""), "{v}");

    let tree = vleo_sheet::load::load_all_from(
        &*vleo_design::source(&root()).expect("the design opens"),
        &root(),
    )
    .expect("the design does not load");
    let mut paths: Vec<String> = vec![
        "/v1/index".into(),
        "/v1/derisk".into(),
        "/v1/form/orbit_velocity".into(),
        "/v1/lesson-form/orbit_velocity".into(),
    ];
    for id in tree.sheets.keys() {
        paths.push(format!("/v1/fragment/{id}"));
        paths.push(format!("/v1/node/{id}"));
    }
    let mut differ = Vec::new();
    for p in &paths {
        let a = get(folders, p);
        let b = get(from_file, p);
        if a.0 != 200 {
            differ.push(format!("{p}: {} from the folders", a.0));
        } else if a != b {
            differ.push(format!("{p}: differs"));
        }
    }
    assert!(
        differ.is_empty(),
        "{} of {} pages differ:\n{}",
        differ.len(),
        paths.len(),
        differ
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
