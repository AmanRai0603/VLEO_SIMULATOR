//! A team's tool is a kit, without the repository; a developer's is a
//! checkout. The two must serve the same design.
//!
//! The kit carries the design's files, `design/`, and beside them only what
//! the daemon reads that is not the design: the web face, the reference data,
//! the manual, and what each node folder holds that is the code's and the
//! prior implementation's grid — never a sheet (`xtask kit`, `cmd_kit`). So one server is started on the checkout
//! and one on a folder laid out as the kit lays it out, and every page a
//! reader opens is asked of both: each node's fragment and what its folder
//! holds, the index, the de-risking record and a lesson form. Any difference
//! is a page a team would see and a developer would not.

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

fn copy(from: &Path, to: &Path, take: &dyn Fn(&Path) -> bool) {
    for e in std::fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let dest = to.join(p.file_name().unwrap());
        if p.is_dir() {
            copy(&p, &dest, take);
        } else if take(&p) {
            std::fs::create_dir_all(to).unwrap();
            std::fs::copy(&p, &dest).unwrap();
        }
    }
}

#[test]
fn a_kit_and_a_checkout_serve_the_same_pages() {
    let scratch = std::env::temp_dir().join(format!("vleo-kit-serve-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let kit = scratch.join("kit");
    std::fs::create_dir_all(&kit).unwrap();
    std::env::set_var("VLEO_CASE", scratch.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", scratch.join("results"));
    std::env::set_var("VLEO_LOG", scratch.join("log"));
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    std::env::remove_var("VLEO_DRIVE");
    std::env::remove_var("VLEO_DESIGN");

    // The kit, as `cmd_kit` lays it out. The design's files are read where
    // they are, named by VLEO_DESIGN, rather than copied: they are the same
    // bytes either way.
    let all = |_: &Path| true;
    for dir in ["web", "bundles", "matlab/reference"] {
        if root().join(dir).is_dir() {
            copy(&root().join(dir), &kit.join(dir), &all);
        }
    }
    std::fs::create_dir_all(kit.join("docs")).unwrap();
    std::fs::copy(
        root().join("docs/manual.toml"),
        kit.join("docs/manual.toml"),
    )
    .unwrap();
    let code = |p: &Path| {
        let s = p.to_string_lossy().replace('\\', "/");
        s.contains("/nodes/")
            && !["node.toml", "fixtures.toml"].contains(&p.file_name().unwrap().to_str().unwrap())
    };
    for c in std::fs::read_dir(root().join("crates")).unwrap() {
        let c = c.unwrap().path();
        let name = c.file_name().unwrap().to_string_lossy().into_owned();
        if name.starts_with("vleo-mod-") && c.join("nodes").is_dir() {
            copy(
                &c.join("nodes"),
                &kit.join("crates").join(&name).join("nodes"),
                &code,
            );
        }
    }

    // One server on the checkout, then one on the kit. Each reads where the
    // design is from as it starts, so the variable is set between the two.
    let checkout = vleo_server::serve(Some(root()), 18911, false, true).expect("no server");
    std::env::set_var("VLEO_DESIGN", root().join("design"));
    let from_kit = vleo_server::serve(Some(kit.clone()), 18941, false, true).expect("no server");
    std::env::remove_var("VLEO_DESIGN");

    // Each says it reads the design from the design's files, and which.
    let (_, a) = get(checkout, "/v1/version");
    let (_, b) = get(from_kit, "/v1/version");
    assert!(a.contains("\"from\":\"files\""), "{a}");
    assert!(b.contains("\"from\":\"files\""), "{b}");
    let print = |v: &str| {
        v.split("\"fingerprint\":\"")
            .nth(1)
            .map(|f| f[..64].to_string())
    };
    assert_eq!(print(&a), print(&b), "the two read different designs");

    let mut paths: Vec<String> = vec![
        "/v1/index".into(),
        "/v1/derisk".into(),
        "/v1/lesson-form/orbit_velocity".into(),
    ];
    for n in vleo_modules::nodes() {
        paths.push(format!("/v1/fragment/{}", n.id));
        paths.push(format!("/v1/node/{}", n.id));
    }
    let mut differ = Vec::new();
    for p in &paths {
        let a = get(checkout, p);
        let b = get(from_kit, p);
        if a.0 != 200 {
            differ.push(format!("{p}: {} from the checkout", a.0));
        } else if a != b {
            // Where the two first part, so a difference says what it is.
            let at =
                a.1.bytes()
                    .zip(b.1.bytes())
                    .position(|(x, y)| x != y)
                    .unwrap_or(a.1.len().min(b.1.len()));
            let near = |s: &str| {
                let lo = s.floor_char_boundary(at.saturating_sub(80));
                let hi = s.ceil_char_boundary((at + 80).min(s.len()));
                s[lo..hi].to_string()
            };
            differ.push(format!(
                "{p}: differs at byte {at}\n  checkout: {:?}\n  kit:      {:?}",
                near(&a.1),
                near(&b.1)
            ));
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
