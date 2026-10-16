//! *Run my code again* runs only when a person asks (docs/PLAN_F.md, F3 and
//! decision 9): a POST runs the node engineer's code and says what it found;
//! a GET runs nothing; a node whose code has no runner here says so; and a
//! node that does not exist is refused by name.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ask(port: u16, method: &str, path: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\n\
         Connection: close\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out
}

#[test]
fn the_code_runs_on_a_post_and_never_on_a_get() {
    let s = std::env::temp_dir().join(format!("vleo-rerun-route-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    std::fs::create_dir_all(&s).unwrap();
    std::env::set_var("VLEO_CASE", s.join("case.csv"));
    std::env::set_var("VLEO_RESULTS", s.join("results"));
    std::env::set_var("VLEO_LOG", s.join("log"));
    std::env::set_var("VLEO_DATA", s.join("data"));
    std::env::remove_var("VLEO_DRIVE");
    std::env::remove_var("VLEO_DESIGN");
    let port = vleo_server::serve(Some(root()), 19611, false, true).unwrap();

    // Solar's Kp scenarios keep their node engineer's code in Rust, which
    // nothing here runs: kept and read, and said so.
    let r = ask(port, "POST", "/v1/rerun/sw_kp_scenarios");
    assert!(r.contains("\"ok\":true"), "{r}");
    assert!(r.contains("\"verdict\":\"not-run\""), "{r}");
    assert!(r.contains("kept and read"), "{r}");

    let r = ask(port, "GET", "/v1/rerun/sw_kp_scenarios");
    assert!(r.starts_with("HTTP/1.1 404"), "a GET ran the code: {r}");

    let r = ask(port, "POST", "/v1/rerun/no_such_node");
    assert!(
        r.contains("\"ok\":false") && r.contains("no node 'no_such_node'"),
        "{r}"
    );
    let _ = std::fs::remove_dir_all(&s);
}
