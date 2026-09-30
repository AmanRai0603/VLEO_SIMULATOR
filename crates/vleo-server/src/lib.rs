//! `vleo-server` — the local engine, serving its own interface. The daemon
//! (`vleo-daemon`) starts it as a program; the Python package (`python -m vleo`)
//! starts it inside python.exe.
//!
//! # The one hard part, settled by design rather than discovered later
//!
//! A page on the internet speaking to a program on the machine is the most
//! likely thing to break this whole model, and it is not obvious. Three browser
//! rules stand between them:
//!
//! * **Cross-origin resource sharing** — a page may not read a response from
//!   another origin without permission.
//! * **Mixed content** — a secure page may not load an insecure resource.
//!   Loopback is treated as trustworthy, so this mostly does not bite; but that
//!   is a policy, not a guarantee.
//! * **Private network access** — a public page reaching a local address needs
//!   an extra preflight, and the rules are tightening. This is the one most
//!   likely to break later, in a browser update, with no change on this side.
//!
//! The boundary is **removed rather than negotiated**: the daemon serves the
//! interface itself, so the page and the engine share one origin and none of
//! the three rules applies. It also works with networking disabled entirely.
//!
//! Loopback by default. Exposing it to a network is an explicit, separate act,
//! which also avoids a firewall prompt on first run.

mod case;
mod http;
mod json;
mod pages;
mod runs;
mod saved;

#[allow(unused_imports)]
use case::*;
#[allow(unused_imports)]
use http::*;
#[allow(unused_imports)]
use pages::*;
#[allow(unused_imports)]
use runs::*;
#[allow(unused_imports)]
use saved::*;

use json::Json;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use vleo_bus::{Case, RunMode};
use vleo_core::graph::Kind;
use vleo_modules::{tables, Scratch, Vleo, GROUPS, NODES, RELATIONS, VARS};

/// Start the tool from the command line: `vleo-daemon [--open]`.
///
/// It opens the browser itself with `--open`, or when the program is named
/// `Start VLEO` — the Windows kit's name for it, so a double-click is the whole
/// of starting it and no script has to launch it.
pub fn main() {
    let port_pref: u16 = std::env::var("VLEO_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(7777);
    let named_to_open = std::env::current_exe()
        .ok()
        .and_then(|e| e.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .is_some_and(|stem| opens_by_name(&stem));
    let open = named_to_open || std::env::args().any(|a| a == "--open");
    if let Err(e) = serve(None, port_pref, open, false) {
        eprintln!("vleo: {e}");
        std::process::exit(1);
    }
}

/// Whether a program of this name opens the browser without being asked.
pub fn opens_by_name(stem: &str) -> bool {
    stem.eq_ignore_ascii_case("start vleo")
}

/// Serve the tool: the interface and the engine, from one origin on loopback.
///
/// `root` is where the web face and the tree are; `None` finds them as a
/// checkout or a kit would (see `repo_root`). The first free port from
/// `port_pref` is taken, and the one actually bound is returned. With
/// `background` the server runs on its own thread and this returns as soon as
/// it listens — how the Python package starts it; otherwise it serves forever.
pub fn serve(
    root: Option<PathBuf>,
    port_pref: u16,
    open: bool,
    background: bool,
) -> Result<u16, String> {
    vleo_data::crash::install("vleo", env!("CARGO_PKG_VERSION"));
    let root = root.unwrap_or_else(repo_root);
    let (data, data_versions, bundles, data_refused) = resolve_data(&root);

    // Try a range and record the port that actually bound. A daemon that fails
    // to start with a message nobody can act on is a support case that cannot
    // be answered without a screen share.
    let mut bound = None;
    for p in port_pref..port_pref.saturating_add(16) {
        if let Ok(l) = TcpListener::bind(("127.0.0.1", p)) {
            bound = Some((l, p));
            break;
        }
    }
    let Some((listener, port)) = bound else {
        return Err(format!(
            "nothing in {}..{} was free on 127.0.0.1. Set VLEO_PORT to choose another.",
            port_pref,
            port_pref.saturating_add(16)
        ));
    };

    println!("vleo {}", env!("CARGO_PKG_VERSION"));
    println!(
        "  kernel {}  graph {}  {} nodes",
        short(Vleo::kernel_hash()),
        short(Vleo::graph_hash()),
        NODES.len()
    );
    if data.is_empty() {
        println!("  \x1b[33mno reference data in the store — nodes that declare a bundle will refuse\x1b[0m");
        // Say why. A warning with no reason is the one a person cannot act on,
        // and the reason is usually a file that changed after it was shipped.
        if let Some(why) = &data_refused {
            println!("  \x1b[33mwhy: {why}\x1b[0m");
        }
    } else {
        println!("  data   {}", data_versions.join(" · "));
    }
    println!("  serving the interface and the engine from one origin:");
    println!("  \x1b[1mhttp://127.0.0.1:{port}\x1b[0m");
    println!("  loopback only. Exposing this to a network is a separate, explicit act.");
    // Opened by the tool, because only the tool knows which port it got — 7777
    // may be taken, and a script that guesses opens somebody else's page.
    if open {
        open_browser(&format!("http://127.0.0.1:{port}"));
    }

    let preview = read_preview(&root);
    if let Some(p) = &preview {
        println!(
            "  \x1b[33mPREVIEW build — not a release: {}\x1b[0m",
            p.chars().take(160).collect::<String>()
        );
    }
    let ctx = std::sync::Arc::new(Ctx {
        root,
        data,
        data_versions,
        bundles,
        port,
        preview,
    });
    if background {
        std::thread::spawn(move || accept(listener, ctx));
    } else {
        accept(listener, ctx);
    }
    Ok(port)
}

struct Ctx {
    root: PathBuf,
    data: Vec<String>,
    data_versions: Vec<String>,
    /// The verified bundles, by name: where each lives and which files its own
    /// manifest declares.
    ///
    /// Held so the reference data can be served to a face. Only what the
    /// manifest lists is reachable, and only from a bundle that verified — a
    /// face drawing the record must be drawing the same bytes the engine reads,
    /// or the picture and the answer are two different claims.
    bundles: BTreeMap<String, (PathBuf, Vec<String>)>,
    port: u16,
    /// What a preview build says about itself, as the JSON object it was
    /// written as; `None` for a release. See `read_preview`.
    preview: Option<String>,
}

/// A preview build's own description: `PREVIEW.json` beside the tool's files.
///
/// A PREVIEW IS BUILT FROM A FORM BRANCH, BEFORE ITS CHANGE IS APPROVED, and
/// the one thing it must never be is mistaken for a release. The preview
/// workflow writes this file — the branch, the commit, the build, the author,
/// the nodes it changes — and the page shows it as a banner on every view, with
/// the Approve button that saves the author's approval of this exact build. A
/// release has no such file. Anything that is not one JSON object is ignored,
/// so a damaged file cannot break the version endpoint.
fn read_preview(root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(root.join("PREVIEW.json")).ok()?;
    let t = text.trim();
    (t.starts_with('{') && t.ends_with('}') && !t.contains("</")).then(|| t.to_string())
}

fn short(h: u64) -> String {
    String::from_utf8(vleo_core::hash::short_hex(h).to_vec()).unwrap_or_default()
}

/// Where the tool's files are: the web face, the tree and its pages.
///
/// A developer runs this from a checkout; a team member runs it from an
/// unpacked kit (`xtask kit`) with no checkout at all. So the files are looked
/// for, in order: where `VLEO_ROOT` says, then upward from where it was started,
/// then upward from the binary itself — a kit keeps the binary beside them, and
/// a double-clicked binary starts wherever the desktop chose.
fn repo_root() -> PathBuf {
    let holds = |p: &Path| p.join("web").is_dir() && p.join("layers").is_dir();
    if let Ok(r) = std::env::var("VLEO_ROOT") {
        let r = PathBuf::from(r);
        if holds(&r) {
            return r;
        }
    }
    let starts = [
        std::env::current_dir().ok(),
        std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(Path::to_path_buf)),
    ];
    for start in starts.into_iter().flatten() {
        let mut p = start;
        loop {
            if holds(&p) {
                return p;
            }
            if !p.pop() {
                break;
            }
        }
    }
    std::env::current_dir().unwrap_or_default()
}

type BundleFiles = BTreeMap<String, (PathBuf, Vec<String>)>;

fn resolve_data(root: &Path) -> (Vec<String>, Vec<String>, BundleFiles, Option<String>) {
    let store_root = vleo_data::data_path().unwrap_or_else(|| root.join(".vleo/data"));
    let mut store = vleo_data::Store::open(&store_root);
    let mut refused = None;
    if store.load().is_err() || store.bundles.is_empty() {
        let shipped = root.join("bundles");
        if shipped.is_dir() {
            refused = store.sync(&vleo_data::Source::Shipped(shipped)).err();
        }
    }
    let files = store
        .bundles
        .values()
        .filter(|b| b.verified)
        .map(|b| {
            (
                b.manifest.name.clone(),
                (b.dir.clone(), b.manifest.files.clone()),
            )
        })
        .collect();
    (store.verified_names(), store.versions(), files, refused)
}

fn route(
    method: &str,
    path: &str,
    params: &str,
    ctx: &Ctx,
) -> (&'static str, &'static str, Vec<u8>) {
    match (method, path) {
        // A route that panics on purpose, present only when VLEO_TEST_PANIC is
        // set: how the crash handling is proved on a running copy (and in CI).
        ("GET", "/v1/__panic") if std::env::var_os("VLEO_TEST_PANIC").is_some() => {
            panic!("deliberate panic for the crash-handling check")
        }
        ("GET", "/") => file(ctx, "index.html", "text/html; charset=utf-8"),
        ("GET", "/app.css") => file(ctx, "app.css", "text/css; charset=utf-8"),
        // The shell is one module per concern. They are served individually
        // rather than bundled: a bundler is a build step between the source and
        // the thing that runs, and the first time they disagree the disagreement
        // is invisible.
        ("GET", p) if p.starts_with("/js/") => module(ctx, p.trim_start_matches("/js/")),
        ("GET", "/v1/version") => ok_json(version_json(ctx)),
        // The manual, with what is true of this running copy right now.
        ("GET", "/v1/manual") => manual_endpoint(ctx),
        ("GET", "/v1/index") => ok_json(index_json()),
        ("GET", "/v1/inputs") => ok_json(inputs_json()),
        // The saved case as a file to keep — in this tree's template, with the
        // record of any upgrade in it — or with `inputs=defaults` the blank
        // template, or with `backup=1` the file as it was before an upgrade.
        ("GET", "/v1/inputs.csv") => inputs_csv(params),
        ("POST", "/v1/inputs/check") => ok_json(inputs_check(params)),
        ("POST", "/v1/inputs") => ok_json(inputs_save(params)),
        ("POST", "/v1/inputs/reset") => ok_json(inputs_reset()),
        // A node's form: one self-contained HTML file to fill anywhere, and
        // the check of a filled one. The check writes nothing — a form is
        // applied by a developer at a terminal, with `xtask intake --apply`.
        ("GET", p) if p.starts_with("/v1/form/") => {
            form_file(ctx, p.trim_start_matches("/v1/form/"))
        }
        ("POST", "/v1/form/check") => ok_json(form_check(ctx, params)),
        // Saved results: what runs returned, with the inputs they ran on, kept
        // outside the repository. Viewing one runs nothing.
        ("GET", "/v1/derisk") => ok_json(derisk_json(ctx)),
        ("GET", "/v1/results") => ok_json(results_list()),
        ("GET", "/v1/result") => ok_json(result_json(params, ctx)),
        ("GET", "/v1/result.csv") => result_file(params, false),
        ("GET", "/v1/result.html") => result_file(params, true),
        ("GET", "/v1/result.vleo") => result_share(params),
        ("POST", "/v1/results/pin") => ok_json(result_pin(params)),
        ("POST", "/v1/results/save") => ok_json(result_save(params, ctx)),
        ("POST", "/v1/results/upload") => ok_json(result_upload(params)),
        ("POST", "/v1/results/delete") => ok_json(result_delete(params)),
        ("POST", "/v1/results/as-case") => ok_json(result_as_case(params)),
        ("GET", p) if p.starts_with("/v1/fragment/") => {
            let id = p.trim_start_matches("/v1/fragment/");
            fragment(ctx, id)
        }
        ("GET", p) if p.starts_with("/v1/node/") => {
            let id = p.trim_start_matches("/v1/node/");
            node_endpoint(ctx, id)
        }
        ("POST", "/v1/run") | ("GET", "/v1/run") => ok_json(run_json(params, ctx)),
        ("GET", "/v1/sweep") => ok_json(sweep_json(params, ctx)),
        ("GET", "/v1/probe") => ok_json(probe_json(params)),
        ("GET", "/v1/levers") => ok_json(levers_json(params, ctx)),
        ("GET", "/v1/branches") => ok_json(branches_json(params)),
        // The reference data itself, so a face can draw the record rather than
        // only the answers computed from it.
        ("GET", p) if p.starts_with("/v1/bundle/") => {
            bundle_file(ctx, p.trim_start_matches("/v1/bundle/"))
        }
        // ANOTHER IMPLEMENTATION'S ANSWERS, which are evidence and not data.
        //
        // Deliberately not folded into a bundle. A bundle is verified reference
        // data with a provenance and a licence — what was OBSERVED — and the
        // legacy tool's saved output is neither: it is a record of what a
        // different program computed, kept so this one can be checked against
        // it. Serving it under its own name keeps the two apart in the one place
        // a reader might otherwise conflate them, which is the face.
        ("GET", p) if p.starts_with("/v1/parity/") => {
            parity_file(ctx, p.trim_start_matches("/v1/parity/"))
        }
        _ => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            b"no such endpoint. Any endpoint not on the published list is a request to compute something on the server, and that request is answered by asking whether the node should be restricted.".to_vec(),
        ),
    }
}

fn ok_json(s: String) -> (&'static str, &'static str, Vec<u8>) {
    ("200 OK", "application/json; charset=utf-8", s.into_bytes())
}

/// Now, as a result records it: UTC, to the second (vleo_data::clock).
fn now_utc() -> String {
    vleo_data::clock::now_utc()
}

fn failed(message: &str) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", false);
    j.str_field("message", message);
    j.raw("}");
    j.0
}

#[cfg(test)]
mod start_by_name {
    use super::opens_by_name;

    #[test]
    fn only_the_kits_start_name_opens_the_browser_by_itself() {
        assert!(opens_by_name("Start VLEO"));
        assert!(opens_by_name("start vleo"));
        assert!(!opens_by_name("vleo-daemon"));
        assert!(!opens_by_name("vleo"));
        assert!(!opens_by_name("Start VLEO (1)"));
    }
}
