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
        let file = opened(rows::decode(request)?)?;
        let found = checks::check_release(&file);
        Ok(format!(
            "{{\"holds\":{},\"findings\":{}}}",
            found.holds(),
            findings(&found)
        ))
    };
    answer().unwrap_or_else(|e| error_json(&e))
}

/// A file's rows as the application opens it: a file from before 1.0 is
/// upgraded first.
fn opened(tables: Vec<crate::model::Table>) -> Result<File, Error> {
    if format_of(&tables) == "1" {
        Ok(upgrade::from_format_1(
            tables,
            &upgrade::Upgrade {
                app: "the page",
                at: "",
            },
        )?
        .file)
    } else {
        File::from_tables(tables)
    }
}

/// Two files compared, block by block (`crate::compare`): the request is the
/// first file's rows, then the second's. The answer is `{"summary", "same",
/// "file": [...], "blocks": [{"uid", "id", "before", "after", "status",
/// "differences": [...]}], "history": [{"table", "only_first",
/// "only_second"}]}`, each difference in the words a person reads.
pub fn compare(request: &[u8]) -> String {
    let answer = || -> Result<String, Error> {
        let mut r = Reader::new(request);
        let a = opened(rows::decode(r.bytes()?)?)?;
        let b = opened(rows::decode(r.bytes()?)?)?;
        let c = crate::compare::compare(&a, &b);
        let says = |ds: &[crate::compare::Difference]| {
            format!(
                "[{}]",
                ds.iter()
                    .map(|d| json_str(&d.says()))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        let opt = |s: &Option<String>| s.as_deref().map_or("null".to_string(), json_str);
        Ok(format!(
            "{{\"summary\":{},\"same\":{},\"file\":{},\"blocks\":[{}],\"history\":[{}]}}",
            json_str(&c.summary()),
            c.same,
            says(&c.file),
            c.blocks
                .iter()
                .map(|b| format!(
                    "{{\"uid\":{},\"id\":{},\"before\":{},\"after\":{},\"status\":{},\"differences\":{}}}",
                    json_str(&b.uid),
                    json_str(b.id()),
                    opt(&b.before),
                    opt(&b.after),
                    json_str(b.status.name()),
                    says(&b.differences)
                ))
                .collect::<Vec<_>>()
                .join(","),
            c.history
                .iter()
                .map(|(t, x, y)| format!(
                    "{{\"table\":{},\"only_first\":{x},\"only_second\":{y}}}",
                    json_str(t)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ))
    };
    answer().unwrap_or_else(|e| error_json(&e))
}

/// A request for [`compare`], as the page writes it.
pub fn compare_request(first_rows: &[u8], second_rows: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for bytes in [first_rows, second_rows] {
        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(bytes);
    }
    out
}

/// The group folder's checks (`crate::folder`), the page's group checker said
/// by the library: the request is a group's file as its rows — a file from
/// before 1.0 as it is, or one upgraded from it. The answer is
/// `{"findings": [{level, where, msg, line}, ...]}`, or `{"error", "kind"}`.
pub fn check_folder(request: &[u8]) -> String {
    let answer = || -> Result<String, Error> {
        let folder = folder_of(rows::decode(request)?)?;
        let spec = crate::folder::Spec::carried()?;
        let found = crate::folder::check(&folder, &spec);
        Ok(format!(
            "{{\"findings\":[{}]}}",
            found
                .iter()
                .map(|f| format!(
                    "{{\"level\":{},\"where\":{},\"msg\":{},\"line\":{}}}",
                    json_str(f.level.name()),
                    json_str(&f.place),
                    json_str(&f.msg),
                    f.line
                ))
                .collect::<Vec<_>>()
                .join(",")
        ))
    };
    answer().unwrap_or_else(|e| error_json(&e))
}

/// The seal of a group's folder, the page's seal rules said by the library
/// (`crate::seal`): the request is a group's file's rows. The answer is
/// `{"scopes": [{"scope", "fingerprint"}], "reviews": [...], "blockers": [...]}`.
pub fn seal_state(request: &[u8]) -> String {
    let answer = || -> Result<String, Error> {
        let folder = folder_of(rows::decode(request)?)?;
        let s = crate::seal::state(&folder, &crate::folder::Spec::carried()?);
        let list = |items: Vec<String>| format!("[{}]", items.join(","));
        Ok(format!(
            "{{\"scopes\":{},\"reviews\":{},\"blockers\":{}}}",
            list(
                s.scopes
                    .iter()
                    .map(|(scope, fp)| format!(
                        "{{\"scope\":{},\"fingerprint\":{}}}",
                        json_str(scope),
                        json_str(fp)
                    ))
                    .collect()
            ),
            list(
                s.reviews
                    .iter()
                    .map(|r| format!(
                        "{{\"name\":{},\"scope\":{},\"version\":{},\"fingerprint\":{},\"date\":{},\"verdict\":{},\"note\":{},\"current\":{}}}",
                        json_str(&r.name),
                        json_str(&r.scope),
                        json_str(&r.version),
                        json_str(&r.fingerprint),
                        json_str(&r.date),
                        json_str(&r.verdict),
                        json_str(&r.note),
                        r.current
                    ))
                    .collect()
            ),
            list(s.blockers.iter().map(|b| json_str(b)).collect())
        ))
    };
    answer().unwrap_or_else(|e| error_json(&e))
}

/// A group's file as the folder its sign-offs were given for, whichever
/// format it is in.
fn folder_of(tables: Vec<crate::model::Table>) -> Result<crate::format_1::Folder, Error> {
    Ok(match format_of(&tables).as_str() {
        "1" => crate::format_1::Old::from_tables(tables)?.folder(),
        _ => upgrade::to_format_1(&File::from_tables(tables)?)?.folder(),
    })
}

/// The format a file's meta says it is in.
fn format_of(tables: &[crate::model::Table]) -> String {
    tables
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
        .unwrap_or_default()
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
