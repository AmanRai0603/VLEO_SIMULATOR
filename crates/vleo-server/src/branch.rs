//! A branch's breakdown in the working copy, as its owner frames it
//! (docs/briefs/F4.md, F4-b; `vleo_files::breakdown`).
//!
//! `GET /v1/branch?group=<id>` gives the branch as the copy holds it: its
//! group file's digest (`opened`), whether it opens through the chain, its
//! owner and whether the open key's person may frame it (`chain::may_frame`,
//! the one rule), every block with what is still open in it, each
//! broken-down node with whether its children reproduce its cases, case by
//! case beside its estimate, its history, and the next step. It says what it
//! shows: your copy, not today's design, which a branch joins when its group
//! seals it (F5).
//!
//! `POST /v1/branch/<id>` makes one edit — `add`, `fill`, `break-down`,
//! `move`, `rename`, `split`, `fold-back`, `remove` — with `group`, `opened`,
//! `why` and the edit's own fields, as whoever's key is open. A file changed
//! since it was opened is not written over. Carries the launch's word; the
//! drive is never written.

use super::*;
use vleo_files::breakdown::{self, Answer, Branch, Edit, Framed, Part, State};
use vleo_files::chain::{self, Registry};

/// A group's id, by the library's one rule of an id (`breakdown::valid_id`).
fn group_id(id: &str) -> Result<&str, String> {
    if !breakdown::valid_id(id) {
        return Err(format!("{id:?} is not a group's id"));
    }
    Ok(id)
}

/// The copy the program reads, once it is one.
fn the_copy(ctx: &Ctx) -> Result<PathBuf, String> {
    let copy = copy::dir(ctx);
    if copy::record(&copy)?.is_none() {
        return Err(
            "a branch is framed in your copy of the database: copy it from the drive first".into(),
        );
    }
    Ok(copy)
}

/// The chain as the copy opens it.
fn chain(copy: &Path) -> Result<me::Chain, String> {
    match me::registries(copy, copy, "") {
        Ok(c) => Ok(c),
        Err(me::NotRead::FirstUse(..)) => Err(
            "this computer has not accepted this database's anchor: open your key on it first"
                .into(),
        ),
        Err(me::NotRead::Why(why)) => Err(why),
    }
}

/// The file that mounts `group`, by the library's one rule
/// (`chain::mounted_by`, as People and the chain itself read it): one
/// parent, the programme's file or the systems', on a block assigned to
/// someone. The programme's own branch is F7's.
fn mounted_by<'a>(c: &'a me::Chain, group: &str) -> Result<&'a Registry, String> {
    if group == "programme" {
        return Err(
            "the programme's branch is framed in the Programme workspace (F7), not here".into(),
        );
    }
    if group != "systems" && c.systems.is_none() {
        let why = c
            .left_out
            .iter()
            .find_map(|l| l.strip_prefix("systems: "))
            .unwrap_or("it is not in your copy");
        return Err(format!(
            "the systems' file does not open in your copy, so no branch it mounts is framed here: {why}"
        ));
    }
    let parents: Vec<&Registry> = std::iter::once(&c.programme)
        .chain(c.systems.iter())
        .collect();
    chain::mounted_by(group, &parents)
        .map(|(r, _)| r)
        .map_err(|e| e.message().to_string())
}

/// How the application opens a file it reads here.
fn how(at: &str) -> vleo_files::upgrade::Upgrade<'_> {
    vleo_files::upgrade::Upgrade { app: APP, at }
}

/// `GET /v1/branch?group=<id>`.
pub(super) fn tree(params: &str, ctx: &Ctx) -> String {
    match tree_json(params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

fn tree_json(params: &str, ctx: &Ctx) -> Result<String, String> {
    let g = param(params, "group").map(decode).unwrap_or_default();
    let group = group_id(&g)?;
    let copy = the_copy(ctx)?;
    // Who mounts it first: a group nobody mounts is said as that, not as a
    // file that is missing.
    let c = chain(&copy)?;
    let mb = mounted_by(&c, group)?;
    let now = results::now_utc();
    // Read as an edit reads it (`breakdown::group_file_in`, `nodes_in`): a
    // file that cannot be read refuses the branch by its name.
    let rel = format!("groups/{group}/{group}.vgroup");
    let (bytes, f) = breakdown::group_file_in(&copy, group).map_err(|e| e.message().to_string())?;
    let nodes =
        breakdown::nodes_in(&copy, group, &how(&now)).map_err(|e| e.message().to_string())?;
    let opens = chain::open_group_in(&f, mb, &c.programme);
    let owner = chain::owner_of(mb, group).unwrap_or("").to_string();
    let may = me::with_key(|key, me| {
        chain::may_frame(
            group,
            &me.name,
            &key.public().to_hex(),
            &now,
            mb,
            &c.programme,
        )
        .map(|_| me.name.clone())
        .map_err(|e| {
            // The owner, or their deputy, whose key does not count now, is
            // told that, not that the branch is someone else's.
            let theirs = me.name == owner
                || c.programme
                    .file
                    .people
                    .iter()
                    .any(|p| p.name == me.name && !owner.is_empty() && p.deputy_for == owner);
            (theirs, e.message().to_string())
        })
    });
    let branch = Branch::new(&f, &nodes);
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("group", group);
    j.str_field("path", &rel);
    j.str_field("opened", &vleo_files::copied::digest(&bytes));
    j.str_field(
        "shows",
        &format!("your copy: {group} as it is framed here. It joins today's design when {group} seals a release that holds it (F5)."),
    );
    j.key("signed").raw("{");
    j.bool_field("ok", opens.is_ok());
    j.str_field(
        "says",
        &match &opens {
            Ok(_) => format!("{group}'s group file opens through the chain"),
            Err(e) => e.message().to_string(),
        },
    );
    j.raw("}");
    j.str_field("owner", &owner);
    let yours = matches!(may, Some(Ok(_)));
    j.bool_field("yours", yours);
    j.str_field(
        "why_not",
        &match &may {
            None => "no key is open: open yours, and whether you frame it is read from the chain"
                .to_string(),
            Some(Ok(_)) => String::new(),
            Some(Err((true, why))) => format!(
                "Your key does not count for {group} now, so nothing is framed with it: {why}"
            ),
            Some(Err((false, why))) => format!(
                "Not yours to frame: {group} is framed by its owner {}, or their deputy. {why}",
                if owner.is_empty() {
                    "nobody registered"
                } else {
                    &owner
                }
            ),
        },
    );
    j.key("blocks").raw("[");
    let mounts: Vec<&str> = f.mounts.iter().map(|m| m.block_uid.as_str()).collect();
    let (mut differs, mut not_run) = (Vec::new(), Vec::new());
    for (i, (b, place)) in branch.blocks().into_iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        let answers = branch.answers(&b.uid);
        j.raw("{");
        j.str_field("uid", &b.uid);
        j.str_field("id", &b.id);
        j.str_field("parent", &b.parent_uid);
        j.num_field("ord", b.ord as f64);
        j.str_field("behaviour", &b.behaviour);
        j.str_field(
            "read_as",
            if answers.is_empty() {
                &b.behaviour
            } else {
                "children"
            },
        );
        j.str_field(
            "in",
            match place {
                breakdown::Place::Group => "the group file",
                breakdown::Place::Node(_) => "its node file",
            },
        );
        j.bool_field("archived", b.archived != 0);
        j.str_field("question", &b.question);
        j.bool_field("mount", mounts.contains(&b.uid.as_str()));
        j.key("open").raw("[");
        for (k, o) in branch.open_parts(&b.uid).iter().enumerate() {
            if k > 0 {
                j.raw(",");
            }
            j.raw(&json::string(o));
        }
        j.raw("]");
        j.key("ports").raw("[");
        for (k, p) in branch.ports(&b.uid).iter().enumerate() {
            if k > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("direction", &p.direction);
            j.str_field("name", &p.name);
            j.str_field("type", &p.port_type);
            j.str_field("unit", &p.unit);
            j.str_field("state", &p.state);
            j.raw("}");
        }
        j.raw("]");
        j.key("answers").raw("[");
        for (k, a) in answers.iter().enumerate() {
            if k > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("output", &a.output);
            j.str_field("child", &a.child);
            j.str_field("port", &a.port);
            j.raw("}");
        }
        j.raw("]");
        if !answers.is_empty() {
            let r = breakdown::reproduces(&branch, &b.uid, opens.as_ref().ok());
            match &r.state {
                State::Agrees => {}
                State::Differs => differs.push(b.id.clone()),
                State::NotRun(why) => not_run.push(format!("{}: {why}", b.id)),
            }
            j.key("reproduces").raw("{");
            j.str_field(
                "state",
                match &r.state {
                    State::Agrees => "agrees",
                    State::Differs => "differs",
                    State::NotRun(_) => "not run",
                },
            );
            j.str_field("says", &r.says());
            j.key("cases").raw("[");
            for (k, c) in r.cases.iter().enumerate() {
                if k > 0 {
                    j.raw(",");
                }
                j.raw("{");
                j.str_field("label", &c.label);
                j.str_field(
                    "expected",
                    &c.expected
                        .map_or("refuses".to_string(), vleo_sheet::method::show),
                );
                j.str_field("children", &c.children);
                j.str_field("estimate", &c.estimate);
                j.bool_field("agrees", c.agrees);
                j.str_field("says", &c.says);
                j.raw("}");
            }
            j.raw("]}");
        }
        j.raw("}");
    }
    j.raw("]");
    j.key("changes").raw("[");
    for (i, ch) in f.changes.iter().rev().take(50).enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.raw("{");
        j.str_field("at", &ch.at);
        j.str_field("who", &ch.who);
        j.str_field("scope", &ch.scope);
        j.str_field("what", &ch.what);
        j.raw("}");
    }
    j.raw("]");
    let open: usize = branch
        .blocks()
        .iter()
        .filter(|(b, _)| b.archived == 0)
        .map(|(b, _)| branch.open_parts(&b.uid).len())
        .sum();
    j.num_field("open", open as f64);
    j.str_field(
        "next",
        &if let Err(e) = &opens {
            format!("Nothing yet: {}", e.message())
        } else if !differs.is_empty() {
            format!("Fold back or fix {}: its children do not reproduce its cases", differs.join(", "))
        } else if !not_run.is_empty() {
            format!("A breakdown cannot be judged now, so it is not taken as reproducing: {}", not_run.join("; "))
        } else if let Some(Err((true, why))) = &may {
            format!("Nothing yet: your key does not count for {group} now: {why}")
        } else if !yours {
            format!("Nothing: {group} is framed by its owner {owner}. Read it here; ask them, or raise an issue")
        } else if open > 0 {
            format!("Fill what is open: {open} part(s) of {group} are not decided yet")
        } else {
            format!("Nothing open in {group}: issue its nodes (F5)")
        },
    );
    j.raw("}");
    Ok(j.0)
}

/// The edit `name` asks for, from its fields.
fn edit_of(name: &str, params: &str) -> Result<Edit, String> {
    let get = |k: &str| param(params, k).map(decode).unwrap_or_default();
    let need = |k: &str| {
        let v = get(k);
        if v.trim().is_empty() {
            Err(format!("{name} needs {k}"))
        } else {
            Ok(v.trim().to_string())
        }
    };
    Ok(match name {
        "add" => Edit::Add {
            parent: need("parent")?,
            id: need("id")?,
            question: get("question"),
        },
        "fill" => Edit::Fill {
            block: need("block")?,
            part: if !get("question").trim().is_empty() {
                Part::Question(get("question"))
            } else {
                Part::Port(Box::new(vleo_files::model::Port {
                    direction: need("direction")?,
                    name: need("name")?,
                    port_type: need("type")?,
                    unit: get("unit"),
                    lower: get("lower"),
                    upper: get("upper"),
                    state: get("state"),
                    maturity: get("maturity"),
                    says: get("says"),
                    ..Default::default()
                }))
            },
        },
        "break-down" => Edit::BreakDown {
            node: need("node")?,
            answers: answers_of(name, &get("answers"))?,
        },
        "move" => Edit::Move {
            block: need("block")?,
            parent: need("parent")?,
            // Nothing given is first; anything else is a whole number.
            ord: match get("ord").trim() {
                "" => 0,
                o => o
                    .parse()
                    .map_err(|_| format!("{name}'s ord is a whole number, and {o:?} is not one"))?,
            },
        },
        "rename" => Edit::Rename {
            block: need("block")?,
            id: need("id")?,
        },
        "split" => Edit::Split {
            block: need("block")?,
            id: need("id")?,
            ports: get("ports")
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect(),
        },
        "fold-back" => Edit::FoldBack {
            node: need("node")?,
        },
        "remove" => Edit::Remove {
            block: need("block")?,
        },
        other => return Err(format!("{other:?} is not an edit of a branch")),
    })
}

/// A breakdown's answers, one a line: `<output>=<child uid>.<port>`, an
/// answer for each output of the node (the library refuses fewer).
fn answers_of(name: &str, text: &str) -> Result<Vec<Answer>, String> {
    let mut out = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let parsed = line.split_once('=').and_then(|(o, from)| {
            let (child, port) = from.trim().split_once('.')?;
            Some(Answer {
                output: o.trim().to_string(),
                child: child.trim().to_string(),
                port: port.trim().to_string(),
            })
        });
        match parsed {
            Some(a) if !a.output.is_empty() && !a.child.is_empty() && !a.port.is_empty() => {
                out.push(a)
            }
            _ => {
                return Err(format!(
                    "{name}'s answer {line:?} is not <output>=<child>.<port>"
                ))
            }
        }
    }
    if out.is_empty() {
        return Err(format!(
            "{name} needs answers: one line for each output, <output>=<child>.<port>"
        ));
    }
    Ok(out)
}

/// The program, as `written_by_app` names it in what it writes.
const APP: &str = concat!("vleo ", env!("CARGO_PKG_VERSION"));

/// `POST /v1/branch/<id>`.
pub(super) fn edit_route(name: &str, params: &str, ctx: &Ctx) -> String {
    match edited(name, params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

fn edited(name: &str, params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    if me::with_key(|_, _| ()).is_none() {
        return Err("no key is open: open yours, and the edit is made and signed as you".into());
    }
    let e = edit_of(name, params)?;
    let g = param(params, "group").map(decode).unwrap_or_default();
    let group = group_id(&g)?;
    let opened = param(params, "opened").map(decode).unwrap_or_default();
    let why = param(params, "why").map(decode).unwrap_or_default();
    let copy = the_copy(ctx)?;
    let c = chain(&copy)?;
    let mb = mounted_by(&c, group)?;
    let at = results::now_utc();
    let done = me::with_key(|key, me| {
        breakdown::edit_in_place(
            &copy,
            group,
            &opened,
            &e,
            &why,
            &me.name,
            key,
            &at,
            APP,
            mb,
            &c.programme,
        )
        .map(|f| (f, me.name.clone()))
        .map_err(|e| e.message().to_string())
    })
    .ok_or("no key is open: open yours, and the edit is made and signed as you")??;
    let mut j = Json::new();
    j.raw("{");
    match done {
        (Framed::Written { digest, reproduced }, who) => {
            j.bool_field("ok", true);
            j.str_field("group", group);
            j.str_field("edit", e.name());
            j.str_field("opened", &digest);
            j.str_field(
                "message",
                &format!(
                    "{who} made the edit {:?} to {group}'s breakdown, in your copy{}. It stays in your copy until {group} is sealed (F5).",
                    e.name(),
                    reproduced
                        .map(|r| format!(": {}", r.says()))
                        .unwrap_or_default()
                ),
            );
        }
        (Framed::SecondWriter { digest }, _) => {
            j.bool_field("ok", false);
            j.bool_field("conflict", true);
            j.str_field("group", group);
            j.str_field("opened", &digest);
            j.str_field(
                "message",
                &format!("{group}'s group file changed in your copy since you opened it. Nothing is written: open it again, and make the edit on what you see."),
            );
        }
    }
    j.raw("}");
    Ok(j.0)
}

/// `GET /v1/branch/view?group=<id>` — today's view of a branch, from your
/// copy, and its impact on the rest of the design (docs/briefs/F5.md, F5-d):
/// today's design with every node of the branch its node engineer signed
/// as it is (`today::branch_view`), the nodes left out and why, and every
/// value outside the branch that moves, on the case the request carries
/// (`today::impact`). It says what it is: a view, not a release.
pub(super) fn view(params: &str, ctx: &Ctx) -> String {
    match view_json(params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

/// The last view's graph, by what it writes over today's design: a view
/// asked again of an unchanged branch is not built again. A graph lives as
/// long as the program, so one is kept, not one a request.
static VIEW_GRAPH: std::sync::Mutex<Option<(String, &'static vleo_modules::Graph)>> =
    std::sync::Mutex::new(None);

fn view_json(params: &str, ctx: &Ctx) -> Result<String, String> {
    let g = param(params, "group").map(decode).unwrap_or_default();
    let group = group_id(&g)?;
    let copy = the_copy(ctx)?;
    let c = chain(&copy)?;
    let mb = mounted_by(&c, group)?;
    let now = results::now_utc();
    let (_, f) = breakdown::group_file_in(&copy, group).map_err(|e| e.message().to_string())?;
    let nodes =
        breakdown::nodes_in(&copy, group, &how(&now)).map_err(|e| e.message().to_string())?;
    let registry = chain::open_group_in(&f, mb, &c.programme).map_err(|e| {
        format!(
            "{group}'s group file does not open through the chain in your copy, so no signature in its branch counts: {}",
            e.message()
        )
    })?;
    let view = today::branch_view(ctx.tree.clone(), &ctx.root, group, &registry, &nodes)?;
    let key = format!("{group}\0{}", view.written);
    let held = VIEW_GRAPH
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .filter(|(k, _)| *k == key)
        .map(|(_, g)| *g);
    let vg = match held {
        Some(g) => g,
        None if view.changes.is_empty() => vleo_modules::engine(),
        None => {
            let tree = vleo_sheet::load::load_all_from(&*view.files, &ctx.root)
                .map_err(|e| format!("today's view of {group} does not load: {e}"))?;
            let g = vleo_modules::opened::graph(&tree)
                .map_err(|e| format!("today's view of {group} does not run: {e}"))?;
            *VIEW_GRAPH.lock().unwrap_or_else(|e| e.into_inner()) = Some((key, g));
            g
        }
    };
    // Inside the branch: its group file's blocks, its node files', and every
    // node of it the design holds.
    let mut inside: std::collections::BTreeSet<String> =
        f.blocks.iter().map(|b| b.id.clone()).collect();
    inside.extend(
        nodes
            .iter()
            .flat_map(|n| n.blocks.iter().map(|b| b.id.clone())),
    );
    inside.extend(
        today::design_nodes(&ctx.root, group)?
            .iter()
            .flat_map(|n| n.blocks.iter().map(|b| b.id.clone())),
    );
    let moved = today::impact(
        vleo_modules::engine(),
        vg,
        &build_case(params, ctx),
        &|id| inside.contains(id),
    )?;
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("group", group);
    j.str_field(
        "says",
        &format!(
            "Today's view of {group}: today's design with every node of {group} signed as it is in your copy. A view, not a release: nothing of it is in today's design until {group} seals it."
        ),
    );
    j.key("changes").open_arr();
    for (i, n) in view.changes.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.push_string(n);
    }
    j.close_arr();
    j.key("left_out").open_arr();
    for (node, why) in &view.left_out {
        j.open_obj();
        j.str_field("node", node);
        j.str_field("why", why);
        j.close_obj();
    }
    j.close_arr();
    j.key("impact").open_arr();
    for m in &moved {
        j.open_obj();
        j.str_field("id", &m.id);
        j.str_field("group", &m.group);
        for (k, v) in [("before", m.before), ("after", m.after)] {
            match v {
                Some(x) => j.num_field(k, x),
                None => j.key(k).raw("null"),
            };
        }
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    Ok(j.0)
}
