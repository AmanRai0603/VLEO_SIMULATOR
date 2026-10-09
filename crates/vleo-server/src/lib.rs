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
pub mod results_file;
pub mod today;

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
use vleo_modules::{cases, groups, nodes, relations, vars, Scratch, Vleo};
use vleo_sheet::files::Files;

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
    let (tree, design, today) = open_tree(&root)?;
    let engine = run_on_the_files(&*tree, &root, design.is_some())?;
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
        nodes().len()
    );
    println!("  engine {engine}");
    for line in &today {
        println!("  {line}");
    }
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
    match &design {
        Some(d) => println!(
            "  design {} — {} rows, fingerprint {}…",
            d.file,
            d.rows,
            d.fingerprint.chars().take(16).collect::<String>()
        ),
        None => println!("  design read from the folders of this checkout"),
    }
    println!("  serving the interface and the engine from one origin:");
    println!("  \x1b[1mhttp://127.0.0.1:{port}\x1b[0m");
    println!("  loopback only. Exposing this to a network is a separate, explicit act.");
    // Opened by the tool, because only the tool knows which port it got — 7777
    // may be taken, and a script that guesses opens somebody else's page.
    if open {
        open_browser(&format!("http://127.0.0.1:{port}"));
    }

    results::thin_at_start();
    let ctx = std::sync::Arc::new(Ctx {
        root,
        tree,
        design,
        data,
        data_versions,
        bundles,
        port,
    });
    if background {
        std::thread::spawn(move || accept(listener, ctx));
    } else {
        accept(listener, ctx);
    }
    Ok(port)
}

/// One figure of the solar-weather record, without a server: the JSON
/// `GET /v1/figures/solar/<id>?<params>` answers, byte for byte, because it is
/// the same function. The command line (`vleo figure`) and Python
/// (`vleo.figure`) ask here, so a number a panel draws is a number either can
/// reproduce, from the same bundle and the same saved case.
///
/// `params` is a query string (`v=ap&by=cycle`). An unknown id, driver, split
/// or view is refused by name, as the route refuses it: the answer carries
/// `"ok":false` and a message, never a stand-in figure.
pub fn figure(root: Option<PathBuf>, id: &str, params: &str) -> String {
    let root = root.unwrap_or_else(repo_root);
    let (tree, design, _) = match open_tree(&root) {
        Ok(t) => t,
        Err(e) => return format!("{{\"ok\":false,\"message\":{}}}", json::string(&e)),
    };
    let (data, data_versions, bundles, _) = resolve_data(&root);
    let ctx = Ctx {
        root,
        tree,
        design,
        data,
        data_versions,
        bundles,
        port: 0,
    };
    results::record_figure(&ctx, id, params)
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
    /// Where the design is read from: its files, `design/`, or the sheets'
    /// folders under `root` where there are none (`open_tree`). Every read of a sheet, a page or a
    /// lesson goes through it; the web face and the reference data do not.
    tree: std::sync::Arc<dyn Files>,
    /// The design's files, when the design is read from them.
    design: Option<DesignFile>,
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

/// What the tool says about the design's files it reads.
struct DesignFile {
    file: String,
    rows: String,
    fingerprint: String,
}

/// Where the design is read from.
///
/// The design is its files, `design/`, beside the web face in a kit and in a
/// checkout alike, or the folder `VLEO_DESIGN` names. A design that is there
/// and does not open stops the tool, naming why: falling back to whatever
/// else happens to sit beside it would show a different design under the
/// same name.
fn open_tree(root: &Path) -> Result<Opened, String> {
    let (files, design) = open_base(root)?;
    let Some(drive) = std::env::var("VLEO_DRIVE")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)
    else {
        *CHANGED.write().unwrap_or_else(|e| e.into_inner()) = Vec::new();
        return Ok((files, design, Vec::new()));
    };
    // Today's design: every group's latest sealed release on the drive that
    // passes its checks, taken into the design (`today`). A drive that is not
    // one stops the tool, naming why; the design is never shown as today's
    // when it is not.
    let t = today::build(files, root, &drive)
        .map_err(|e| format!("today's design is not built: {e}"))?;
    let mut said = vec![format!(
        "today's design, built from the drive {}:",
        drive.display()
    )];
    if t.groups.is_empty() {
        said.push("  no group has a release on the drive: the design's own sheets".into());
    }
    for g in &t.groups {
        said.push(format!("  {}", g.said()));
        if let Some(k) = &g.taken {
            for n in &k.notes {
                said.push(format!("    {n}"));
            }
        }
    }
    for i in &t.ignored {
        said.push(format!("  ignored: {i}"));
    }
    *CHANGED.write().unwrap_or_else(|e| e.into_inner()) = t.changed();
    Ok((t.files, design, said))
}

/// What the releases taken into the design the tool opened last change, node
/// by node, with which release (`today::Today::changed`); empty when the
/// design is not today's.
static CHANGED: std::sync::RwLock<Vec<(String, String)>> = std::sync::RwLock::new(Vec::new());

/// Every node a release taken into the open design changes, and which
/// release: what a closure is traced against when it held before and does
/// not now (`vleo_modules::health::trace_since`).
pub fn changed_in_the_design() -> Vec<(String, String)> {
    CHANGED.read().unwrap_or_else(|e| e.into_inner()).clone()
}

/// The design as the tool opens it: where its files are read from, the design
/// file when it is one, and — when it is today's design — what it was built
/// from, line by line.
type Opened = (std::sync::Arc<dyn Files>, Option<DesignFile>, Vec<String>);

/// The design the tool is given: the folder `VLEO_DESIGN` names, or
/// `design/` where the tool finds it. Where there is neither, there is no
/// design, and the tool says so rather than reading anything in its place.
fn open_base(root: &Path) -> Result<(std::sync::Arc<dyn Files>, Option<DesignFile>), String> {
    let named = std::env::var("VLEO_DESIGN")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from);
    if let Some(dir) = named {
        // A design is a folder of its files; anything else named is refused,
        // never read past to whatever sits at the root.
        if !dir.is_dir() {
            return Err(format!(
                "the design {} is not a folder of the design's files: \
                 VLEO_DESIGN names the folder that holds them, such as design/",
                dir.display()
            ));
        }
        return open_converted(root, &dir);
    }
    if root.join("design").is_dir() {
        return open_converted(root, &root.join("design"));
    }
    Err(format!(
        "there is no design here: {} holds no design/, and VLEO_DESIGN names no folder of \
         the design's files",
        root.display()
    ))
}

/// Whether an application at version `app` can run what the application at
/// `wrote` wrote: every version is major.minor.patch, compared as numbers,
/// part by part, so 0.10 is later than 0.9. A version that is not one is
/// refused by what it says.
pub fn runs_on(wrote: &str, app: &str) -> Result<bool, String> {
    let parse = |v: &str| -> Option<(u64, u64, u64)> {
        let mut it = v.trim().split('.').map(|p| p.parse::<u64>().ok());
        let t = (it.next()??, it.next()??, it.next()??);
        it.next().is_none().then_some(t)
    };
    let need = parse(wrote)
        .ok_or_else(|| format!("'{wrote}' is not an application's version (major.minor.patch)"))?;
    let have = parse(app)
        .ok_or_else(|| format!("'{app}' is not an application's version (major.minor.patch)"))?;
    Ok(have >= need)
}

/// The design as its files (docs/PLAN_1_0.md, phase E): every group, node
/// and case file in `design/`, or in the folder `VLEO_DESIGN` names, read as
/// the folders they were converted from (`vleo_files::convert::Served`).
///
/// It must load as a design, every check the loader makes, made. Its rows run
/// as its files state them: a method in the interpreter, and a stated value
/// as stated.
fn open_converted(
    root: &Path,
    dir: &Path,
) -> Result<(std::sync::Arc<dyn Files>, Option<DesignFile>), String> {
    let (files, fingerprint) = vleo_files::convert::read_folder(dir)
        .map_err(|e| format!("the design folder {} does not open: {e}", dir.display()))?;
    // Each file names the application that wrote it; one written by a newer
    // application than this is refused before anything is read from it, by
    // name: what a newer application wrote may mean what this one does not.
    let ours = env!("CARGO_PKG_VERSION");
    for (path, f) in &files {
        let wrote = f.meta.get("written_by_app").map_or("", String::as_str);
        let version = wrote.strip_prefix("vleo ").unwrap_or(wrote);
        match runs_on(version, ours) {
            Ok(true) => {}
            Ok(false) => {
                return Err(format!(
                    "the design folder {}: {path} was written by vleo {version}, and this is \
                     vleo {ours}: open it with that application, or a newer one",
                    dir.display()
                ))
            }
            Err(e) => {
                return Err(format!(
                    "the design folder {}: {path} names no application that wrote it: {e}",
                    dir.display()
                ))
            }
        }
    }
    let served = vleo_files::convert::Served::new(root, &files)
        .map_err(|e| format!("the design folder {} does not read: {e}", dir.display()))?;
    let tree = vleo_sheet::load::load_all_from(&served, root)
        .map_err(|e| format!("the design folder {} does not load: {e}", dir.display()))?;
    let info = DesignFile {
        file: dir.display().to_string(),
        rows: tree.sheets.len().to_string(),
        fingerprint,
    };
    Ok((std::sync::Arc::new(served), Some(info)))
}

/// Open the design where the tool finds it — its files, as the server opens
/// them — and run the engine on the graph
/// read from its files (docs/PLAN_1_0.md, phase D: every face is re-pointed).
/// Says which graph runs. The command line, Python and the C interface open
/// the design here, so every face runs the same graph the server does.
///
/// A design that does not open or does not load is refused, naming why, and
/// so is no design at all — no design named, no design's files and no folders
/// where the tool looked: the tool holds no design of its own to run instead.
pub fn run_the_design(root: Option<PathBuf>) -> Result<String, String> {
    let root = root.unwrap_or_else(repo_root);
    let (tree, design, today) = open_tree(&root)?;
    let mut said = run_on_the_files(&*tree, &root, design.is_some())?;
    for line in today {
        said.push_str("\n  ");
        said.push_str(&line);
    }
    Ok(said)
}

/// Run the engine on the graph read from `files`, and say which graph runs.
fn run_on_the_files(files: &dyn Files, root: &Path, from_a_file: bool) -> Result<String, String> {
    if !from_a_file && !root.join("layers").is_dir() {
        return Err(format!(
            "no design where the tool looked: no design/ folder and no layers/ under {}, and \
             VLEO_DESIGN names none. Open the tool where the design is, or name it with VLEO_DESIGN",
            root.display()
        ));
    }
    let tree = vleo_sheet::load::load_all_from(files, root)
        .map_err(|e| format!("the design's files do not load: {e}"))?;
    let graph = vleo_modules::opened::graph(&tree)
        .map_err(|e| format!("the design's files do not make a graph: {e}"))?;
    vleo_modules::run_on(graph);
    Ok(format!(
        "the graph read from the design's files — {} rows, {} of them methods run by the interpreter",
        graph.nodes.len(),
        graph
            .nodes
            .iter()
            .zip(graph.run.iter())
            .filter(|(d, r)| {
                d.behaviour == vleo_modules::core_engine::graph::Behaviour::Method && r.is_some()
            })
            .count()
    ))
}

impl Ctx {
    /// The tree, as this copy reads it — every check the loader makes, made.
    fn load(&self) -> Result<vleo_sheet::Tree, vleo_sheet::Error> {
        vleo_sheet::load::load_all_from(&*self.tree, &self.root)
    }
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
    let holds = |p: &Path| {
        p.join("web").is_dir() && p.join("design").is_dir()
    };
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
            refused = store
                .sync(&vleo_data::Source::Shipped(shipped))
                .err()
                .map(String::from);
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
        // The face's type, bundled beside the stylesheet (web/fonts).
        ("GET", p) if p.starts_with("/fonts/") => font(ctx, p.trim_start_matches("/fonts/")),
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
        // Saved results: what runs returned, with the inputs they ran on, kept
        // outside the repository. Viewing one runs nothing.
        ("GET", "/v1/derisk") => ok_json(derisk_json(ctx)),
        ("GET", "/v1/results") => ok_json(results_list()),
        // One well-formed figure of every kind, with illustrative numbers: what
        // a face is built against before the engine sends that kind for a run.
        ("GET", "/v1/figures/samples") => ok_json(figure_samples()),
        // The numbers a figure of the record draws, worked out by the engine
        // from the bundle the page draws the record from (phase 10).
        ("GET", p) if p.starts_with("/v1/figures/solar/") => {
            ok_json(record_figure(
                ctx,
                p.trim_start_matches("/v1/figures/solar/"),
                params,
            ))
        }
        ("GET", "/v1/result") => ok_json(result_json(params)),
        ("GET", "/v1/result.csv") => result_file(params, "csv"),
        ("GET", "/v1/result.sweep.csv") => result_file(params, "sweep"),
        ("GET", "/v1/result.html") => result_file(params, "report"),
        ("POST", "/v1/results/save") => ok_json(result_save(params, ctx)),
        ("POST", "/v1/results/upload") => ok_json(result_upload(params)),
        ("POST", "/v1/results/upload-file") => ok_json(result_upload_file(params)),
        ("POST", "/v1/results/delete") => ok_json(result_delete(params)),
        ("POST", "/v1/results/pin") => ok_json(result_pin(params)),
        ("POST", "/v1/results/as-case") => ok_json(result_as_case(params)),
        ("GET", p) if p.starts_with("/v1/fragment/") => {
            let id = p.trim_start_matches("/v1/fragment/");
            fragment(ctx, id)
        }
        ("GET", p) if p.starts_with("/v1/lesson-form/") => {
            lesson_form_file(ctx, p.trim_start_matches("/v1/lesson-form/"))
        }
        ("GET", p) if p.starts_with("/v1/lesson/") => {
            ok_json(lesson_json(ctx, p.trim_start_matches("/v1/lesson/")))
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

/// The version of the contract between the page and the engine
/// (contract/README.md). The page carries the one it was built for and says
/// so when the engine speaks another; `contract/VERSION` and the page are
/// held to this by the contract tests.
pub const CONTRACT: &str = "1";

fn version_json(ctx: &Ctx) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.str_field("contract", CONTRACT);
    j.str_field("kernel", &short(Vleo::kernel_hash()));
    j.str_field("graph", &short(Vleo::graph_hash()));
    j.str_field("version", env!("CARGO_PKG_VERSION"));
    j.str_field("endpoint", "local-daemon");
    j.num_field("port", ctx.port as f64);
    j.num_field("nodes", nodes().len() as f64);
    // Where this copy reads the design from, and which design that is.
    j.key("design").raw("{");
    match &ctx.design {
        Some(d) => {
            j.str_field("from", "files");
            j.str_field("fingerprint", &d.fingerprint);
        }
        None => {
            j.str_field("from", "folders");
            j.str_field("fingerprint", "");
        }
    }
    j.close_obj();
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
    let n = nodes().len();
    let mut parents: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut readers = vec![0usize; n];
    for (i, d) in nodes().iter().enumerate() {
        for &v in d.inputs {
            let p = vars()[v as usize].producer as usize;
            if p != i {
                parents[i].push(p);
                readers[p] += 1;
            }
        }
    }
    let mut reach = vec![false; n];
    let mut stack: Vec<usize> = (0..n).filter(|&i| nodes()[i].kind == Kind::Kpi).collect();
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
    j.num_field("nodes", nodes().len() as f64);
    j.key("rows").open_arr();
    for (i, d) in nodes().iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        let v = &vars()[i];
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
            j.raw(&vars()[*x as usize].producer.to_string());
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
    for (i, g) in groups().iter().enumerate() {
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
    for (i, (a, b, why)) in relations().iter().enumerate() {
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
        for d in nodes().iter() {
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
    for (i, c) in cases().iter().enumerate() {
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
            j.str_field("id", vars()[*v as usize].id);
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

/// Which inputs an answer was for, as the run response says it: whether it is
/// the declared design, how many inputs the saved case changes, and what the
/// last update did to the case. A saved answer shown again says it the same way.
fn inputs_note(j: &mut Json, params: &str) {
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
    // A QUESTION ALREADY ANSWERED IS SHOWN, NOT ASKED AGAIN — when the face
    // asks for that (`reuse=1`), and never silently: the answer says which
    // saved result it is. The same engine on the same inputs gives the same
    // answer, and everything that decides it is in the question's key.
    if flag(params, "reuse") && !flag(params, "again") {
        let q = question_of(&case, None);
        if let Some((file, saved)) = vleo_modules::results::store::find(&results_dir(), &q) {
            return saved_run_json(params, &file, &saved, &q);
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
            inputs_note(&mut j, params);
            j.key("values").open_arr();
            for (i, v) in r.values.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                let unit = Vleo::find(&v.id)
                    .map(|k| vars()[k as usize].unit)
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
    let def = &nodes()[k as usize];

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
        let id = vars()[v as usize].id;
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
            let ov = vars()[def.outputs[0] as usize].unit;
            j.str_field("unit", ov.symbol());
            j.num_field("factor", ov.si_factor());
            j.key("inputs").open_arr();
            for (i, &v) in def.inputs.iter().enumerate() {
                if i > 0 {
                    j.raw(",");
                }
                let var = &vars()[v as usize];
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
                j.str_field("id", vars()[v as usize].id);
                j.num_field("si", out[i]);
                j.close_obj();
            }
            j.close_arr();
        }
    }
    j.raw("}");
    j.0
}

/// What a sweep request asks: the row, the input moved, the range in SI, and
/// how many points — clamped as the sweep will run it.
fn sweep_spec(params: &str) -> (String, String, f64, f64, usize) {
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
    (node, over, from, to, points)
}

/// Run a sweep: the row's answer at each point across the range, every other
/// input held at the case. Refused points are kept with why. A request the
/// sweep cannot make at all comes back as the wire refusal.
fn run_sweep(params: &str, ctx: &Ctx) -> Result<vleo_modules::results::Sweep, String> {
    let (node, over, from, to, points) = sweep_spec(params);
    if let Some(why) = unsuppliable(&over) {
        return Err(refuse(&over, &why));
    }
    if let Some(refusal) = case_refused(params, ctx) {
        return Err(refusal);
    }
    vleo_modules::results::sweep(&build_case(params, ctx), &node, &over, from, to, points)
        .map_err(|e| failed(e.message()))
}

/// A behaviour sweep. Refused points are recorded with their reason, never
/// dropped — a sweep in which some rows quietly used a substituted value is a
/// sweep whose conclusion is unknown.
///
/// With `reuse=1`, a sweep already saved for exactly this question — same row,
/// range, points, inputs, engine and data — is returned from its result and
/// says so, rather than run again.
fn sweep_json(params: &str, ctx: &Ctx) -> String {
    let (node, over, from, to, points) = sweep_spec(params);
    if flag(params, "reuse") && !flag(params, "again") && case_refused(params, ctx).is_none() {
        let mut case = build_case(params, ctx);
        case.target = node.clone();
        let q = question_of(&case, Some((over.as_str(), from, to, points)));
        if let Some((file, saved)) = vleo_modules::results::store::find(&results_dir(), &q) {
            if let Some(w) = &saved.sweep {
                return sweep_wire(&node, w, Some((&file, &saved, &q)));
            }
        }
    }
    match run_sweep(params, ctx) {
        Ok(w) => sweep_wire(&node, &w, None),
        Err(refusal) => refusal,
    }
}

/// A sweep on the wire — run now, or read from the result it was saved in.
fn sweep_wire(
    node: &str,
    w: &vleo_modules::results::Sweep,
    from_saved: Option<(&str, &vleo_modules::results::Saved, &str)>,
) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("x_id", &w.over);
    j.str_field("y_id", node);
    j.str_field("x_unit", &w.x_unit);
    j.str_field("y_unit", &w.y_unit);
    j.num_field("x_factor", w.x_factor);
    j.num_field("y_factor", w.y_factor);
    j.key("x").open_arr();
    for (i, v) in w.x.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&json::num(*v));
    }
    j.close_arr();
    j.key("y").open_arr();
    for (i, v) in w.y.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw(&json::num(*v));
    }
    j.close_arr();
    j.key("refused").open_arr();
    for (i, (x, why)) in w.refused.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.num_field("x", *x);
        j.str_field("why", why);
        j.close_obj();
    }
    j.close_arr();
    // The same sweep as the engine describes it for drawing: a face draws this
    // rather than its own picture of the points.
    j.key("figure").raw(&vleo_modules::figure::json(
        &vleo_modules::figure::from_sweep(node, w, None),
    ));
    if let Some((file, s, q)) = from_saved {
        from_saved_json(&mut j, file, s, q);
    }
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
        Some(a) => vars()[a as usize].producer,
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
    let mut cons: Vec<Vec<u16>> = vec![Vec::new(); nodes().len()];
    for (n, d) in nodes().iter().enumerate() {
        for x in d.inputs {
            cons[vars()[*x as usize].producer as usize].push(n as u16);
        }
    }
    let walk = |from: u16, edges: &Vec<Vec<u16>>| -> Vec<bool> {
        let mut seen = vec![false; nodes().len()];
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
    let mut prod: Vec<Vec<u16>> = vec![Vec::new(); nodes().len()];
    for (n, d) in nodes().iter().enumerate() {
        for x in d.inputs {
            prod[n].push(vars()[*x as usize].producer);
        }
    }
    let live = |n: u16| nodes()[n as usize].state == vleo_core::graph::State::Published;

    let down = walk(ni, &cons);
    let mut cand: Vec<(u16, usize)> = Vec::new();
    for n in 0..nodes().len() as u16 {
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
        let mut v = vec![false; nodes().len()];
        for (n, _) in &cand {
            v[*n as usize] = true;
        }
        v
    };
    let mut out: Vec<(u16, usize)> = cand
        .iter()
        .filter(|(n, _)| {
            let up = walk(*n, &cons);
            !(0..nodes().len()).any(|k| up[k] && is_cand[k])
        })
        .cloned()
        .collect();
    // Biggest first: the branch that covers most of the design is the one a
    // reader wants at the top.
    out.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| nodes()[a.0 as usize].id.cmp(nodes()[b.0 as usize].id))
    });

    // How many rows read this one at all, so a face can say whether an empty
    // list means "nothing reads it" or "everything that does is unfinished".
    let read_by = (0..nodes().len()).filter(|k| down[*k]).count();
    j.num_field("read_by", read_by as f64);

    j.key("branches").open_arr();
    for (i, (n, size)) in out.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        let d = &nodes()[*n as usize];
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

    let (base, out) = levers_of(params, ctx, &node, ni);
    match base {
        Some(b) => j.num_field("base", b),
        None => j.key("base").raw("null"),
    };

    j.key("levers").open_arr();
    for (i, lev) in out.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        let d = &vars()[lev.var as usize];
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

/// One decision and what moving it across its own declared range does to
/// the answer. `span` is negative where an end could not be evaluated at
/// all, which is a different fact from a span of zero.
pub(crate) struct Lever {
    pub(crate) span: f64,
    pub(crate) var: u16,
    pub(crate) at_lower: Option<f64>,
    pub(crate) at_upper: Option<f64>,
    pub(crate) why: String,
}

/// Every declared decision upstream of `node` and how far moving it across its
/// own range moves the answer, most first — and the answer on the case, which
/// a span is relative to. What `/v1/levers` sends, and what a figure choosing
/// its own axis reads.
pub(crate) fn levers_of(params: &str, ctx: &Ctx, node: &str, ni: u16) -> (Option<f64>, Vec<Lever>) {
    let node = node.to_string();
    // Every declared row upstream, however far: a decision three rows away is
    // still a decision, and the reason a reader opens this row may be a choice
    // taken well before it.
    // The walk is over NODES, because `inputs` is a node's declaration of what
    // it reads. A candidate is a VARIABLE, because that is what a sweep
    // supplies: the two indices are different spaces and conflating them is
    // the defect this file has already shipped once.
    let start = vars()[ni as usize].producer;
    let mut seen = vec![false; nodes().len()];
    let mut stack = vec![start];
    seen[start as usize] = true;
    let mut cands: Vec<u16> = Vec::new();
    while let Some(n) = stack.pop() {
        for x in nodes()[n as usize].inputs {
            let pv = &vars()[*x as usize];
            let pn = pv.producer;
            if !seen[pn as usize] {
                seen[pn as usize] = true;
                stack.push(pn);
            }
            if *x != ni
                && nodes()[pn as usize].kind == vleo_core::graph::Kind::Declared
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
    let mut out: Vec<Lever> = Vec::new();
    for v in cands {
        let d = &vars()[v as usize];
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
            .then_with(|| vars()[a.var as usize].id.cmp(vars()[b.var as usize].id))
    });

    (base, out)
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
        // The rows are the design's: its files in design/, opened once.
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            crate::run_the_design(Some(crate::repo_root())).expect("the design opens");
        });
        let id = vleo_modules::nodes()[0].id;
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
