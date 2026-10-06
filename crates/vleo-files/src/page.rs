//! What the page asks of the library, and how it is answered.
//!
//! The page cannot link Rust; it calls a WebAssembly build of this library
//! (`crates/vleo-files-wasm`) with bytes, and reads JSON back. Everything the
//! page can ask is here, in the library itself, so the native tests run the
//! very function the page runs: a check that refuses a file installed refuses
//! it in the page, by the same code (docs/OPERATING_1_0.md, section 14).
//!
//! A request is length-prefixed fields (`crate::rows`): for checking a
//! release, the anchor's fingerprint as text, then the programme's file and
//! the release, each as its rows.

use crate::chain;
use crate::checks;
use crate::error::Error;
use crate::model::File;
use crate::rows::{self, Reader};
use crate::upgrade;

/// Text as a JSON string.
pub fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn list(items: &[String]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|s| json_str(s))
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// The answer to a request that could not be answered.
pub fn error_json(e: &Error) -> String {
    format!(
        "{{\"error\":{},\"kind\":{}}}",
        json_str(e.message()),
        json_str(&format!("{:?}", e.kind()))
    )
}

fn file(r: &mut Reader) -> Result<File, Error> {
    File::from_tables(rows::decode(r.bytes()?)?)
}

/// Check a release through the chain: the request is the anchor, the
/// programme's file and the release. The answer is
/// `{"holds": bool, "signed": [...], "refused": [...]}`, or
/// `{"error": ..., "kind": ...}` when the files themselves are refused.
pub fn check_release(request: &[u8]) -> String {
    let answer = || -> Result<String, Error> {
        let mut r = Reader::new(request);
        let anchor = r.str()?;
        let programme = file(&mut r)?;
        let release = file(&mut r)?;
        let registry = chain::open_programme(programme, &anchor)?;
        let checked = chain::check_release(&release, &registry)?;
        let found = checks::check_release(&release);
        Ok(format!(
            "{{\"holds\":{},\"signed\":{},\"refused\":{},\"findings\":{}}}",
            checked.holds() && found.holds(),
            list(&checked.signed),
            list(&checked.refused),
            findings(&found)
        ))
    };
    answer().unwrap_or_else(|e| error_json(&e))
}

fn findings(found: &checks::Findings) -> String {
    format!(
        "[{}]",
        found
            .0
            .iter()
            .map(|f| format!(
                "{{\"level\":{},\"place\":{},\"what\":{}}}",
                json_str(match f.level {
                    checks::Level::Error => "error",
                    checks::Level::Warning => "warning",
                }),
                json_str(&f.place),
                json_str(&f.what)
            ))
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// Check what one release holds (`crate::checks`): the request is its rows. A
/// file from before 1.0 is upgraded first, as the application opens it. The
/// answer is `{"holds": bool, "findings": [{level, place, what}, ...]}`, or
/// `{"error": ..., "kind": ...}`.
pub fn check_content(request: &[u8]) -> String {
    let answer = || -> Result<String, Error> {
        let tables = rows::decode(request)?;
        let format = tables
            .iter()
            .find(|t| t.name == "meta")
            .and_then(|t| {
                t.rows.iter().find_map(|r| match r.as_slice() {
                    [crate::model::Cell::Text(k), crate::model::Cell::Text(v)] if k == "format" => {
                        Some(v.clone())
                    }
                    _ => None,
                })
            })
            .unwrap_or_default();
        let file = if format == "1" {
            upgrade::from_format_1(
                tables,
                &upgrade::Upgrade {
                    app: "the page",
                    at: "",
                },
            )?
            .file
        } else {
            File::from_tables(tables)?
        };
        let found = checks::check_release(&file);
        Ok(format!(
            "{{\"holds\":{},\"findings\":{}}}",
            found.holds(),
            findings(&found)
        ))
    };
    answer().unwrap_or_else(|e| error_json(&e))
}

/// A request for [`check_release`], as the page writes it.
pub fn check_release_request(anchor: &str, programme: &File, release: &File) -> Vec<u8> {
    let mut out = Vec::new();
    let mut field = |bytes: &[u8]| {
        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(bytes);
    };
    field(anchor.as_bytes());
    field(&rows::encode(&programme.to_tables()));
    field(&rows::encode(&release.to_tables()));
    out
}

/// The sources the page's build of this library is made from, relative to the
/// repository: a change to any of them and the committed build is stale.
#[cfg(not(target_arch = "wasm32"))]
pub const WASM_SOURCES: &[&str] = &[
    "crates/vleo-files/Cargo.toml",
    "crates/vleo-files/src",
    "crates/vleo-files-wasm/Cargo.toml",
    "crates/vleo-files-wasm/Cargo.lock",
    "crates/vleo-files-wasm/src",
];

/// The fingerprint of [`WASM_SOURCES`] as they are under `root`: every file,
/// by path and SHA-256, in path order. `web/files.wasm.stamp` records the one
/// the committed build was made from, and a test holds the two together.
#[cfg(not(target_arch = "wasm32"))]
pub fn sources_fingerprint(root: &std::path::Path) -> Result<String, Error> {
    use crate::error::ErrorKind;
    use crate::keys::{hex, sha256};
    fn walk(root: &std::path::Path, rel: &str, out: &mut Vec<String>) -> Result<(), Error> {
        let p = root.join(rel);
        if p.is_dir() {
            let rd = std::fs::read_dir(&p)
                .map_err(|e| Error::new(ErrorKind::Io, format!("{rel}: {e}")))?;
            for e in rd {
                let e = e.map_err(|e| Error::new(ErrorKind::Io, format!("{rel}: {e}")))?;
                let name = e.file_name().to_string_lossy().into_owned();
                walk(root, &format!("{rel}/{name}"), out)?;
            }
        } else {
            out.push(rel.to_string());
        }
        Ok(())
    }
    let mut files = Vec::new();
    for s in WASM_SOURCES {
        walk(root, s, &mut files)?;
    }
    files.sort();
    let mut lines = Vec::new();
    for f in files {
        let bytes = std::fs::read(root.join(&f))
            .map_err(|e| Error::new(ErrorKind::Io, format!("{f}: {e}")))?;
        lines.push(format!("{f}\u{0}{}", hex(&sha256(&bytes))));
    }
    Ok(hex(&sha256(lines.join("\n").as_bytes())[..8]))
}
