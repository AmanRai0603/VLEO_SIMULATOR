//! The shipped reference data verifies, and the same data with its line
//! endings rewritten does not.
//!
//! The 0.1.1 Windows kit was built from a checkout that git had rewritten to
//! CRLF. Every bundle failed its hash, the daemon said only "no reference data"
//! and every row that reads a bundle refused. These tests hold both halves: the
//! bytes in the repository are the bytes the manifest hashed, and a rewrite of
//! them is refused rather than trusted — so the fix is in how the data is
//! carried (`.gitattributes`), never in loosening the check.

use std::fs;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Copy the shipped bundles into a fresh directory, rewriting every payload
/// file with `rewrite`.
fn copy_bundles(tag: &str, rewrite: impl Fn(&[u8]) -> Vec<u8>) -> PathBuf {
    let to = std::env::temp_dir().join(format!("vleo-data-eol-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&to);
    for bundle in fs::read_dir(repo().join("bundles")).unwrap().flatten() {
        if !bundle.path().is_dir() {
            continue;
        }
        for version in fs::read_dir(bundle.path()).unwrap().flatten() {
            let dest = to.join(bundle.file_name()).join(version.file_name());
            fs::create_dir_all(&dest).unwrap();
            for f in fs::read_dir(version.path()).unwrap().flatten() {
                let bytes = fs::read(f.path()).unwrap();
                let out = if f.file_name() == "manifest.toml" {
                    bytes
                } else {
                    rewrite(&bytes)
                };
                fs::write(dest.join(f.file_name()), out).unwrap();
            }
        }
    }
    to
}

fn crlf(b: &[u8]) -> Vec<u8> {
    let mut o = Vec::with_capacity(b.len() + b.len() / 40);
    for &c in b {
        if c == b'\n' {
            o.push(b'\r');
        }
        o.push(c);
    }
    o
}

#[test]
fn the_shipped_bundles_verify_as_they_are_stored() {
    let from = copy_bundles("as-is", |b| b.to_vec());
    let mut store = vleo_data::Store::open(&from.join("store"));
    let n = store
        .sync(&vleo_data::Source::Shipped(from.clone()))
        .expect("the bundles in the repository must verify");
    assert!(n >= 2, "expected both solar bundles, synced {n}");
    let _ = fs::remove_dir_all(&from);
}

#[test]
fn the_same_bundles_with_windows_line_endings_are_refused() {
    let from = copy_bundles("crlf", crlf);
    let mut store = vleo_data::Store::open(&from.join("store"));
    let err = store
        .sync(&vleo_data::Source::Shipped(from.clone()))
        .expect_err("a payload rewritten to CRLF must not verify");
    assert!(err.contains("failed verification"), "{err}");
    let _ = fs::remove_dir_all(&from);
}

#[test]
fn git_never_rewrites_the_reference_data() {
    let attrs = fs::read_to_string(repo().join(".gitattributes"))
        .expect(".gitattributes decides how git carries the reference data");
    assert!(
        attrs.lines().any(|l| l.trim() == "bundles/** -text"),
        "bundles/ must be marked -text, or a Windows checkout rewrites its bytes"
    );
}

#[test]
fn home_is_userprofile_where_there_is_no_home() {
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |k: &str| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| std::ffi::OsString::from(v))
        }
    };
    assert_eq!(
        vleo_data::home_from(env(&[("USERPROFILE", r"C:\Users\ana")])),
        Some(PathBuf::from(r"C:\Users\ana"))
    );
    assert_eq!(
        vleo_data::home_from(env(&[("HOME", "/home/ana"), ("USERPROFILE", "x")])),
        Some(PathBuf::from("/home/ana"))
    );
    assert_eq!(
        vleo_data::home_from(env(&[("HOME", ""), ("USERPROFILE", r"C:\Users\ana")])),
        Some(PathBuf::from(r"C:\Users\ana"))
    );
    assert_eq!(vleo_data::home_from(env(&[])), None);
}
