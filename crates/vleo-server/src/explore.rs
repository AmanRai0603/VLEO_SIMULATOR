//! What the Explore workspace asks of the engine (docs/PLAN_F.md, stage F2):
//! which design is open and what built it, the health map, a closure traced to
//! its cause, and two files compared.
//!
//! Each of these already exists in the library: today's design is built in
//! `today`, the map and the trace in `vleo_modules::health`, the comparison in
//! `vleo_files::compare`. Until now only the console and the command line
//! could reach them. These routes are how a screen does, and they decide
//! nothing of their own: every state, cause and difference is the library's,
//! written as it says it.

use std::path::{Component, Path, PathBuf};

use vleo_bus::{Case, RunMode};
use vleo_modules::health::{health, is_closure, range, trace_since, Lever};

use super::http::{decode, param};
use super::json::Json;
use super::{build_case, case_refused, Ctx};

/// `GET /v1/today` — which design the tool has open, and, when it is today's
/// design, the drive it was built from: each group's release, why any newer
/// one was refused, and what on the drive was not taken.
///
/// A design that is not today's says so (`"today": false`): a screen showing
/// the design's own files must never look as if it showed the drive's.
pub(super) fn today_json(ctx: &Ctx) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    let built = super::built_from();
    j.bool_field("today", built.is_some());
    j.key("design").raw("{");
    match &ctx.design {
        Some(d) => {
            j.str_field("from", &d.file);
            j.num_field("rows", d.rows.parse().unwrap_or(0.0));
            j.str_field("fingerprint", &d.fingerprint);
        }
        None => {
            j.str_field("from", "");
            j.num_field("rows", 0.0);
            j.str_field("fingerprint", "");
        }
    }
    j.close_obj();
    match &built {
        Some(r) => j.str_field("drive", &r.drive),
        None => j.str_field("drive", ""),
    };
    j.key("groups").open_arr();
    for g in built.iter().flat_map(|r| r.groups.iter()) {
        j.open_obj();
        j.str_field("id", &g.id);
        j.str_field("said", &g.said());
        match &g.taken {
            Some(t) => {
                j.key("taken").raw("{");
                j.str_field("version", &t.version);
                j.str_field("sealed", &t.sealed);
                j.str_field("sealed_by", &t.sealed_by);
                j.str_field("fingerprint", &t.fingerprint);
                strings(&mut j, "changes", &t.changes);
                strings(&mut j, "notes", &t.notes);
                j.close_obj();
            }
            None => {
                j.key("taken").raw("null");
            }
        }
        j.key("refused").open_arr();
        for (version, why) in &g.refused {
            j.open_obj();
            j.str_field("version", version);
            j.str_field("why", why);
            j.close_obj();
        }
        j.close_arr();
        j.close_obj();
    }
    j.close_arr();
    let ignored: Vec<String> = built.map(|r| r.ignored).unwrap_or_default();
    strings(&mut j, "ignored", &ignored);
    j.raw("}");
    j.0
}

/// The case the health map is drawn on: the request's, as every run takes it,
/// over the whole design.
fn whole_design(params: &str, ctx: &Ctx) -> Case {
    let g = vleo_modules::engine();
    let mut case = build_case(params, ctx);
    case.target = g.nodes.first().map_or("", |n| n.id).to_string();
    case.mode = RunMode::All;
    case
}

/// `GET /v1/health` — the health map of the design that is open, on the case
/// the request carries: every node's state and why, every group under the
/// programme's as bad as its worst node, and the spacecraft as bad as its
/// worst group (docs/OPERATING_1_0.md, section 6).
pub(super) fn health_json(params: &str, ctx: &Ctx) -> String {
    if let Some(refusal) = case_refused(params, ctx) {
        return refusal;
    }
    let g = vleo_modules::engine();
    let map = health(g, &whole_design(params, ctx));
    let id = |k: u16| g.nodes[k as usize].id;
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("spacecraft", map.spacecraft.name());
    j.key("counts").raw("{");
    for (s, n) in map.counts() {
        j.num_field(s.name(), n as f64);
    }
    j.close_obj();
    j.key("groups").open_arr();
    for gr in &map.groups {
        let def = vleo_modules::groups().iter().find(|x| x.id == gr.id);
        j.open_obj();
        j.str_field("id", gr.id);
        j.str_field("label", def.map_or("", |d| d.label));
        j.str_field("parent", def.map_or("", |d| d.parent));
        j.str_field("owner", def.map_or("", |d| d.owner));
        j.str_field("state", gr.state.name());
        j.key("worst").open_arr();
        for (i, &k) in gr.worst.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.push_string(id(k));
        }
        j.close_arr();
        j.close_obj();
    }
    j.close_arr();
    j.key("nodes").open_arr();
    for n in &map.nodes {
        let def = &g.nodes[n.node as usize];
        j.open_obj();
        j.str_field("id", def.id);
        j.str_field("parent", def.parent);
        j.str_field("state", n.state.name());
        j.str_field("why", &n.why);
        j.bool_field("closure", is_closure(g, n.node));
        match n.blocked_by {
            Some(b) => j.str_field("blocked_by", id(b)),
            None => j.key("blocked_by").raw("null"),
        };
        match n.margin {
            Some(m) => j.num_field("margin", m),
            None => j.key("margin").raw("null"),
        };
        j.close_obj();
    }
    j.close_arr();
    strings(&mut j, "notes", &map.notes);
    j.raw("}");
    j.0
}

/// `GET /v1/health/trace?closure=<id>` — one closure walked down to what
/// causes it: the nodes, nearest first, each with its group and owner and,
/// for a node a release taken into today's design changed, which release; the
/// declared inputs that move its margin, most first; and its verdict over the
/// values still open behind it.
pub(super) fn trace_json(params: &str, ctx: &Ctx) -> String {
    if let Some(refusal) = case_refused(params, ctx) {
        return refusal;
    }
    let g = vleo_modules::engine();
    let asked = param(params, "closure").map(decode).unwrap_or_default();
    let Some(k) = g.find(&asked) else {
        return refuse(
            &asked,
            &format!("no node '{asked}': the trace starts at a closure the design holds"),
        );
    };
    if !is_closure(g, k) {
        return refuse(
            &asked,
            &format!(
                "{asked} is not a closure: a closure is an achieved or KPI row, whose answer \
                 is its own margin"
            ),
        );
    }
    let case = whole_design(params, ctx);
    let map = health(g, &case);
    let changed: Vec<(u16, String)> = super::changed_in_the_design()
        .into_iter()
        .filter_map(|(id, said)| g.find(&id).map(|k| (k, said)))
        .collect();
    let t = trace_since(g, &case, &map, k, &changed);
    let rv = range(g, &case, &map, k);
    let id = |k: u16| g.nodes[k as usize].id;

    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("closure", &asked);
    j.str_field("state", t.state.name());
    j.str_field("why", &map.node(k).why);
    match t.margin {
        Some(m) => j.num_field("margin", m),
        None => j.key("margin").raw("null"),
    };
    j.key("causes").open_arr();
    for c in &t.causes {
        j.open_obj();
        j.str_field("id", id(c.node));
        j.str_field("state", c.state.name());
        j.str_field("why", &c.why);
        j.str_field("group", c.group);
        j.str_field("owner", c.owner);
        j.close_obj();
    }
    j.close_arr();
    j.key("levers").open_arr();
    for l in &t.levers {
        lever(&mut j, g, l);
    }
    j.close_arr();
    j.key("range").raw("{");
    j.str_field("verdict", verdict_name(rv.verdict));
    j.str_field("said", rv.verdict.said());
    j.key("tornado").open_arr();
    for l in &rv.tornado {
        lever(&mut j, g, l);
    }
    j.close_arr();
    j.str_field("least_mature", rv.least_mature.0.name());
    j.key("least_mature_rows").open_arr();
    for (i, &r) in rv.least_mature.1.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.push_string(id(r));
    }
    j.close_arr();
    j.close_obj();
    j.raw("}");
    j.0
}

fn verdict_name(v: vleo_modules::health::Verdict) -> &'static str {
    use vleo_modules::health::Verdict;
    match v {
        Verdict::NoOpenValue => "no-open-value",
        Verdict::ClosesForAll => "closes-for-all",
        Verdict::ClosesForPart => "closes-for-part",
        Verdict::FailsForAll => "fails-for-all",
    }
}

/// One declared input and what it does to a closure's margin across its range.
fn lever(j: &mut Json, g: &vleo_modules::Graph, l: &Lever) {
    let var = &g.vars[g.nodes[l.node as usize].outputs[0] as usize];
    j.open_obj();
    j.str_field("id", g.nodes[l.node as usize].id);
    j.str_field("unit", var.unit.symbol());
    j.num_field("factor", var.unit.si_factor());
    j.num_field("lower", var.limit.lower);
    j.num_field("upper", var.limit.upper);
    for (k, v) in [
        ("at_lower", l.at_lower),
        ("at_upper", l.at_upper),
        ("closes_at", l.closes_at),
    ] {
        match v {
            Some(x) => j.num_field(k, x),
            None => j.key(k).raw("null"),
        };
    }
    j.str_field("group", l.group);
    j.str_field("owner", l.owner);
    j.close_obj();
}

/// `GET /v1/compare?a=<file>&b=<file>` — two files of the design compared,
/// block by block (`vleo_files::compare`), in the words the group page uses
/// for it. Each file is named under where the design is read from,
/// `design:<path>`, or under the drive today's design was built from,
/// `drive:<path>`; nothing else on the computer is opened.
pub(super) fn compare_json(params: &str, ctx: &Ctx) -> String {
    let a = param(params, "a").map(decode).unwrap_or_default();
    let b = param(params, "b").map(decode).unwrap_or_default();
    let opened = |named: &str| -> Result<vleo_files::model::File, String> {
        let path = place(named, ctx)?;
        let how = vleo_files::upgrade::Upgrade {
            app: concat!("vleo ", env!("CARGO_PKG_VERSION")),
            at: "",
        };
        vleo_files::sqlite::open(&path, &how)
            .map(|o| o.file)
            .map_err(|e| format!("{named} does not open as a file of the design: {e}"))
    };
    let (fa, fb) = match (opened(&a), opened(&b)) {
        (Ok(fa), Ok(fb)) => (fa, fb),
        (Err(why), _) => return refuse(&a, &why),
        (_, Err(why)) => return refuse(&b, &why),
    };
    let c = vleo_files::compare::compare(&fa, &fb);
    let says = |j: &mut Json, k: &str, ds: &[vleo_files::compare::Difference]| {
        let said: Vec<String> = ds.iter().map(|d| d.says()).collect();
        strings(j, k, &said);
    };
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("a", &a);
    j.str_field("b", &b);
    j.str_field("summary", &c.summary());
    j.bool_field("same_content", c.is_same());
    j.num_field("same", c.same as f64);
    says(&mut j, "file", &c.file);
    j.key("blocks").open_arr();
    for blk in &c.blocks {
        j.open_obj();
        j.str_field("uid", &blk.uid);
        j.str_field("id", blk.id());
        for (k, v) in [("before", &blk.before), ("after", &blk.after)] {
            match v {
                Some(s) => j.str_field(k, s),
                None => j.key(k).raw("null"),
            };
        }
        j.str_field("status", blk.status.name());
        says(&mut j, "differences", &blk.differences);
        j.close_obj();
    }
    j.close_arr();
    j.key("history").open_arr();
    for (t, x, y) in &c.history {
        j.open_obj();
        j.str_field("table", t);
        j.num_field("only_first", *x as f64);
        j.num_field("only_second", *y as f64);
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}

/// Where a file named in a request is: `design:<path>` under the design the
/// tool reads, `drive:<path>` under the drive today's design was built from.
/// A path that climbs out of either, or names neither, is refused by name.
fn place(named: &str, ctx: &Ctx) -> Result<PathBuf, String> {
    let (under, rest) =
        match named.split_once(':') {
            Some(("design", rest)) => match &ctx.design {
                Some(d) => (PathBuf::from(&d.file), rest),
                None => return Err("no design is open to name a file in".into()),
            },
            Some(("drive", rest)) => match super::built_from() {
                Some(r) => (PathBuf::from(r.drive), rest),
                None => return Err(
                    "the design open is not today's design: there is no drive to name a file on"
                        .into(),
                ),
            },
            _ => {
                return Err(format!(
                    "'{named}' names no file: write design:<path> or drive:<path>"
                ))
            }
        };
    let rel = Path::new(rest);
    if rest.is_empty() || !rel.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(format!(
            "'{named}' is not a path inside the design or the drive"
        ));
    }
    let path = under.join(rel);
    if !path.is_file() {
        return Err(format!("{named}: there is no such file"));
    }
    Ok(path)
}

fn strings(j: &mut Json, k: &str, items: &[String]) {
    j.key(k).open_arr();
    for (i, s) in items.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.push_string(s);
    }
    j.close_arr();
}

fn refuse(node: &str, why: &str) -> String {
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", false);
    j.str_field("node", node);
    j.str_field("message", why);
    j.raw("}");
    j.0
}
