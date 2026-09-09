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
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use vleo_bus::{Case, RunMode};
use vleo_modules::{tables, Scratch, Vleo, GROUPS, NODES, RELATIONS, VARS};

fn main() {
    let port_pref: u16 = std::env::var("VLEO_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(7777);
    let root = repo_root();
    let (data, data_versions) = resolve_data(&root);

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

    let ctx = Ctx {
        root,
        data,
        data_versions,
        port,
    };
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                if let Err(e) = serve(s, &ctx) {
                    eprintln!("vleo-daemon: {e}");
                }
            }
            Err(e) => eprintln!("vleo-daemon: accept: {e}"),
        }
    }
}

struct Ctx {
    root: PathBuf,
    data: Vec<String>,
    data_versions: Vec<String>,
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

fn resolve_data(root: &Path) -> (Vec<String>, Vec<String>) {
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
    (store.verified_names(), store.versions())
}

// ---------------------------------------------------------------------------

fn serve(mut stream: TcpStream, ctx: &Ctx) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    reader.read_line(&mut request)?;
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

    let (status, ctype, payload) = route(&method, &path, &params, ctx);
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
        ("GET", "/app.js") => file(ctx, "app.js", "application/javascript; charset=utf-8"),
        ("GET", "/v1/version") => ok_json(version_json(ctx)),
        ("GET", "/v1/index") => ok_json(index_json()),
        ("GET", p) if p.starts_with("/v1/fragment/") => {
            let id = p.trim_start_matches("/v1/fragment/");
            fragment(ctx, id)
        }
        ("POST", "/v1/run") | ("GET", "/v1/run") => ok_json(run_json(params, ctx)),
        ("GET", "/v1/sweep") => ok_json(sweep_json(params, ctx)),
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
    let def = &NODES[i as usize];
    let folder = if def.id.starts_with("kpi_") {
        def.id.to_string()
    } else {
        def.id
            .split_once('_')
            .map(|x| x.1)
            .unwrap_or(def.id)
            .to_string()
    };
    let crate_name = crate_for(def.subsystem, def.id);
    let p = ctx
        .root
        .join("crates")
        .join(crate_name)
        .join("nodes")
        .join(&folder)
        .join("page.html");
    match std::fs::read(&p) {
        Ok(b) => ("200 OK", "text/html; charset=utf-8", b),
        Err(_) => (
            "404 Not Found",
            "text/html; charset=utf-8",
            format!("<p class=\"empty\">No fragment for <code>{id}</code>. Run <code>cargo xtask docs</code>.</p>").into_bytes(),
        ),
    }
}

fn crate_for(subsystem: &str, id: &str) -> &'static str {
    if id.starts_with("pwr_") {
        return "vleo-mod-power";
    }
    match subsystem {
        "env" => "vleo-mod-env",
        "orbit" => "vleo-mod-orbit",
        "aero" => "vleo-mod-aero",
        "prop" => "vleo-mod-prop",
        "power" => "vleo-mod-power",
        "thm" => "vleo-mod-thermal",
        "gnc" => "vleo-mod-gnc",
        "com" => "vleo-mod-comms",
        "pay" => "vleo-mod-payload",
        "mass" => "vleo-mod-mass",
        "mis" | "kpi" => "vleo-mod-mission",
        _ => "vleo-mod-cost",
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
fn index_json() -> String {
    let mut j = Json::new();
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
        j.str_field("sub", d.subsystem);
        j.str_field("kind", d.kind.name());
        j.str_field("state", d.state.name());
        j.str_field("owner", d.owner);
        j.str_field("tier", d.tier.name());
        j.str_field("unit", v.unit.symbol());
        j.str_field("symbol", v.symbol);
        j.str_field("question", d.question);
        j.num_field("lo", v.limit.lower);
        j.num_field("hi", v.limit.upper);
        j.num_field("fixtures", d.fixtures.len() as f64);
        j.key("in").open_arr();
        for (k, x) in d.inputs.iter().enumerate() {
            if k > 0 {
                j.raw(",");
            }
            j.raw(&x.to_string());
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
    let b = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < b.len() => {
                let h = u8::from_str_radix(&s[i + 1..i + 3], 16).unwrap_or(b'?');
                out.push(h as char);
                i += 3;
            }
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    out
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

fn build_case(params: &str, ctx: &Ctx) -> Case {
    Case {
        base: param(params, "case")
            .map(decode)
            .unwrap_or_else(|| "nominal".into()),
        supply: sets(params),
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
    let case = build_case(params, ctx);
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
