//! An issue raised from a red mark (docs/briefs/F2.md, item 4; W11, its first
//! half).
//!
//! `POST /v1/issues/raise?node=<id>&what=<words>` writes one issue file,
//! `issues/<number>-<group>.vissue`, into the person's working copy: addressed
//! to the group the node is in and to the node, signed by whoever's key is
//! open, and carrying the evidence. The evidence is the engine's, never the
//! page's: the node's state and why on the case the request carries, and, for
//! a closure, what causes it, nearest first; with the design it was read on,
//! by its fingerprint, and the engine that read it. The person's own words
//! say what is wrong.
//!
//! A file is never written over: the number is one more than the highest in
//! the copy's `issues/`, chosen with the copy held, and a file already at that
//! name stops the write. Closing an issue is not here (F5: by the release that
//! names it).

use super::*;
use vleo_files::chain::{self, FILE};
use vleo_files::meta::Kind;
use vleo_files::model::{File, Issue};
use vleo_files::sqlite;
use vleo_modules::health::{health, is_closure, trace_since};

/// `POST /v1/issues/raise`: the issue written, and where.
pub(super) fn raise(params: &str, ctx: &Ctx) -> String {
    match raised(params, ctx) {
        Ok((file, number, group, addressed_to)) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("file", &file);
            j.num_field("number", number as f64);
            j.str_field("group", &group);
            j.str_field("addressed_to", &addressed_to);
            j.str_field(
                "message",
                &format!(
                    "issue {number} is raised on {group}, addressed to {addressed_to}, in your copy as {file}. \
                     It goes to the drive when you put your work back."
                ),
            );
            j.raw("}");
            j.0
        }
        Err(why) => me::refusal(&why),
    }
}

/// The highest number an issue in `dir` carries in its name, `<number>-…`.
fn highest(dir: &Path) -> u64 {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let stem = name.strip_suffix(".vissue")?;
            stem.split_once('-')?.0.parse::<u64>().ok()
        })
        .max()
        .unwrap_or(0)
}

/// What the engine says of `node` on the case `params` carries, as the
/// evidence an issue keeps, and the group and owner it is addressed to.
fn evidence(params: &str, ctx: &Ctx, node: &str) -> Result<(String, &'static str, String), String> {
    if let Some(refusal) = case_refused(params, ctx) {
        return Err(format!("the case does not run: {refusal}"));
    }
    let g = vleo_modules::engine();
    let k = g.find(node).ok_or_else(|| {
        format!("no node '{node}': an issue is raised on a node the design holds")
    })?;
    let def = &g.nodes[k as usize];
    let mut case = build_case(params, ctx);
    case.target = g.nodes.first().map_or("", |n| n.id).to_string();
    case.mode = vleo_bus::RunMode::All;
    let map = health(g, &case);
    let n = map.node(k);
    let (which, fingerprint) = open_design(ctx);
    let (kernel, graph) = vleo_modules::results::engine();
    let mut lines = vec![
        format!("on {which}, fingerprint {fingerprint}"),
        format!("by the engine kernel {kernel}, graph {graph}"),
        format!(
            "{node} {}: {}{}{}",
            n.state.name(),
            n.why,
            n.margin
                .map(|m| format!("; margin {:+.1}%", m * 100.0))
                .unwrap_or_default(),
            n.blocked_by
                .map(|b| format!("; waits on {}", g.nodes[b as usize].id))
                .unwrap_or_default()
        ),
    ];
    let set: Vec<String> = case
        .supply
        .iter()
        .map(|(id, v)| format!("{id} = {v}"))
        .collect();
    lines.push(if set.is_empty() {
        "on the case with every input at its default".into()
    } else {
        format!("on the case with {}", set.join(", "))
    });
    if is_closure(g, k) {
        let changed: Vec<(u16, String)> = changed_in_the_design()
            .into_iter()
            .filter_map(|(id, said)| g.find(&id).map(|k| (k, said)))
            .collect();
        let t = trace_since(g, &case, &map, k, &changed);
        if t.causes.is_empty() {
            lines.push(
                "traced: no row it reads is refused, open or unproven, and no release changed one"
                    .into(),
            );
        }
        for c in &t.causes {
            lines.push(format!(
                "caused by {} ({}, {} · {}): {}",
                g.nodes[c.node as usize].id,
                c.state.name(),
                c.group,
                c.owner,
                c.why
            ));
        }
    }
    // The group the design holds the node in: a subsystem's folder on the
    // drive for a node of the subsystem layer, the system's or the
    // management's group above it.
    let group = def.parent;
    let owner = if def.owner.is_empty() {
        g.groups
            .iter()
            .find(|x| x.id == group)
            .map_or("", |x| x.owner)
    } else {
        def.owner
    };
    let addressed_to = if owner.is_empty() {
        format!("{group} and its node {node}")
    } else {
        format!("{group} ({owner}) and its node {node}")
    };
    Ok((lines.join("\n"), group, addressed_to))
}

fn raised(params: &str, ctx: &Ctx) -> Result<(String, u64, String, String), String> {
    me::carries_session(params)?;
    let node = param(params, "node").map(decode).unwrap_or_default();
    let what = param(params, "what").map(decode).unwrap_or_default();
    if what.trim().is_empty() {
        return Err("say what is wrong: an issue with no words is one nobody can act on".into());
    }
    let copy = copy::dir(ctx);
    if copy::record(&copy)?.is_none() {
        return Err(
            "an issue is written into your copy of the database: copy it from the drive first"
                .into(),
        );
    }
    let (evidence, group, addressed_to) = evidence(params, ctx, &node)?;
    // One program at a time writes the copy, and the number is chosen with it
    // held, so two issues raised here never take the same one.
    let _held = copy::lock(&copy)?;
    let dir = copy.join("issues");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let number = highest(&dir) + 1;
    let name = format!("{number}-{group}.vissue");
    let path = dir.join(&name);
    if path.exists() {
        return Err(format!(
            "{} is already there, so nothing is written over it: raise the issue again",
            path.display()
        ));
    }
    let at = results::now_utc();
    let file = me::with_key(|key, me| {
        let mut f = File::new(Kind::Issue, concat!("vleo ", env!("CARGO_PKG_VERSION")));
        for (k, v) in [
            ("number", number.to_string()),
            ("group_id", group.to_string()),
            ("writer", me.name.clone()),
        ] {
            f.meta.insert(k.to_string(), v);
        }
        f.issues.push(Issue {
            number: number as i64,
            group_id: group.to_string(),
            raised_by: me.name.clone(),
            at: at.clone(),
            place: node.clone(),
            what: what.trim().to_string(),
            evidence: evidence.clone(),
            addressed_to: addressed_to.clone(),
            closed_by: String::new(),
        });
        let row = chain::sign(&f, FILE, &me.name, key, &at, "ok", "raised")
            .map_err(|e| e.message().to_string())?;
        f.signatures.push(row);
        Ok::<_, String>(f)
    })
    .ok_or("no key is open: open yours, and the issue is signed with it")??;
    sqlite::write(&file, &path).map_err(|e| format!("{}: {}", path.display(), e.message()))?;
    Ok((
        format!("issues/{name}"),
        number,
        group.to_string(),
        addressed_to,
    ))
}
