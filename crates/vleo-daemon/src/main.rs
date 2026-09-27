//! `vleo-daemon` — the local engine, serving its own interface.
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

mod json;

use json::Json;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use vleo_bus::{Case, RunMode};
use vleo_core::graph::Kind;
use vleo_modules::{tables, Scratch, Vleo, GROUPS, NODES, RELATIONS, VARS};

fn main() {
    let port_pref: u16 = std::env::var("VLEO_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(7777);
    let root = repo_root();
    let (data, data_versions, bundles) = resolve_data(&root);

    // Try a range and record the port that actually bound. A daemon that fails
    // to start with a message nobody can act on is a support case that cannot
    // be answered without a screen share.
    let mut bound = None;
    for p in port_pref..port_pref + 16 {
        if let Ok(l) = TcpListener::bind(("127.0.0.1", p)) {
            bound = Some((l, p));
            break;
        }
    }
    let (listener, port) = match bound {
        Some(x) => x,
        None => {
            eprintln!(
                "vleo-daemon: nothing in {}..{} was free on 127.0.0.1. Set VLEO_PORT to choose another.",
                port_pref,
                port_pref + 16
            );
            std::process::exit(1);
        }
    };

    println!("vleo-daemon {}", env!("CARGO_PKG_VERSION"));
    println!(
        "  kernel {}  graph {}  {} nodes",
        short(Vleo::kernel_hash()),
        short(Vleo::graph_hash()),
        NODES.len()
    );
    if data.is_empty() {
        println!("  \x1b[33mno reference data in the store — nodes that declare a bundle will refuse\x1b[0m");
    } else {
        println!("  data   {}", data_versions.join(" · "));
    }
    println!("  serving the interface and the engine from one origin:");
    println!("  \x1b[1mhttp://127.0.0.1:{port}\x1b[0m");
    println!("  loopback only. Exposing this to a network is a separate, explicit act.");

    let ctx = std::sync::Arc::new(Ctx {
        root,
        data,
        data_versions,
        bundles,
        port,
    });
    // ONE CONNECTION NEVER HOLDS UP ANOTHER, AND ONE REQUEST IS HANDLED AT A TIME.
    //
    // This loop used to read each connection itself, in turn. A browser opens
    // connections it may never use — it keeps a spare one warm so the next
    // request does not wait for a handshake — and the loop sat reading one of
    // those, with no timeout, while every real request queued behind it. The
    // page looked hung until the browser happened to close the spare. The more
    // requests a page made at once, the more spares it opened, so a busier page
    // made it worse; the manual's browser walk caught it as page loads that
    // timed out at random.
    //
    // So each connection is READ on its own thread, with a timeout, and an idle
    // one simply ends. HANDLING stays one at a time behind the lock: the form
    // writes sheets and regenerates folders, and two saves interleaving would
    // be a race this tool has never had to think about and should not start to.
    let one_at_a_time = std::sync::Arc::new(std::sync::Mutex::new(()));
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let ctx = ctx.clone();
                let gate = one_at_a_time.clone();
                std::thread::spawn(move || {
                    if let Err(e) = serve(s, &ctx, &gate) {
                        eprintln!("vleo-daemon: {e}");
                    }
                });
            }
            Err(e) => eprintln!("vleo-daemon: accept: {e}"),
        }
    }
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
}

fn short(h: u64) -> String {
    String::from_utf8(vleo_core::hash::short_hex(h).to_vec()).unwrap_or_default()
}

fn repo_root() -> PathBuf {
    let mut p = std::env::current_dir().unwrap_or_default();
    loop {
        if p.join("web").is_dir() && p.join("layers").is_dir() {
            return p;
        }
        if !p.pop() {
            return std::env::current_dir().unwrap_or_default();
        }
    }
}

type BundleFiles = BTreeMap<String, (PathBuf, Vec<String>)>;

fn resolve_data(root: &Path) -> (Vec<String>, Vec<String>, BundleFiles) {
    let store_root = std::env::var("VLEO_DATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".vleo").join("data"))
                .unwrap_or_else(|_| root.join(".vleo/data"))
        });
    let mut store = vleo_data::Store::open(&store_root);
    if store.load().is_err() || store.bundles.is_empty() {
        let shipped = root.join("bundles");
        if shipped.is_dir() {
            let _ = store.sync(&vleo_data::Source::Shipped(shipped));
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
    (store.verified_names(), store.versions(), files)
}

/// One file of one verified bundle, as it is on disk.
///
/// Serving the reference data rather than a shaped summary of it is deliberate.
/// The bundle's own index says "open any of them in a spreadsheet; nothing here
/// needs a library", and a face that reads the same CSV the engine reads cannot
/// drift from it. A JSON projection here would be a second description of the
/// data, and the first time it disagreed with the file the disagreement would be
/// invisible.
///
/// Two refusals, and neither is about secrecy — this is public reference data.
/// A name not in the store, or a file its manifest does not declare, is a
/// request for something this repository cannot vouch for, and the answer is to
/// say so rather than to read whatever is at that path.
fn bundle_file(ctx: &Ctx, rest: &str) -> (&'static str, &'static str, Vec<u8>) {
    let mut it = rest.splitn(2, '/');
    let name = it.next().unwrap_or("");
    let file = it.next().unwrap_or("");
    let Some((dir, files)) = ctx.bundles.get(name) else {
        return (
            "404 Not Found",
            "text/plain; charset=utf-8",
            format!(
                "no verified bundle called '{name}'. Verified bundles here: {}",
                ctx.data.join(", ")
            )
            .into_bytes(),
        );
    };
    if !files.iter().any(|f| f == file) {
        return (
            "404 Not Found",
            "text/plain; charset=utf-8",
            format!(
                "'{file}' is not a file '{name}' declares. Its manifest lists: {}",
                files.join(", ")
            )
            .into_bytes(),
        );
    }
    match std::fs::read(dir.join(file)) {
        Ok(b) => ("200 OK", "text/csv; charset=utf-8", b),
        Err(e) => (
            "500 Internal Server Error",
            "text/plain; charset=utf-8",
            format!("{file} is declared by {name} and could not be read: {e}").into_bytes(),
        ),
    }
}

// ---------------------------------------------------------------------------

fn serve(mut stream: TcpStream, ctx: &Ctx, gate: &std::sync::Mutex<()>) -> std::io::Result<()> {
    // Long enough for any real request to arrive; a connection that has sent
    // nothing by then is a spare the browser kept warm, and it is let go.
    stream.set_read_timeout(Some(std::time::Duration::from_secs(20)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    match reader.read_line(&mut request) {
        Ok(0) => return Ok(()),
        Ok(_) => {}
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ) =>
        {
            return Ok(())
        }
        Err(e) => return Err(e),
    }
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("/").to_string();

    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let l = line.trim_end();
        if l.is_empty() {
            break;
        }
        if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") {
            length = v.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; length];
    if length > 0 {
        reader.read_exact(&mut body)?;
    }
    let body = String::from_utf8_lossy(&body).to_string();

    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.clone(), String::new()),
    };
    let params = if method == "POST" && !body.is_empty() {
        body
    } else {
        query
    };

    let (status, ctype, payload) = {
        // A poisoned lock means a handler panicked on an earlier request; that
        // request already failed, and refusing every later one would turn one
        // bad request into a dead tool.
        let _one = gate.lock().unwrap_or_else(|p| p.into_inner());
        route(&method, &path, &params, ctx)
    };
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()
}

fn route(
    method: &str,
    path: &str,
    params: &str,
    ctx: &Ctx,
) -> (&'static str, &'static str, Vec<u8>) {
    match (method, path) {
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
        ("GET", "/v1/result") => ok_json(result_json(params)),
        ("GET", "/v1/result.csv") => result_file(params, false),
        ("GET", "/v1/result.html") => result_file(params, true),
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

fn file(ctx: &Ctx, name: &str, ctype: &'static str) -> (&'static str, &'static str, Vec<u8>) {
    match std::fs::read(ctx.root.join("web").join(name)) {
        Ok(b) => ("200 OK", ctype, b),
        Err(_) => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            format!("web/{name} is not on disk").into_bytes(),
        ),
    }
}

/// One parity file from `matlab/reference`, by name.
///
/// Same traversal guard as `module`: anything that is not a plain file name with
/// a `.csv` suffix is refused before it reaches the filesystem. The directory is
/// fixed here rather than taken from the request, so there is no path to
/// construct and therefore no path to escape.
fn parity_file(ctx: &Ctx, name: &str) -> (&'static str, &'static str, Vec<u8>) {
    let stem = name.strip_suffix(".csv").unwrap_or("");
    let ok = !stem.is_empty()
        && stem
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !ok {
        return (
            "400 Bad Request",
            "text/plain; charset=utf-8",
            b"a parity file is a plain .csv name under matlab/reference".to_vec(),
        );
    }
    match std::fs::read(ctx.root.join("matlab").join("reference").join(name)) {
        Ok(b) => ("200 OK", "text/csv; charset=utf-8", b),
        Err(_) => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            format!("matlab/reference/{name} is not on disk").into_bytes(),
        ),
    }
}

/// One shell module, by name. Anything that is not a plain file name under
/// `web/js` is refused before it reaches the filesystem.
fn module(ctx: &Ctx, name: &str) -> (&'static str, &'static str, Vec<u8>) {
    let stem = name.strip_suffix(".js").unwrap_or("");
    let ok = !stem.is_empty()
        && stem
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !ok {
        return (
            "400 Bad Request",
            "text/plain; charset=utf-8",
            b"not a module name".to_vec(),
        );
    }
    match std::fs::read(ctx.root.join("web").join("js").join(name)) {
        Ok(b) => ("200 OK", "application/javascript; charset=utf-8", b),
        Err(_) => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            format!("web/js/{name} is not on disk").into_bytes(),
        ),
    }
}

/// The manual the page shows about the tool, and what is true of THIS copy.
///
/// The file says how things work. Only the running copy knows where its saved
/// case lives, how far that case is from the defaults, and how many rows there
/// are — and a manual that wrote any of those down would be wrong the day
/// after. So they are added here, from the same functions that decide them,
/// with the node form's own tables for what a form asks and what it cannot
/// change.
fn manual_endpoint(ctx: &Ctx) -> (&'static str, &'static str, Vec<u8>) {
    const JSON: &str = "application/json; charset=utf-8";
    let m = match vleo_sheet::manual::load(&ctx.root) {
        Ok(m) => m,
        // A manual that does not load is reported, not replaced by an empty
        // one: an empty manual reads as "there is nothing to know".
        Err(e) => {
            return (
                "500 Internal Server Error",
                JSON,
                format!("{{\"ok\":false,\"message\":{}}}", json::string(&e)).into_bytes(),
            )
        }
    };
    use vleo_sheet::form;
    let case = saved_case();
    let (rows, published, seeded) = match vleo_sheet::load::load_all(&ctx.root) {
        Ok(t) => {
            let n = t.sheets.len();
            let s = t.sheets.values().filter(|s| s.is_seeded()).count();
            let p = t.sheets.values().filter(|s| s.state == "published").count();
            (n, p, s)
        }
        Err(_) => (0, 0, 0),
    };
    let fields: Vec<String> = form::FIELDS
        .iter()
        .filter(|f| f.asked)
        .map(|f| {
            format!(
                "{{\"field\":{},\"group\":{},\"ask\":{},\"why\":{},\"shape\":{},\"blocks\":{}}}",
                json::string(f.field),
                json::string(f.group),
                json::string(f.ask),
                json::string(f.why),
                json::string(f.shape.name()),
                f.blocks
            )
        })
        .collect();
    let arrays: Vec<String> = form::ARRAYS
        .iter()
        .map(|a| {
            format!(
                "{{\"name\":{},\"label\":{},\"why\":{},\"end_only\":{},\"keys\":[{}]}}",
                json::string(a.name),
                json::string(a.label),
                json::string(a.why),
                a.blocks == form::Blocks::EndOnly,
                a.columns
                    .iter()
                    .filter(|c| !c.managed)
                    .map(|c| json::string(c.key))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect();
    let locked: Vec<String> = form::LOCKED
        .iter()
        .map(|k| {
            format!(
                "{{\"field\":{},\"why\":{}}}",
                json::string(k),
                json::string(form::structural(k).unwrap_or(""))
            )
        })
        .collect();
    let body = format!(
        "{{\"ok\":true,\"manual\":{},\n\"live\":{{\"case_path\":{},\"case_stored\":{},\
         \"case_changed\":{},\"port\":{},\"rows\":{},\"published\":{},\
         \"seeded\":{},\"fields\":[{}],\"arrays\":[{}],\"locked\":[{}]}}}}",
        vleo_sheet::manual::json(&m),
        json::string(&case_path().display().to_string()),
        case.stored,
        case.reading.changed,
        ctx.port,
        rows,
        published,
        seeded,
        fields.join(","),
        arrays.join(","),
        locked.join(",")
    );
    ("200 OK", JSON, body.into_bytes())
}

/// What one node's folder actually holds.
///
/// The architecture view claims a node is one folder with a fixed set of
/// artefacts, seven of them generated. A claim about the layout that the page
/// asserts from memory is a claim that goes stale the first time the layout
/// changes, so it is read off the disk instead.
fn node_endpoint(ctx: &Ctx, id: &str) -> (&'static str, &'static str, Vec<u8>) {
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return (
            "400 Bad Request",
            "text/plain; charset=utf-8",
            b"not a node identifier".to_vec(),
        );
    }
    let i = match Vleo::find(id) {
        Some(i) => i,
        None => {
            return (
                "404 Not Found",
                "application/json; charset=utf-8",
                b"{\"ok\":false,\"message\":\"no such node\"}".to_vec(),
            )
        }
    };
    let def = &NODES[i as usize];
    let dir = ctx.root.join(def.folder);

    // The eight files, and who writes each. This list is the template: it is
    // the same eight for every one of the 1333 folders, which is what makes
    // adding the next node a copy rather than a decision.
    const ARTEFACTS: &[(&str, &str, &str)] = &[
        (
            "node.toml",
            "by hand",
            "the sheet — the only file here written by hand",
        ),
        (
            "fixtures.toml",
            "by hand",
            "known-good values, and where each came from",
        ),
        (
            "model.rs",
            "generated",
            "the whole file, with one numbered HOLE per algorithm step",
        ),
        (
            "contract.rs",
            "generated",
            "the untyped adapter the bus calls",
        ),
        ("mod.rs", "generated", "the module wiring"),
        ("evidence.rs", "generated", "the fixtures, as tests"),
        ("page.html", "generated", "the tabs a reader opens"),
        (
            "meta.json",
            "generated",
            "state and hashes, written by the gate",
        ),
    ];

    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("id", def.id);
    j.str_field("folder", def.folder);
    j.str_field("subsystem", def.subsystem);
    j.str_field("state", def.state.name());
    j.str_field("sheet_hash", &short(def.sheet_hash));
    j.str_field("impl_hash", &short(def.impl_hash));
    j.num_field("steps", def.steps.len() as f64);
    j.num_field("inputs", def.inputs.len() as f64);
    j.num_field("outputs", def.outputs.len() as f64);
    j.num_field("fixtures", def.fixtures.len() as f64);
    j.key("artefacts").open_arr();
    for (n, (name, who, what)) in ARTEFACTS.iter().enumerate() {
        if n > 0 {
            j.raw(",");
        }
        let meta = std::fs::metadata(dir.join(name));
        j.raw("{");
        j.str_field("name", name);
        j.str_field("written", who);
        j.str_field("what", what);
        j.bool_field("present", meta.is_ok());
        j.num_field("bytes", meta.map(|m| m.len() as f64).unwrap_or(0.0));
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    ok_json(j.0)
}

/// One fragment per node, fetched when it is opened.
///
/// First paint is the shell and the index and does not get slower as the tree
/// fills; opening a node costs one fragment. That is the whole loading
/// strategy, and it is a consequence of the folder layout rather than a
/// separate design.
fn fragment(ctx: &Ctx, id: &str) -> (&'static str, &'static str, Vec<u8>) {
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return (
            "400 Bad Request",
            "text/plain; charset=utf-8",
            b"not a node identifier".to_vec(),
        );
    }
    let i = match Vleo::find(id) {
        Some(i) => i,
        None => {
            return (
                "404 Not Found",
                "text/html; charset=utf-8",
                b"<p>no such node</p>".to_vec(),
            )
        }
    };
    // The path is carried on the node, not rebuilt from the id. Rebuilding it
    // is a second implementation of the layout rule, and it fails on the first
    // row whose folder is not its id minus a prefix — as a 404 that nobody
    // attributes to a layout change.
    let def = &NODES[i as usize];
    let p = ctx.root.join(def.folder).join("page.html");
    match std::fs::read(&p) {
        Ok(b) => ("200 OK", "text/html; charset=utf-8", b),
        Err(_) => (
            "404 Not Found",
            "text/html; charset=utf-8",
            format!("<p class=\"empty\">No fragment for <code>{id}</code>. Run <code>cargo xtask docs</code>.</p>").into_bytes(),
        ),
    }
}

fn version_json(ctx: &Ctx) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.str_field("kernel", &short(Vleo::kernel_hash()));
    j.str_field("graph", &short(Vleo::graph_hash()));
    j.str_field("version", env!("CARGO_PKG_VERSION"));
    j.str_field("endpoint", "local-daemon");
    j.num_field("port", ctx.port as f64);
    j.num_field("nodes", NODES.len() as f64);
    j.key("data").open_arr();
    for (i, d) in ctx.data_versions.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.push_string(d);
    }
    j.close_arr();
    j.raw("}");
    j.0
}

/// The index: every row, its state, and all three graphs.
///
/// Emitted from the same tables the engine walks, so the picture and the
/// execution cannot diverge. There is no version of this where the page and the
/// code disagree.
/// Which rows reach a KPI closure, and how many rows read each one.
///
/// The tree's stated purpose is twelve promises to a customer; every other row
/// exists to move one of them. So "does this number reach a KPI" is the
/// end-to-end question, and it is not the same as "does this row answer" — a
/// subsystem can answer on every row it has and be wired to nothing.
///
/// Walked BACKWARDS from the KPIs, once. The forward version — ask each row
/// whether it can reach one, and memoise — has to seed `false` before recursing
/// so a declared cycle terminates, and then memoises that provisional `false`
/// for any row whose real answer arrived later by another edge. It reports a
/// row inside a cycle as unread when it is read, which is the one direction
/// this must never be wrong in. `xtask reach` carries the same walk and the
/// tests that pin it.
fn reach_and_readers() -> (Vec<bool>, Vec<usize>) {
    let n = NODES.len();
    let mut parents: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut readers = vec![0usize; n];
    for (i, d) in NODES.iter().enumerate() {
        for &v in d.inputs {
            let p = VARS[v as usize].producer as usize;
            if p != i {
                parents[i].push(p);
                readers[p] += 1;
            }
        }
    }
    let mut reach = vec![false; n];
    let mut stack: Vec<usize> = (0..n).filter(|&i| NODES[i].kind == Kind::Kpi).collect();
    while let Some(i) = stack.pop() {
        if reach[i] {
            continue;
        }
        reach[i] = true;
        stack.extend(parents[i].iter().copied());
    }
    (reach, readers)
}

fn index_json() -> String {
    let mut j = Json::new();
    let (reach, readers) = reach_and_readers();
    j.raw("{");
    j.num_field("nodes", NODES.len() as f64);
    j.key("rows").open_arr();
    for (i, d) in NODES.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        let v = &VARS[i];
        j.raw("{");
        j.num_field("i", i as f64);
        j.str_field("id", d.id);
        j.str_field("label", d.label);
        j.str_field("parent", d.parent);
        j.num_field("layer", d.layer as f64);
        j.num_field("order", d.order as f64);
        j.str_field("crosses", d.crosses_to);
        j.str_field("sub", d.subsystem);
        j.str_field("kind", d.kind.name());
        j.str_field("state", d.state.name());
        // WHETHER IT ANSWERS, AND WHY NOT.
        //
        // `state` says how far through its life the row is; these say whether
        // the design has made this number. A function whose relation the sheet
        // never derives is refused by the resolver, so a face that draws it
        // like any other row is offering a reader a control that cannot work
        // and a blank where a number should be, with nothing saying which.
        //
        // Both are sent rather than one combined flag, because the face has to
        // say what would fix it: `fn` decides whether the rule applies at all
        // — an input is defined by carrying a default — and `derived` is the
        // thing that is missing.
        j.bool_field("fn", d.is_function());
        j.bool_field("derived", d.derived);
        // WHERE THE ANSWER GOES, which is not the same question as whether
        // there is one. A row can answer perfectly and feed nothing the tool
        // exists to report, and a face that draws the two identically tells a
        // reader their subsystem is finished when it is wired to nothing.
        j.bool_field("kpi_reach", reach[i]);
        j.num_field("readby", readers[i] as f64);
        // Who read the relation against its source. Not the same claim, and it
        // does not silence the row — it is what the mathematics factor of the
        // credibility vector is scored on. Empty is the normal state.
        j.str_field("by", d.relation_by);
        j.str_field("owner", d.owner);
        j.str_field("tier", d.tier.name());
        j.str_field("unit", v.unit.symbol());
        j.str_field("symbol", v.symbol);
        j.str_field("question", d.question);
        j.num_field("lo", v.limit.lower);
        j.num_field("hi", v.limit.upper);
        // THE LIMITS ABOVE ARE SI, AND THE UNIT BESIDE THEM IS NOT.
        //
        // Mission duration is `yr` with a lower bound of 15778800 — five
        // years is 157788000 seconds, and a face that puts the bound straight
        // into a field beside the word "yr" offers a reader a range of fifteen
        // million years. The factor is what makes the two agree, and it is the
        // same `si_factor()` the levers endpoint already sends for exactly this
        // reason. Any face that lets a person type a value needs it, so it is
        // here rather than fetched per row from somewhere else.
        j.num_field("factor", v.unit.si_factor());
        // A bound without its reason is a bound the next person deletes when it
        // is inconvenient. The face shows these when a typed value is refused.
        j.str_field("why_lo", v.limit.reason_lower);
        j.str_field("why_hi", v.limit.reason_upper);
        j.num_field("fixtures", d.fixtures.len() as f64);
        // THE PRODUCING NODE, NOT THE VARIABLE.
        //
        // `d.inputs` holds VARIABLE indices, and the face reads this array as
        // ROW indices — `S.consumers[p].push(r.i)` in state.js. That was the
        // same number while every row published exactly one variable. It stopped
        // being one when a row's answer became a SET: a member's variable index
        // sits past the end of the row list, so the face indexed off the end of
        // an array, threw inside ingest(), and drew an empty matrix. Nothing in
        // the kernel noticed, because nothing in the kernel was wrong.
        //
        // Mapped here rather than in the face because the face's array is
        // documented as node-to-node coupling; which MEMBER was read is a
        // question the node page answers from the contract, not the index.
        j.key("in").open_arr();
        for (k, x) in d.inputs.iter().enumerate() {
            if k > 0 {
                j.raw(",");
            }
            j.raw(&VARS[*x as usize].producer.to_string());
        }
        j.close_arr();
        j.key("kpi").open_arr();
        for (k, x) in d.contributes.iter().enumerate() {
            if k > 0 {
                j.raw(",");
            }
            j.push_string(x);
        }
        j.close_arr();
        j.close_obj();
    }
    j.close_arr();
    j.key("groups").open_arr();
    for (i, g) in GROUPS.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("id", g.id);
        j.str_field("label", g.label);
        j.str_field("parent", g.parent);
        j.str_field("owner", g.owner);
        j.num_field("layer", g.layer as f64);
        j.num_field("order", g.order as f64);
        j.bool_field("box", g.is_box);
        j.str_field("tone", g.tone);
        j.key("cases").open_arr();
        for (k, c) in g.cases.iter().enumerate() {
            if k > 0 {
                j.raw(",");
            }
            j.push_string(c);
        }
        j.close_arr();
        j.close_obj();
    }
    j.close_arr();
    j.key("relations").open_arr();
    for (i, (a, b, why)) in RELATIONS.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("from", a);
        j.str_field("to", b);
        j.str_field("why", why);
        j.close_obj();
    }
    j.close_arr();
    // Which crate holds how many folders, read off the carried paths rather
    // than asserted. The isolation rule is a manifest line, so the count of it
    // should be a fact too.
    j.key("crates").open_arr();
    {
        let mut seen: Vec<(&str, usize)> = Vec::new();
        for d in NODES.iter() {
            let name = d.folder.split('/').nth(1).unwrap_or("");
            match seen.iter_mut().find(|(n, _)| *n == name) {
                Some((_, c)) => *c += 1,
                None => seen.push((name, 1)),
            }
        }
        seen.sort_by(|a, b| a.0.cmp(b.0));
        for (i, (name, count)) in seen.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("name", name);
            j.num_field("nodes", *count as f64);
            j.close_obj();
        }
    }
    j.close_arr();
    j.key("cases").open_arr();
    for (i, c) in tables::CASES.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("id", c.id);
        j.str_field("label", c.label);
        j.str_field("note", c.note);
        j.num_field("cycles", c.cycles.len() as f64);
        j.key("supply").open_arr();
        for (k, (v, val)) in c.supply.iter().enumerate() {
            if k > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("id", VARS[*v as usize].id);
            j.num_field("value", *val);
            j.close_obj();
        }
        j.close_arr();
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}

// ---------------------------------------------------------------------------

fn param<'a>(params: &'a str, key: &str) -> Option<&'a str> {
    params.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        if k == key {
            Some(v)
        } else {
            None
        }
    })
}

fn decode(s: &str) -> String {
    // BYTES FIRST, TEXT AFTER. Each %xx is one byte of UTF-8, and a character
    // like ° or — is two or three of them; pushing each byte as a character
    // turned "°" into two unrelated letters, which a unit check then refused.
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < b.len() => {
                out.push(u8::from_str_radix(&s[i + 1..i + 3], 16).unwrap_or(b'?'));
                i += 3;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ---------------------------------------------------------------------------
// the saved case
//
// The inputs a person saved, stored by the application and never in the
// repository: beside the reference data, in ~/.vleo/case/inputs.csv, or
// wherever VLEO_CASE points. Stored as the same CSV a person downloads, so the
// file on disk is one they can open, read and keep.

fn case_path() -> PathBuf {
    vleo_data::case_path()
}

/// The saved case, read and checked against the tree as it is now.
///
/// Read on every request rather than held, because it is a small file and a
/// person may replace it from another tab. CARRIED OVER on the first read after
/// the tool changes under it: a case written for another set of inputs is
/// copied aside and written again in this tree's template, with every value
/// that could not be carried set aside by name — see `inputs::saved::load`. So
/// a stored case always applies whole, and what an update did to it is shown
/// on the Inputs page and counted on every run until the case is next saved.
fn saved_case() -> vleo_modules::inputs::saved::Saved {
    vleo_modules::inputs::saved::load(&case_path())
}

fn inputs_csv(params: &str) -> (&'static str, &'static str, Vec<u8>) {
    const CSV: &str = "text/csv; charset=utf-8";
    if param(params, "inputs") == Some("defaults") {
        return ("200 OK", CSV, vleo_modules::inputs::csv(&[]).into_bytes());
    }
    let saved = saved_case();
    if param(params, "backup") == Some("1") {
        // Only the copy the upgrade itself recorded, and only beside the case:
        // a path read out of a file is never a path this route will open
        // anywhere else.
        let dir = case_path().parent().map(|d| d.to_path_buf());
        let kept = saved
            .reading
            .upgrade
            .as_ref()
            .and_then(|u| u.backup.as_ref())
            .map(PathBuf::from)
            .filter(|b| b.parent().map(|d| d.to_path_buf()) == dir);
        return match kept.and_then(|b| std::fs::read(b).ok()) {
            Some(bytes) => ("200 OK", CSV, bytes),
            None => (
                "404 Not Found",
                "text/plain; charset=utf-8",
                b"no copy of an earlier case is recorded".to_vec(),
            ),
        };
    }
    (
        "200 OK",
        CSV,
        vleo_modules::inputs::csv_with(&saved.reading.set, saved.reading.upgrade.as_ref())
            .into_bytes(),
    )
}

fn reading_json(j: &mut Json, r: &vleo_modules::inputs::Reading) {
    j.num_field("changed", r.changed as f64);
    j.num_field("defaulted", r.defaulted as f64);
    j.key("set").open_arr();
    for (k, (id, si)) in r.set.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("id", id);
        j.num_field("value", *si);
        j.close_obj();
    }
    j.close_arr();
    j.key("refused").open_arr();
    for (k, (line, id, why)) in r.refused.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.num_field("line", *line as f64);
        j.str_field("id", id);
        j.str_field("why", why);
        j.close_obj();
    }
    j.close_arr();
    match &r.template {
        Some(t) => j.str_field("template", t),
        None => j.key("template").raw("null"),
    };
    j.bool_field("outdated", r.outdated);
    j.key("upgrade");
    match &r.upgrade {
        None => {
            j.raw("null");
        }
        Some(u) => {
            j.raw("{");
            j.str_field("from", &u.from);
            match &u.backup {
                Some(b) => j.str_field("backup", b),
                None => j.key("backup").raw("null"),
            };
            j.key("new").open_arr();
            for (k, n) in u.new.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                j.push_string(n);
            }
            j.close_arr();
            j.key("set_aside").open_arr();
            for (k, a) in u.set_aside.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("id", &a.id);
                j.str_field("value", &a.value);
                j.str_field("unit", &a.unit);
                j.str_field("why", &a.why);
                j.close_obj();
            }
            j.close_arr();
            j.close_obj();
        }
    }
}

/// One node's form, as a file to download.
fn form_file(ctx: &Ctx, id: &str) -> (&'static str, &'static str, Vec<u8>) {
    let id = decode(id);
    let tree = match vleo_sheet::load::load_all(&ctx.root) {
        Ok(t) => t,
        Err(e) => {
            return (
                "500 Internal Server Error",
                "text/plain; charset=utf-8",
                format!("the tree does not load: {e}").into_bytes(),
            )
        }
    };
    // `new` is the form for a node the design does not have yet. No row can
    // be called that: an id names an answer, and `new` names none.
    if id == "new" {
        return (
            "200 OK",
            "text/html; charset=utf-8",
            vleo_sheet::template::document_new(&tree).into_bytes(),
        );
    }
    match tree.sheets.get(&id) {
        Some(sh) => (
            "200 OK",
            "text/html; charset=utf-8",
            vleo_sheet::template::document(sh, &tree).into_bytes(),
        ),
        None => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            format!("no node '{id}'").into_bytes(),
        ),
    }
}

// ---------------------------------------------------------------------------
// saved results

/// Now, as a result records it: UTC, to the second. Through `date` rather than a
/// crate, as xtask stamps its dates.
fn now_utc() -> String {
    std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn results_dir() -> PathBuf {
    vleo_data::results_path()
}

fn rows_json(j: &mut Json, key: &str, rows: &[vleo_modules::results::Row]) {
    j.key(key).open_arr();
    for (k, r) in rows.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("id", &r.id);
        j.str_field("name", &r.name);
        j.str_field("value", &r.value);
        j.str_field("unit", &r.unit);
        match r.si {
            Some(v) => j.num_field("si", v),
            None => j.key("si").raw("null"),
        };
        j.str_field("credibility", &r.credibility);
        j.str_field("governing", &r.governing);
        j.str_field("note", &r.note);
        j.close_obj();
    }
    j.close_arr();
}

fn result_head(j: &mut Json, s: &vleo_modules::results::Saved) {
    j.str_field("target", &s.target);
    j.str_field("name", &s.name);
    j.str_field("saved", &s.saved);
    j.str_field("mode", &s.mode);
    j.str_field("chain", &s.chain);
    j.str_field("kernel", &s.kernel);
    j.str_field("graph", &s.graph);
    j.str_field("template", &s.template);
    j.bool_field(
        "template_current",
        s.template == vleo_modules::inputs::template(),
    );
    j.str_field("data", &s.data.join(" "));
    j.num_field("ran", s.ran as f64);
    j.num_field("blocked", s.blocked_count as f64);
    j.num_field("changed", s.changed() as f64);
    // The node versions it rests on, and those whose record has moved on
    // since: each a belief the result rested on that has broken.
    j.key("versions").open_arr();
    for (i, (id, n, rel)) in s.versions.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("node", id);
        j.num_field("n", *n as f64);
        j.str_field("release", rel);
        j.close_obj();
    }
    j.close_arr();
    j.key("moved").open_arr();
    for (i, (id, then, now)) in vleo_modules::results::moved_since(s).iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("node", id);
        j.num_field("then", *then as f64);
        j.num_field("now", *now as f64);
        j.close_obj();
    }
    j.close_arr();
    match s.answer() {
        Some(a) => {
            j.key("answer").raw("{");
            j.str_field("value", &a.value);
            j.str_field("unit", &a.unit);
            j.close_obj();
        }
        None => {
            j.key("answer").raw("null");
        }
    }
}

/// Every saved result, newest first — and any file that no longer reads,
/// named with why rather than left out of the list.
fn results_list() -> String {
    let (good, bad) = vleo_modules::results::store::list(&results_dir());
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("path", &results_dir().display().to_string());
    j.key("results").open_arr();
    for (k, (file, s)) in good.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("file", file);
        result_head(&mut j, s);
        j.close_obj();
    }
    j.close_arr();
    j.key("unreadable").open_arr();
    for (k, (file, why)) in bad.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("file", file);
        j.str_field("why", why);
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}

fn failed(message: &str) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", false);
    j.str_field("message", message);
    j.raw("}");
    j.0
}

/// One saved result, whole.
fn result_json(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    let s = match vleo_modules::results::store::open(&results_dir(), &name) {
        Ok(s) => s,
        Err(e) => return failed(&e),
    };
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("file", &name);
    result_head(&mut j, &s);
    rows_json(&mut j, "inputs", &s.inputs);
    rows_json(&mut j, "outputs", &s.outputs);
    rows_json(&mut j, "blocked_rows", &s.blocked);
    j.raw("}");
    j.0
}

fn result_file(params: &str, report: bool) -> (&'static str, &'static str, Vec<u8>) {
    let name = param(params, "name").map(decode).unwrap_or_default();
    match vleo_modules::results::store::open(&results_dir(), &name) {
        Ok(s) if report => (
            "200 OK",
            "text/html; charset=utf-8",
            vleo_modules::results::html(&s).into_bytes(),
        ),
        Ok(s) => (
            "200 OK",
            "text/csv; charset=utf-8",
            vleo_modules::results::csv(&s).into_bytes(),
        ),
        Err(e) => ("404 Not Found", "text/plain; charset=utf-8", e.into_bytes()),
    }
}

/// Run a row on the saved case — with whatever the request sets on top — and
/// keep what it returned.
fn result_save(params: &str, ctx: &Ctx) -> String {
    if let Some(why) = case_refused(params, ctx) {
        return failed(&why);
    }
    let case = build_case(params, ctx);
    let mut scratch = Scratch::new();
    let r = match vleo_modules::evaluate(&case, &mut scratch) {
        Ok(r) => r,
        Err(f) => return failed(&format!("{f}")),
    };
    let label = param(params, "label").map(decode).unwrap_or_default();
    let s = vleo_modules::results::from_run(&r, &case.supply, &now_utc(), label.trim());
    match vleo_modules::results::store::save(&results_dir(), &s) {
        Ok(file) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("file", &file);
            result_head(&mut j, &s);
            j.raw("}");
            j.0
        }
        Err(e) => failed(&e),
    }
}

/// Why the design is what it is: every registered risk, where it stands and
/// every node version that moved it — the conclusion the risk-register rows
/// draw — and the version each node is at.
///
/// Read from the sheets on each request rather than held: the daemon never
/// writes a sheet, and a developer applying forms beside a running copy
/// should see the register move without a restart.
fn derisk_json(ctx: &Ctx) -> String {
    let tree = match vleo_sheet::load::load_all(&ctx.root) {
        Ok(t) => t,
        Err(e) => return failed(&format!("the tree does not load: {e}")),
    };
    let reg = vleo_sheet::derisk::register(&tree);
    let (changes, _) = vleo_sheet::derisk::narrative(&tree);
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.num_field("changes", changes.len() as f64);
    j.num_field(
        "published",
        tree.sheets.values().filter(|s| !s.is_seeded()).count() as f64,
    );
    j.num_field(
        "versioned",
        tree.sheets
            .values()
            .filter(|s| !s.versions.is_empty())
            .count() as f64,
    );
    j.key("register").open_arr();
    for (i, r) in reg.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        for (k, v) in [
            ("id", &r.id),
            ("title", &r.title),
            ("owner", &r.owner),
            ("why", &r.why),
            ("row", &r.row),
            ("row_label", &r.row_label),
            ("registered", &r.registered),
            ("now", &r.now),
        ] {
            j.str_field(k, v);
        }
        j.key("moves").open_arr();
        for (m_i, m) in r.moves.iter().enumerate() {
            if m_i > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("node", &m.node);
            j.str_field("label", &m.label);
            j.num_field("n", m.n as f64);
            j.str_field("date", &m.date);
            j.str_field("release", &m.release);
            j.str_field("what", &m.what);
            j.str_field("learned", &m.learned);
            j.close_obj();
        }
        j.close_arr();
        j.close_obj();
    }
    j.close_arr();
    j.key("nodes").raw("{");
    let mut first = true;
    for sh in tree.ordered() {
        let Some(v) = sh.versions.last() else {
            continue;
        };
        if !first {
            j.raw(",");
        }
        first = false;
        j.push_string(&sh.id);
        j.raw(":{");
        j.num_field("n", v.n as f64);
        j.str_field("release", &v.release);
        j.str_field("rests_on", &v.rests_on);
        j.str_field("learned", &v.learned);
        j.close_obj();
    }
    j.raw("}");
    j.raw("}");
    j.0
}

/// Keep a result somebody sent — its CSV, or the report page it rides in.
fn result_upload(params: &str) -> String {
    let text = param(params, "csv").map(decode).unwrap_or_default();
    let s = match vleo_modules::results::read(&vleo_modules::results::unwrap_report(&text)) {
        Ok(s) => s,
        Err(e) => return failed(&e),
    };
    match vleo_modules::results::store::save(&results_dir(), &s) {
        Ok(file) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("file", &file);
            result_head(&mut j, &s);
            j.raw("}");
            j.0
        }
        Err(e) => failed(&e),
    }
}

fn result_delete(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    match vleo_modules::results::store::remove(&results_dir(), &name) {
        Ok(()) => "{\"ok\":true}".to_string(),
        Err(e) => failed(&e),
    }
}

/// A result's inputs, made the saved case: checked like any other case, saved
/// only when every value still applies, and carried over first when the result
/// was saved against another set of inputs.
fn result_as_case(params: &str) -> String {
    let name = param(params, "name").map(decode).unwrap_or_default();
    let s = match vleo_modules::results::store::open(&results_dir(), &name) {
        Ok(s) => s,
        Err(e) => return failed(&e),
    };
    let r = vleo_modules::inputs::check_values(&s.case_values());
    let mut j = Json::new();
    j.raw("{");
    if !r.ok() {
        j.bool_field("ok", false);
        j.str_field(
            "message",
            &format!(
                "the case was not changed: {} of the result's inputs no longer apply to this tree",
                r.refused.len()
            ),
        );
        reading_json(&mut j, &r);
        j.raw("}");
        return j.0;
    }
    let path = case_path();
    let written = if r.set.is_empty() {
        vleo_modules::inputs::saved::clear(&path)
    } else {
        vleo_modules::inputs::saved::store(&path, &vleo_modules::inputs::csv(&r.set))
    };
    match written {
        Ok(()) => {
            j.bool_field("ok", true);
            reading_json(&mut j, &r);
        }
        Err(e) => {
            j.bool_field("ok", false);
            j.str_field("message", &format!("{}: {e}", path.display()));
        }
    }
    j.raw("}");
    j.0
}

/// What a filled form would change in its node. Writes nothing.
fn form_check(ctx: &Ctx, params: &str) -> String {
    use vleo_sheet::template::{self, Verdict};
    let mut j = Json::new();
    j.raw("{");
    let Some(html) = param(params, "html").map(decode) else {
        j.bool_field("ok", false);
        j.str_field("message", "no form was sent");
        j.raw("}");
        return j.0;
    };
    let p = match template::plan(&ctx.root, &html) {
        Ok(p) => p,
        Err(e) => {
            j.bool_field("ok", false);
            j.str_field("message", &e);
            j.raw("}");
            return j.0;
        }
    };
    let f = &p.form;
    j.bool_field("ok", true);
    j.str_field("node", &f.node);
    match &p.new {
        Some(n) => {
            j.key("new").raw("{");
            j.str_field("id", &n.id);
            j.str_field("parent", &n.parent);
            j.str_field("kind", &n.kind);
            j.close_obj();
        }
        None => {
            j.key("new").raw("null");
        }
    }
    j.key("interfaces").open_arr();
    for (k, i) in p.interfaces.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("binding", &i.binding);
        j.str_field("var", &i.var);
        j.str_field("have", &i.have);
        j.str_field("unit", &i.unit);
        j.str_field("label", &i.label);
        j.str_field("why", &i.why);
        j.close_obj();
    }
    j.close_arr();
    j.key("open").open_arr();
    for (k, o) in p.open.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.push_string(o);
    }
    j.close_arr();
    j.key("filled_by").raw("{");
    j.str_field("name", &f.name);
    j.str_field("team", &f.team);
    j.str_field("date", &f.date);
    j.str_field("ai", &f.ai);
    j.close_obj();
    j.bool_field("base_current", p.base_current);
    j.num_field("applicable", p.applicable() as f64);
    j.num_field("blocked", p.blocked() as f64);
    j.key("items").open_arr();
    for (k, i) in p.items.iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        let (verdict, why) = match &i.verdict {
            Verdict::Apply => ("apply", ""),
            Verdict::Already => ("already", ""),
            Verdict::Conflict(w) => ("conflict", w.as_str()),
            Verdict::Refused(w) => ("refused", w.as_str()),
        };
        j.raw("{");
        j.str_field("what", &i.what);
        j.str_field("from", &i.from);
        j.str_field("to", &i.to);
        j.str_field("verdict", verdict);
        j.str_field("why", why);
        j.close_obj();
    }
    j.close_arr();
    j.str_field("notes", &f.notes);
    j.num_field("known", f.known.len() as f64);
    j.str_field("fixture_request", &template::fixture_request(f));
    j.raw("}");
    j.0
}

/// Every input of the case, its group, default, range and saved value.
fn inputs_json() -> String {
    let loaded = saved_case();
    let saved = &loaded.reading;
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    if let Some(c) = Vleo::default_case() {
        j.str_field("case", c.id);
        j.str_field("label", c.label);
        j.str_field("note", c.note);
    }
    j.str_field("path", &case_path().display().to_string());
    j.bool_field("stored", loaded.stored);
    j.str_field("template_now", &vleo_modules::inputs::template());
    if let Some(e) = &loaded.error {
        j.str_field("error", e);
    }
    reading_json(&mut j, saved);
    j.key("inputs").open_arr();
    for (k, i) in vleo_modules::inputs::case_inputs().iter().enumerate() {
        if k > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("id", i.id);
        j.str_field("label", i.label);
        j.str_field("symbol", i.symbol);
        j.str_field("group", i.group.name());
        j.str_field("unit", i.unit);
        j.num_field("factor", i.factor);
        j.num_field("lo", i.lo);
        j.num_field("hi", i.hi);
        j.num_field("default", i.default);
        match saved.set.iter().find(|(id, _)| id == i.id) {
            Some((_, v)) => j.num_field("value", *v),
            None => j.key("value").raw("null"),
        };
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}

/// What a CSV or a set of values would do, without saving it.
fn inputs_reading(params: &str) -> vleo_modules::inputs::Reading {
    match param(params, "csv") {
        Some(csv) => vleo_modules::inputs::read_csv(&decode(csv)),
        None => vleo_modules::inputs::check_values(&sets(params)),
    }
}

fn inputs_check(params: &str) -> String {
    let r = inputs_reading(params);
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", r.ok());
    reading_json(&mut j, &r);
    j.raw("}");
    j.0
}

/// Save a case — a whole one, replacing what was there.
///
/// ALL OR NOTHING. A file with one refused row is not saved at all, and the
/// reply names every refusal: keeping the good rows of a bad file would leave
/// the tool running on a case nobody wrote.
fn inputs_save(params: &str) -> String {
    let r = inputs_reading(params);
    let mut j = Json::new();
    j.raw("{");
    if !r.ok() {
        j.bool_field("ok", false);
        j.str_field(
            "message",
            &format!(
                "nothing was saved: {} row(s) cannot be applied. Correct them and upload again.",
                r.refused.len()
            ),
        );
        reading_json(&mut j, &r);
        j.raw("}");
        return j.0;
    }
    let path = case_path();
    let written = vleo_modules::inputs::saved::store(&path, &vleo_modules::inputs::csv(&r.set));
    match written {
        Ok(()) => {
            j.bool_field("ok", true);
            j.str_field("path", &path.display().to_string());
            reading_json(&mut j, &r);
        }
        Err(e) => {
            j.bool_field("ok", false);
            j.str_field(
                "message",
                &format!("the case could not be written to {}: {e}", path.display()),
            );
        }
    }
    j.raw("}");
    j.0
}

/// Back to every default: the stored case is removed.
fn inputs_reset() -> String {
    let path = case_path();
    let mut j = Json::new();
    j.raw("{");
    match vleo_modules::inputs::saved::clear(&path) {
        Ok(()) => {
            j.bool_field("ok", true);
        }
        Err(e) => {
            j.bool_field("ok", false);
            j.str_field(
                "message",
                &format!("could not remove {}: {e}", path.display()),
            );
        }
    }
    j.raw("}");
    j.0
}

/// `set=id:value` repeated. Values are SI, always: a face converts for display
/// and never for transport.
fn sets(params: &str) -> Vec<(String, f64)> {
    params
        .split('&')
        .filter_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            if k != "set" {
                return None;
            }
            let d = decode(v);
            let (id, val) = d.split_once(':')?;
            Some((id.to_string(), val.parse().ok()?))
        })
        .collect()
}

/// A supplied value only survives on a row that declares its own number.
///
/// Every other kind works its answer out during the run and overwrites what was
/// supplied, so a `set=` on one was accepted, ignored, and reported as a
/// successful run against a number nobody asked for. A sweep over one drew a
/// flat line and said "0 refused", which reads as a real result — a reader
/// turns the knob and nothing moves, and nothing anywhere says why.
///
/// The CLI has refused this since it was written (`suppliable`); this path
/// never checked. It stayed invisible while every driver was declared, and
/// became load-bearing the moment `env_f107` started reading the solar
/// subsystem: the two faces then disagreed about the same request.
fn unsuppliable(id: &str) -> Option<String> {
    let k = Vleo::find(id)?;
    let def = &NODES[k as usize];
    if def.kind == Kind::Declared {
        return None;
    }
    Some(format!(
        "'{}' is {}, so a supplied value would be overwritten the moment it is \
         evaluated. Set one of the declared numbers it reads instead.",
        def.id,
        match def.kind {
            Kind::Computed => "computed from its inputs",
            Kind::Required => "a target handed down from the layer above",
            Kind::Achieved => "what a subsystem returned",
            Kind::Kpi => "a key performance indicator",
            Kind::Declared => unreachable!(),
        }
    ))
}

/// The refusal, as the wire form every endpoint here uses.
fn refuse(node: &str, message: &str) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", false);
    j.str_field("fault", "not-suppliable");
    j.str_field("node", node);
    j.str_field("message", message);
    j.raw("}");
    j.0
}

/// A customer or condition the engine would refuse, as the wire refusal.
///
/// Asked before any evaluation, by every endpoint that builds a case, so an
/// unknown name or a condition that cannot be applied comes back as a sentence
/// rather than as the declared design with a customer's name on it.
fn case_refused(params: &str, ctx: &Ctx) -> Option<String> {
    let case = build_case(params, ctx);
    let why = vleo_modules::case_refusal(&case)?;
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", false);
    j.str_field("fault", "case-refused");
    j.str_field("node", &case.base);
    j.str_field("message", &why);
    j.raw("}");
    Some(j.0)
}

fn build_case(params: &str, ctx: &Ctx) -> Case {
    // THE SAVED CASE FIRST, THEN WHAT THIS REQUEST TYPED ON TOP. Every run,
    // sweep, lever and panel therefore answers for the inputs the person saved,
    // without each face having to carry them. `inputs=defaults` asks for the
    // design as declared instead — what the parity and audit tools compare, so
    // their verdicts never depend on whatever case somebody last uploaded.
    let mut supply = if param(params, "inputs").map(decode).as_deref() == Some("defaults") {
        Vec::new()
    } else {
        saved_case().reading.set
    };
    supply.extend(sets(params));
    Case {
        // `case` names the case — today there is one. Empty means that one.
        base: param(params, "case").map(decode).unwrap_or_default(),
        supply,
        target: param(params, "node").map(decode).unwrap_or_default(),
        mode: RunMode::from_name(
            &param(params, "mode")
                .map(decode)
                .unwrap_or_else(|| "branch".into()),
        ),
        data: ctx.data.clone(),
        data_versions: ctx.data_versions.clone(),
    }
}

fn run_json(params: &str, ctx: &Ctx) -> String {
    if let Some(refusal) = case_refused(params, ctx) {
        return refusal;
    }
    let case = build_case(params, ctx);
    // Refuse before running, not after: a supplied value that cannot survive
    // the run has to be reported as a refusal rather than silently dropped.
    for (id, _) in &case.supply {
        if let Some(why) = unsuppliable(id) {
            return refuse(id, &why);
        }
    }
    let mut scratch = Scratch::new();
    let mut j = Json::new();
    j.raw("{");
    match vleo_modules::evaluate(&case, &mut scratch) {
        Err(f) => {
            // A refusal is a value, not a magic number the caller may forget to
            // check. It names the field, the bound and the reason.
            j.bool_field("ok", false);
            j.str_field("fault", f.kind());
            j.str_field("node", f.node());
            j.str_field("message", &format!("{f}"));
        }
        Ok(r) => {
            j.bool_field("ok", true);
            // Which inputs this answer was for. A number with no case beside it
            // is a number a reader will quote for the wrong inputs.
            let saved = if param(params, "inputs").map(decode).as_deref() == Some("defaults") {
                None
            } else {
                Some(saved_case().reading)
            };
            let note = saved.as_ref().and_then(|s| s.upgrade.as_ref());
            j.key("inputs").raw("{");
            j.bool_field("defaults", saved.is_none());
            j.num_field(
                "changed",
                saved.as_ref().map(|s| s.changed).unwrap_or(0) as f64,
            );
            // What the last update did to the case, until it is next saved:
            // values it could not carry, and inputs it added at their defaults.
            j.bool_field("upgraded", note.is_some());
            j.num_field(
                "set_aside",
                note.map(|u| u.set_aside.len()).unwrap_or(0) as f64,
            );
            j.num_field("new", note.map(|u| u.new.len()).unwrap_or(0) as f64);
            j.close_obj();
            j.key("values").open_arr();
            for (i, v) in r.values.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                let unit = Vleo::find(&v.id)
                    .map(|k| VARS[k as usize].unit)
                    .unwrap_or(vleo_units::Unit::One);
                let (shown, sym) = vleo_bus::present(v.value, unit, 6);
                j.raw("{");
                j.str_field("id", &v.id);
                j.str_field("symbol", &v.symbol);
                j.str_field("label", &v.label);
                j.num_field("si", v.value);
                j.str_field("shown", &shown);
                j.str_field("unit", sym);
                j.num_field("cred", v.cred.governing_score() as f64);
                j.str_field("governing", v.governing);
                j.key("vec").open_arr();
                for (k, c) in v.cred.0.iter().enumerate() {
                    if k > 0 {
                        j.raw(",");
                    }
                    j.raw(&c.to_string());
                }
                j.close_arr();
                j.close_obj();
            }
            j.close_arr();
            j.key("blocked").open_arr();
            for (i, b) in r.blocked.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("id", &b.id);
                j.str_field("kind", b.kind);
                j.str_field("message", &b.message);
                j.close_obj();
            }
            j.close_arr();
            j.key("verdicts").open_arr();
            for (i, v) in r.verdicts.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("node", &v.node);
                j.str_field("label", &v.label);
                j.num_field("expected", v.expected);
                j.num_field("got", v.got);
                j.num_field("error", v.relative_error);
                j.num_field("tolerance", v.tolerance);
                j.bool_field("passed", v.passed);
                j.str_field("provenance", v.provenance);
                j.str_field("source", v.source);
                j.close_obj();
            }
            j.close_arr();
            j.key("manifest").raw("{");
            j.str_field("node", &r.manifest.node);
            j.str_field("mode", r.manifest.mode);
            j.str_field("kernel", &r.manifest.kernel);
            j.str_field("graph", &r.manifest.graph);
            j.str_field("case", &r.manifest.case);
            j.str_field("chain", &r.manifest.chain);
            j.str_field("endpoint", "local-daemon");
            j.num_field("ran", r.manifest.ran as f64);
            j.num_field("blocked", r.manifest.blocked_count as f64);
            j.num_field("iterations", r.manifest.iterations as f64);
            j.key("data").open_arr();
            for (i, d) in r.manifest.data.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.push_string(d);
            }
            j.close_arr();
            j.close_obj();
        }
    }
    j.raw("}");
    j.0
}

/// A behaviour sweep. Refused points are recorded with their reason, never
/// dropped — a sweep in which some rows quietly used a substituted value is a
/// sweep whose conclusion is unknown.
/// `/v1/probe?node=<id>&in=<var>:<value>&in=…` — one relation, at given inputs.
///
/// The counterpart to `run` refusing a supplied value on a computed row. That
/// refusal is right: the run would overwrite it. But a relation's behaviour at
/// chosen driver values is a different question with a different answer, and
/// until `env_f107` became computed the two could be asked the same way.
///
/// Inputs are named by the VARIABLE they bind, not given positionally. The
/// dispatch table takes them in the node's declared order, and a caller
/// counting commas to match that order is a caller that silently swaps two
/// drivers of the same type the first time a sheet's input list is reordered —
/// which `env_exospheric_temperature`, whose three drivers are all `Ratio`,
/// would do without a single type error.
fn probe_json(params: &str) -> String {
    let node = param(params, "node").map(decode).unwrap_or_default();
    let Some(k) = Vleo::find(&node) else {
        return refuse(&node, "no such row");
    };
    let def = &NODES[k as usize];

    let mut given: BTreeMap<String, f64> = BTreeMap::new();
    for kv in params.split('&') {
        if let Some(v) = kv.strip_prefix("in=") {
            let d = decode(v);
            let Some((name, val)) = d.rsplit_once(':') else {
                return refuse(&node, &format!("'{d}' is not <var>:<value>"));
            };
            let Ok(x) = val.parse::<f64>() else {
                return refuse(&node, &format!("'{val}' is not a number"));
            };
            given.insert(name.to_string(), x);
        }
    }

    // Every declared input must be given, by name. A missing one defaulting to
    // zero is a probe that answers confidently about a relation nobody asked
    // about, which is the whole failure this endpoint exists to stop repeating.
    let mut inputs = Vec::with_capacity(def.inputs.len());
    for &v in def.inputs {
        let id = VARS[v as usize].id;
        match given.remove(id) {
            Some(x) => inputs.push(x),
            None => {
                return refuse(
                    &node,
                    &format!(
                        "no value given for input '{id}'; this row reads {} of them",
                        def.inputs.len()
                    ),
                )
            }
        }
    }
    if let Some((extra, _)) = given.iter().next() {
        return refuse(&node, &format!("'{extra}' is not an input of {}", def.id));
    }

    let mut j = Json::new();
    j.raw("{");
    match vleo_modules::probe(k, &inputs) {
        Err(f) => {
            j.bool_field("ok", false);
            j.str_field("fault", f.kind());
            j.str_field("node", f.node());
            j.str_field("message", &format!("{f}"));
        }
        Ok(out) => {
            j.bool_field("ok", true);
            j.str_field("node", def.id);
            // The primary answer, then every published member in slot order.
            j.num_field("si", out[0]);
            // UNIT AND FACTOR, the same pair the sweep endpoint sends.
            //
            // Everything crossing this boundary is SI and a face divides by the
            // factor to display. A caller that has to source those from
            // somewhere else is a caller that gets `undefined`, divides by it,
            // and draws nothing — which is exactly what the thermosphere
            // panel's two flux curves did the first time they came through
            // here: legend entries with no lines under them.
            let ov = VARS[def.outputs[0] as usize].unit;
            j.str_field("unit", ov.symbol());
            j.num_field("factor", ov.si_factor());
            j.key("inputs").open_arr();
            for (i, &v) in def.inputs.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                let var = &VARS[v as usize];
                j.raw("{");
                j.str_field("id", var.id);
                j.str_field("unit", var.unit.symbol());
                j.num_field("factor", var.unit.si_factor());
                j.close_obj();
            }
            j.close_arr();
            j.key("outputs").open_arr();
            for (i, &v) in def.outputs.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("id", VARS[v as usize].id);
                j.num_field("si", out[i]);
                j.close_obj();
            }
            j.close_arr();
        }
    }
    j.raw("}");
    j.0
}

fn sweep_json(params: &str, ctx: &Ctx) -> String {
    let node = param(params, "node").map(decode).unwrap_or_default();
    let over = param(params, "over").map(decode).unwrap_or_default();
    let from: f64 = param(params, "from")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    let to: f64 = param(params, "to")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0);
    let points: usize = param(params, "points")
        .and_then(|v| v.parse().ok())
        .unwrap_or(48)
        .clamp(2, 400);

    // The axis has to be a row a reader can actually move. Sweeping a computed
    // one drew a flat line and reported no refusals, which is the same silent
    // substitution as `set=` on one and reads as a real result.
    if let Some(why) = unsuppliable(&over) {
        return refuse(&over, &why);
    }
    if let Some(refusal) = case_refused(params, ctx) {
        return refusal;
    }

    let mut j = Json::new();
    j.raw("{");
    let (ni, oi) = match (Vleo::find(&node), Vleo::find(&over)) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            j.bool_field("ok", false);
            j.str_field("message", "the sweep names a node that does not exist");
            j.raw("}");
            return j.0;
        }
    };
    j.bool_field("ok", true);
    j.str_field("x_id", &over);
    j.str_field("y_id", &node);
    j.str_field("x_unit", VARS[oi as usize].unit.symbol());
    j.str_field("y_unit", VARS[ni as usize].unit.symbol());
    j.num_field("x_factor", VARS[oi as usize].unit.si_factor());
    j.num_field("y_factor", VARS[ni as usize].unit.si_factor());

    let mut scratch = Scratch::new();
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut refused: Vec<(f64, String)> = Vec::new();
    for i in 0..points {
        let t = i as f64 / (points - 1) as f64;
        let x = from + t * (to - from);
        let mut case = build_case(params, ctx);
        case.target = node.clone();
        case.supply.push((over.clone(), x));
        match vleo_modules::evaluate(&case, &mut scratch) {
            Ok(r) => match r.values.iter().find(|v| v.id == node) {
                Some(v) => {
                    xs.push(x);
                    ys.push(v.value);
                }
                None => refused.push((x, "blocked".to_string())),
            },
            Err(f) => refused.push((x, format!("{f}"))),
        }
    }
    j.key("x").open_arr();
    for (i, v) in xs.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&json::num(*v));
    }
    j.close_arr();
    j.key("y").open_arr();
    for (i, v) in ys.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&json::num(*v));
    }
    j.close_arr();
    j.key("refused").open_arr();
    for (i, (x, why)) in refused.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.num_field("x", *x);
        j.str_field("why", why);
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}

/// Which declared decisions actually move this node's answer.
///
/// The sweep control has to offer the reader a decision, and every declared
/// row anywhere upstream is a candidate. That list is useless on its own: a
/// term can be in the relation, be correct, and still be inert — the
/// persistence weight in sw_central_expectation is exp(-L/27) and contributes
/// 0.0001 per cent at any mission lead, so a reader handed it as the default
/// sweeps it, sees a flat line, and concludes the tool is broken. It is not
/// broken; it is answering a question nobody would have asked had they known
/// the answer.
///
/// So this measures rather than guesses. Each candidate is evaluated at both
/// ends of its own declared range with everything else held at the case, and
/// the answer's span is reported. A face can then lead with the decision that
/// moves the answer most, and say plainly of the others that they do not.
///
/// A candidate whose ends both refuse is reported with its reason, not
/// dropped: a decision that cannot be swept is a different fact from one that
/// changes nothing, and collapsing the two would hide a broken bound.
/// The maximal active branches a row is in.
///
/// A branch is the dependency closure of one row. A row is in as many branches
/// as there are rows that read it, and most of those nest inside each other, so
/// what is useful is the maximal ones: a candidate that no other candidate
/// reads, directly or at any distance. Running that set computes every row in
/// every active branch containing the input and computes none of them twice.
///
/// ACTIVE means every row in the closure is `published` — the tree is filled in
/// that far. It does not mean the branch will succeed: a published row can
/// still refuse when its own value lands outside its own declared domain, and
/// that refusal is a real answer rather than a gap.
///
/// THIS LIVES HERE RATHER THAN IN THE FACE because two things need it — the
/// node page and the audit in tools/ — and a rule with two implementations is
/// a rule that drifts. The face used to walk the graph in JavaScript off the
/// index; it now asks for this.
fn branches_json(params: &str) -> String {
    let node = param(params, "node").map(decode).unwrap_or_default();
    let mut j = Json::new();
    j.raw("{");
    let ni = match Vleo::find(&node) {
        Some(a) => VARS[a as usize].producer,
        None => {
            j.bool_field("ok", false);
            j.str_field("message", "the branches name a node that does not exist");
            j.raw("}");
            return j.0;
        }
    };
    j.bool_field("ok", true);
    j.str_field("node", &node);

    // node -> the nodes that read it. Built from `inputs`, which holds VARIABLE
    // indices, so each is mapped to its producing node first; conflating the
    // two indices is a defect this file has already shipped once.
    let mut cons: Vec<Vec<u16>> = vec![Vec::new(); NODES.len()];
    for (n, d) in NODES.iter().enumerate() {
        for x in d.inputs {
            cons[VARS[*x as usize].producer as usize].push(n as u16);
        }
    }
    let walk = |from: u16, edges: &Vec<Vec<u16>>| -> Vec<bool> {
        let mut seen = vec![false; NODES.len()];
        let mut st = vec![from];
        while let Some(n) = st.pop() {
            for m in &edges[n as usize] {
                if !seen[*m as usize] {
                    seen[*m as usize] = true;
                    st.push(*m);
                }
            }
        }
        seen
    };
    let mut prod: Vec<Vec<u16>> = vec![Vec::new(); NODES.len()];
    for (n, d) in NODES.iter().enumerate() {
        for x in d.inputs {
            prod[n].push(VARS[*x as usize].producer);
        }
    }
    let live = |n: u16| NODES[n as usize].state == vleo_core::graph::State::Published;

    let down = walk(ni, &cons);
    let mut cand: Vec<(u16, usize)> = Vec::new();
    for n in 0..NODES.len() as u16 {
        if !down[n as usize] || !live(n) {
            continue;
        }
        let cl = walk(n, &prod);
        let mut ok = true;
        let mut size = 1usize;
        for (k, inside) in cl.iter().enumerate() {
            if *inside {
                size += 1;
                if !live(k as u16) {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            cand.push((n, size));
        }
    }
    let is_cand: Vec<bool> = {
        let mut v = vec![false; NODES.len()];
        for (n, _) in &cand {
            v[*n as usize] = true;
        }
        v
    };
    let mut out: Vec<(u16, usize)> = cand
        .iter()
        .filter(|(n, _)| {
            let up = walk(*n, &cons);
            !(0..NODES.len()).any(|k| up[k] && is_cand[k])
        })
        .cloned()
        .collect();
    // Biggest first: the branch that covers most of the design is the one a
    // reader wants at the top.
    out.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| NODES[a.0 as usize].id.cmp(NODES[b.0 as usize].id))
    });

    // How many rows read this one at all, so a face can say whether an empty
    // list means "nothing reads it" or "everything that does is unfinished".
    let read_by = (0..NODES.len()).filter(|k| down[*k]).count();
    j.num_field("read_by", read_by as f64);

    j.key("branches").open_arr();
    for (i, (n, size)) in out.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        let d = &NODES[*n as usize];
        j.raw("{");
        j.str_field("id", d.id);
        j.str_field("label", d.label);
        j.str_field("sub", d.subsystem);
        j.num_field("rows", *size as f64);
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}

fn levers_json(params: &str, ctx: &Ctx) -> String {
    let node = param(params, "node").map(decode).unwrap_or_default();
    if let Some(refusal) = case_refused(params, ctx) {
        return refusal;
    }
    let mut j = Json::new();
    j.raw("{");
    let ni = match Vleo::find(&node) {
        Some(a) => a,
        None => {
            j.bool_field("ok", false);
            j.str_field("message", "the levers name a node that does not exist");
            j.raw("}");
            return j.0;
        }
    };
    j.bool_field("ok", true);
    j.str_field("node", &node);

    // Every declared row upstream, however far: a decision three rows away is
    // still a decision, and the reason a reader opens this row may be a choice
    // taken well before it.
    // The walk is over NODES, because `inputs` is a node's declaration of what
    // it reads. A candidate is a VARIABLE, because that is what a sweep
    // supplies: the two indices are different spaces and conflating them is
    // the defect this file has already shipped once.
    let start = VARS[ni as usize].producer;
    let mut seen = vec![false; NODES.len()];
    let mut stack = vec![start];
    seen[start as usize] = true;
    let mut cands: Vec<u16> = Vec::new();
    while let Some(n) = stack.pop() {
        for x in NODES[n as usize].inputs {
            let pv = &VARS[*x as usize];
            let pn = pv.producer;
            if !seen[pn as usize] {
                seen[pn as usize] = true;
                stack.push(pn);
            }
            if *x != ni
                && NODES[pn as usize].kind == vleo_core::graph::Kind::Declared
                && pv.limit.upper > pv.limit.lower
                && !cands.contains(x)
            {
                cands.push(*x);
            }
        }
    }

    let mut scratch = Scratch::new();
    let base = {
        let mut case = build_case(params, ctx);
        case.target = node.clone();
        vleo_modules::evaluate(&case, &mut scratch)
            .ok()
            .and_then(|r| r.values.iter().find(|v| v.id == node).map(|v| v.value))
    };
    match base {
        Some(b) => j.num_field("base", b),
        None => j.key("base").raw("null"),
    };

    /// One decision and what moving it across its own declared range does to
    /// the answer. `span` is negative where an end could not be evaluated at
    /// all, which is a different fact from a span of zero.
    struct Lever {
        span: f64,
        var: u16,
        at_lower: Option<f64>,
        at_upper: Option<f64>,
        why: String,
    }
    let mut out: Vec<Lever> = Vec::new();
    for v in cands {
        let d = &VARS[v as usize];
        let mut ends: [Option<f64>; 2] = [None, None];
        let mut why = String::new();
        for (k, x) in [d.limit.lower, d.limit.upper].iter().enumerate() {
            let mut case = build_case(params, ctx);
            case.target = node.clone();
            case.supply.push((d.id.to_string(), *x));
            match vleo_modules::evaluate(&case, &mut scratch) {
                Ok(r) => {
                    ends[k] = r.values.iter().find(|q| q.id == node).map(|q| q.value);
                    if ends[k].is_none() && why.is_empty() {
                        why = "blocked".to_string();
                    }
                }
                Err(f) => {
                    if why.is_empty() {
                        why = format!("{f}");
                    }
                }
            }
        }
        // The span is relative to the base where there is one, so decisions on
        // rows of different size can be ranked against each other at all.
        let span = match (ends[0], ends[1]) {
            (Some(a), Some(b)) => {
                let d = (b - a).abs();
                match base {
                    Some(z) if z != 0.0 => d / z.abs(),
                    _ => d,
                }
            }
            _ => -1.0,
        };
        out.push(Lever {
            span,
            var: v,
            at_lower: ends[0],
            at_upper: ends[1],
            why,
        });
    }
    // Most movement first, so the face's default is the decision worth asking
    // about. Ties keep a stable order by id for a page that does not reshuffle.
    out.sort_by(|a, b| {
        b.span
            .partial_cmp(&a.span)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| VARS[a.var as usize].id.cmp(VARS[b.var as usize].id))
    });

    j.key("levers").open_arr();
    for (i, lev) in out.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        let d = &VARS[lev.var as usize];
        j.raw("{");
        j.str_field("id", d.id);
        j.str_field("symbol", d.symbol);
        j.str_field("label", d.label);
        j.str_field("unit", d.unit.symbol());
        j.num_field("factor", d.unit.si_factor());
        j.num_field("lower", d.limit.lower);
        j.num_field("upper", d.limit.upper);
        match lev.at_lower {
            Some(x) => j.num_field("at_lower", x),
            None => j.key("at_lower").raw("null"),
        };
        match lev.at_upper {
            Some(x) => j.num_field("at_upper", x),
            None => j.key("at_upper").raw("null"),
        };
        if lev.span < 0.0 {
            j.key("span").raw("null");
        } else {
            j.num_field("span", lev.span);
        }
        j.str_field("why", &lev.why);
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}
