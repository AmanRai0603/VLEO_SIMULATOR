//! The files the page is made of: the shell, node pages and fragments, the manual and the lesson form.

use super::*;

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
pub(super) fn bundle_file(ctx: &Ctx, rest: &str) -> (&'static str, &'static str, Vec<u8>) {
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

pub(super) fn file(
    ctx: &Ctx,
    name: &str,
    ctype: &'static str,
) -> (&'static str, &'static str, Vec<u8>) {
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
pub(super) fn parity_file(ctx: &Ctx, name: &str) -> (&'static str, &'static str, Vec<u8>) {
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
pub(super) fn module(ctx: &Ctx, name: &str) -> (&'static str, &'static str, Vec<u8>) {
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

/// One of the face's fonts, by name. The same guard as `module`: a plain
/// `.woff2` name under `web/fonts`, or refused before the filesystem.
pub(super) fn font(ctx: &Ctx, name: &str) -> (&'static str, &'static str, Vec<u8>) {
    let stem = name.strip_suffix(".woff2").unwrap_or("");
    let ok = !stem.is_empty()
        && stem
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !ok {
        return (
            "400 Bad Request",
            "text/plain; charset=utf-8",
            b"not a font name".to_vec(),
        );
    }
    match std::fs::read(ctx.root.join("web").join("fonts").join(name)) {
        Ok(b) => ("200 OK", "font/woff2", b),
        Err(_) => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            format!("web/fonts/{name} is not on disk").into_bytes(),
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
pub(super) fn manual_endpoint(ctx: &Ctx) -> (&'static str, &'static str, Vec<u8>) {
    const JSON: &str = "application/json; charset=utf-8";
    let m = match vleo_sheet::manual::load(&ctx.root) {
        Ok(m) => m,
        // A manual that does not load is reported, not replaced by an empty
        // one: an empty manual reads as "there is nothing to know".
        Err(e) => {
            return (
                "500 Internal Server Error",
                JSON,
                format!("{{\"ok\":false,\"message\":{}}}", json::string(e.message())).into_bytes(),
            )
        }
    };
    use vleo_sheet::form;
    let case = saved_case();
    let (rows, published, seeded) = match ctx.load() {
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
pub(super) fn node_endpoint(ctx: &Ctx, id: &str) -> (&'static str, &'static str, Vec<u8>) {
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
    let def = &nodes()[i as usize];
    let dir = ctx.root.join(def.folder);

    // The seven files, and who writes each. This list is the template: it is
    // the same seven for every folder in the tree, which is what makes
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
        let len = ctx.tree.len(&dir.join(name));
        j.raw("{");
        j.str_field("name", name);
        j.str_field("written", who);
        j.str_field("what", what);
        j.bool_field("present", len.is_some());
        j.num_field("bytes", len.unwrap_or(0) as f64);
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
pub(super) fn fragment(ctx: &Ctx, id: &str) -> (&'static str, &'static str, Vec<u8>) {
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
    // RENDERED, NOT READ. A node's page is written from its sheet when it is
    // opened — from the design's files or the folders alike — so there is no
    // committed copy to fall behind the sheet it describes.
    let def = &nodes()[i as usize];
    let tree = match ctx.load() {
        Ok(t) => t,
        Err(e) => {
            return (
                "500 Internal Server Error",
                "text/html; charset=utf-8",
                format!("<p class=\"empty\">The tree does not load: {e}</p>").into_bytes(),
            )
        }
    };
    match tree.sheets.get(def.id) {
        Some(sh) => (
            "200 OK",
            "text/html; charset=utf-8",
            vleo_sheet::page::fragment(sh, &tree).into_bytes(),
        ),
        None => (
            "404 Not Found",
            "text/html; charset=utf-8",
            format!("<p class=\"empty\">No sheet for <code>{id}</code>.</p>").into_bytes(),
        ),
    }
}

// ---------------------------------------------------------------------------
// saved results

/// A row's lesson, when it has one: `lesson` is null for a row with none.
///
/// Checked here as the gate checks it, against the tree on disk, so a lesson
/// edited beside a running copy is refused with its reasons rather than drawn
/// half-right. Where the tree cannot be read — a kit without the sheets — the
/// lesson is served as the release checked it, and `checked` says so.
pub(super) fn lesson_json(ctx: &Ctx, id: &str) -> String {
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return failed("not a node identifier");
    }
    let Some(i) = Vleo::find(id) else {
        return failed("no such node");
    };
    let dir = ctx.root.join(nodes()[i as usize].folder);
    let l = match vleo_sheet::lesson::load_from(&*ctx.tree, &dir, id) {
        None => return "{\"ok\":true,\"checked\":true,\"lesson\":null}".to_string(),
        Some(Err(e)) => return failed(e.message()),
        Some(Ok(l)) => l,
    };
    let checked = match ctx.load() {
        Ok(tree) => {
            let bad = vleo_sheet::lesson::problems(&l, &tree);
            if !bad.is_empty() {
                return failed(&format!(
                    "{id}'s lesson does not pass its check: {}",
                    bad.join("; ")
                ));
            }
            true
        }
        Err(_) => false,
    };
    format!(
        "{{\"ok\":true,\"checked\":{checked},\"lesson\":{}}}",
        vleo_sheet::lesson::json(&l)
    )
}

/// A row's lesson form: one HTML file to fill anywhere, as `xtask lesson form`
/// writes it. Reading it writes nothing; a filled one is applied by a
/// developer, with `xtask lesson apply`.
pub(super) fn lesson_form_file(ctx: &Ctx, id: &str) -> (&'static str, &'static str, Vec<u8>) {
    let id = decode(id);
    let text = |status, body: String| (status, "text/plain; charset=utf-8", body.into_bytes());
    let tree = match ctx.load() {
        Ok(t) => t,
        Err(e) => {
            return text(
                "500 Internal Server Error",
                format!("the tree does not load: {e}"),
            )
        }
    };
    match tree.sheets.get(&id) {
        None => text("404 Not Found", format!("no row '{id}'")),
        Some(sh) => match vleo_sheet::lesson_form::document(sh, &tree) {
            Ok(html) => ("200 OK", "text/html; charset=utf-8", html.into_bytes()),
            Err(e) => text("500 Internal Server Error", e.into()),
        },
    }
}
