//! What the browser reads: files, the manual, each node's page and fragment,
//! the index, the version, a node's form and its check, and the de-risking register.

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
pub(crate) fn bundle_file(ctx: &Ctx, rest: &str) -> (&'static str, &'static str, Vec<u8>) {
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

pub(crate) fn file(
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
pub(crate) fn parity_file(ctx: &Ctx, name: &str) -> (&'static str, &'static str, Vec<u8>) {
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
pub(crate) fn module(ctx: &Ctx, name: &str) -> (&'static str, &'static str, Vec<u8>) {
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
pub(crate) fn manual_endpoint(ctx: &Ctx) -> (&'static str, &'static str, Vec<u8>) {
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
pub(crate) fn node_endpoint(ctx: &Ctx, id: &str) -> (&'static str, &'static str, Vec<u8>) {
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
pub(crate) fn fragment(ctx: &Ctx, id: &str) -> (&'static str, &'static str, Vec<u8>) {
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

pub(crate) fn version_json(ctx: &Ctx) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.str_field("kernel", &short(Vleo::kernel_hash()));
    j.str_field("graph", &short(Vleo::graph_hash()));
    j.str_field("version", env!("CARGO_PKG_VERSION"));
    j.str_field("endpoint", "local-daemon");
    // The desktop app: the page offers Quit and says it is still open.
    j.bool_field("app", app::APP.load(std::sync::atomic::Ordering::Relaxed));
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
pub(crate) fn reach_and_readers() -> (Vec<bool>, Vec<usize>) {
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

pub(crate) fn index_json() -> String {
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

/// One node's form, as a file to download.
pub(crate) fn form_file(ctx: &Ctx, id: &str) -> (&'static str, &'static str, Vec<u8>) {
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

/// Why the design is what it is: every registered risk, where it stands and
/// every node version that moved it — the conclusion the risk-register rows
/// draw — and the version each node is at.
///
/// Read from the sheets on each request rather than held: the daemon never
/// writes a sheet, and a developer applying forms beside a running copy
/// should see the register move without a restart.
pub(crate) fn derisk_json(ctx: &Ctx) -> String {
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

/// What a filled form would change in its node. Writes nothing.
pub(crate) fn form_check(ctx: &Ctx, params: &str) -> String {
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
    // Why it is changing: the decisions the form moves, whether it becomes a
    // version, and the record it carries — so a filler sees, before sending,
    // what the developers will see.
    j.str_field("about", &p.about.join(", "));
    j.num_field("version", p.version.map(f64::from).unwrap_or(0.0));
    j.key("derisk").open_arr();
    let mut first = true;
    for (k, ask, _) in vleo_sheet::template::DERISK {
        let v = f.derisk.get(k);
        if v.trim().is_empty() {
            continue;
        }
        if !first {
            j.raw(",");
        }
        first = false;
        j.raw("{");
        j.str_field("ask", ask);
        j.str_field("said", v);
        j.close_obj();
    }
    j.close_arr();
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
