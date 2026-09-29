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

mod http;
mod inputs;
mod json;
mod pages;
mod results;

use http::*;
use inputs::*;
use json::Json;
use pages::*;
use results::*;
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
    // First, so that a bug anywhere after this — in any request — leaves a file
    // a person can send (vleo_data::crash). The daemon and `python -m vleo`
    // both start here.
    vleo_data::crash::install("vleo-server", env!("CARGO_PKG_VERSION"));
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

/// Open the page in the default browser.
///
/// On Windows through `explorer.exe`, which hands a web address to the default
/// browser. It used to be `cmd /C start`: a program that starts a command shell
/// is one of the first things an antivirus heuristic looks for, and the tool
/// has no reason to look like that.
fn open_browser(url: &str) {
    let opener = if cfg!(target_os = "windows") {
        "explorer.exe"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(opener).arg(url).spawn();
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

fn version_json(ctx: &Ctx) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.str_field("kernel", &short(Vleo::kernel_hash()));
    j.str_field("graph", &short(Vleo::graph_hash()));
    j.str_field("version", env!("CARGO_PKG_VERSION"));
    j.str_field("endpoint", "local-daemon");
    j.num_field("port", ctx.port as f64);
    j.num_field("nodes", NODES.len() as f64);
    if let Some(p) = &ctx.preview {
        j.key("preview").raw(p);
    }
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

fn failed(message: &str) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", false);
    j.str_field("message", message);
    j.raw("}");
    j.0
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
    if let Some((node, why)) = set_refusal(params) {
        let mut j = Json::new();
        j.raw("{");
        j.bool_field("ok", false);
        j.str_field("fault", "bad-set");
        j.str_field("node", &node);
        j.str_field("message", &why);
        j.raw("}");
        return Some(j.0);
    }
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

#[cfg(test)]
mod requests {
    use super::{decode, set_refusal, Head, MAX_BODY};

    fn head(lines: &[&str]) -> Head {
        let mut h = Head::default();
        for l in lines {
            h.take(l);
        }
        h
    }

    #[test]
    fn decoding_never_panics_and_keeps_what_it_cannot_read() {
        assert_eq!(decode("a+b%20c"), "a b c");
        assert_eq!(decode("%C2%B0"), "°");
        // A raw multi-byte character after `%` once sliced text mid-character
        // and aborted the release build.
        assert_eq!(decode("%€x"), "%€x");
        assert_eq!(decode("%zz"), "%zz");
        assert_eq!(decode("50%"), "50%");
        assert_eq!(decode("%4"), "%4");
    }

    #[test]
    fn only_this_server_s_own_address_is_answered() {
        let ok = head(&["Host: 127.0.0.1:7777"]);
        assert!(ok.refusal("GET", 7777).is_none());
        assert!(head(&["Host: localhost:7777"])
            .refusal("GET", 7777)
            .is_none());
        // DNS rebinding: another site's name, pointed at loopback.
        let rebound = head(&["Host: evil.example:7777"]);
        assert_eq!(
            rebound.refusal("GET", 7777).unwrap().0,
            "421 Misdirected Request"
        );
        assert!(head(&["Host: 127.0.0.1:7778"])
            .refusal("GET", 7777)
            .is_some());
        assert!(head(&[]).refusal("GET", 7777).is_some());
    }

    #[test]
    fn a_page_on_another_site_cannot_change_anything() {
        let own = head(&["Host: 127.0.0.1:7777", "Origin: http://127.0.0.1:7777"]);
        assert!(own.refusal("POST", 7777).is_none());
        let other = head(&["Host: 127.0.0.1:7777", "Origin: https://evil.example"]);
        assert_eq!(other.refusal("POST", 7777).unwrap().0, "403 Forbidden");
        let null = head(&["Host: 127.0.0.1:7777", "Origin: null"]);
        assert!(null.refusal("POST", 7777).is_some());
        let told = head(&["Host: 127.0.0.1:7777", "Sec-Fetch-Site: cross-site"]);
        assert!(told.refusal("POST", 7777).is_some());
        // A program, not a page: the parity tools post with neither header.
        let program = head(&["Host: localhost:7777"]);
        assert!(program.refusal("POST", 7777).is_none());
        // Reading is left to the browser's own same-origin rule.
        assert!(other.refusal("GET", 7777).is_none());
    }

    #[test]
    fn bodies_are_bounded_and_measured() {
        let h = |l: &str| {
            head(&["Host: 127.0.0.1:1", l])
                .refusal("POST", 1)
                .map(|r| r.0)
        };
        assert_eq!(h("Transfer-Encoding: chunked"), Some("411 Length Required"));
        assert_eq!(h("Content-Length: lots"), Some("400 Bad Request"));
        let big = format!("Content-Length: {}", MAX_BODY + 1);
        assert_eq!(h(&big), Some("413 Content Too Large"));
        assert_eq!(h("Content-Length: 10"), None);
    }

    #[test]
    fn a_value_that_cannot_be_applied_is_named_not_dropped() {
        let id = vleo_modules::NODES[0].id;
        assert!(set_refusal(&format!("set={id}:1.5")).is_none());
        assert!(set_refusal(&format!("set={id}%3A2")).is_none());
        let named = |p: &str| set_refusal(p).map(|(n, _)| n);
        assert_eq!(named("set=no_such_row:1").as_deref(), Some("no_such_row"));
        assert_eq!(named(&format!("set={id}:fast")).as_deref(), Some(id));
        assert_eq!(named(&format!("set={id}:NaN")).as_deref(), Some(id));
        assert!(named(&format!("set={id}")).is_some());
        assert!(set_refusal("node=x&mode=branch").is_none());
    }

    #[test]
    fn a_bug_in_one_request_is_answered_and_the_next_request_still_served() {
        use super::handled;
        let (status, ctype, body) = handled("GET", "/v1/run", || panic!("a bug in the handler"));
        assert_eq!(status, "500 Internal Server Error");
        assert!(
            ctype.starts_with("application/json"),
            "the face reads /v1 as JSON"
        );
        let body = String::from_utf8(body).unwrap();
        assert!(
            body.contains("\"ok\":false") && body.contains("still running"),
            "{body}"
        );
        // A page, not the API, is told in plain text.
        assert!(handled("GET", "/", || panic!("again"))
            .1
            .starts_with("text/plain"));
        // And the next request is served as if nothing had happened.
        let fine = handled("GET", "/v1/version", || {
            ("200 OK", "application/json", b"{}".to_vec())
        });
        assert_eq!(fine.0, "200 OK");
    }

    #[test]
    fn a_connection_slot_is_given_back_when_its_thread_panics() {
        use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
        use std::sync::Arc;
        let open = Arc::new(AtomicUsize::new(1));
        let place = super::OpenPlace(open.clone());
        let ended = std::thread::spawn(move || {
            let _place = place;
            panic!("a bug outside the guarded handler");
        })
        .join();
        assert!(ended.is_err());
        assert_eq!(
            open.load(SeqCst),
            0,
            "the slot leaked; enough of these and every connection is refused"
        );
    }
}
