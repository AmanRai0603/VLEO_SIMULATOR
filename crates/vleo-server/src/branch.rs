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
use vleo_files::sqlite;

/// A group's id as its folder is named.
fn group_id(id: &str) -> Result<&str, String> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
    {
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

/// The chain as the copy opens it, and the file that mounts `group` in it.
fn chain_for(copy: &Path, group: &str) -> Result<(me::Chain, usize), String> {
    let c =
        match me::registries(copy, copy, "") {
            Ok(c) => c,
            Err(me::NotRead::FirstUse(..)) => return Err(
                "this computer has not accepted this database's anchor: open your key on it first"
                    .into(),
            ),
            Err(me::NotRead::Why(why)) => return Err(why),
        };
    // 0: the programme's file; 1: the systems'.
    let by = if group == "systems" {
        0
    } else if c
        .systems
        .as_ref()
        .is_some_and(|s| s.file.mounts.iter().any(|m| m.group_id == group))
    {
        1
    } else {
        return Err(format!(
            "no file in your copy mounts {group}, so nobody frames it here: a group mounted on a subsystem's node is F6's"
        ));
    };
    Ok((c, by))
}

fn mounted_by(c: &me::Chain, by: usize) -> &vleo_files::chain::Registry {
    match by {
        1 => c.systems.as_ref().unwrap_or(&c.programme),
        _ => &c.programme,
    }
}

/// The branch's files in the copy: its group file's bytes and file, and its
/// node files.
#[allow(clippy::type_complexity)]
fn read(
    copy: &Path,
    group: &str,
) -> Result<
    (
        String,
        Vec<u8>,
        vleo_files::model::File,
        Vec<vleo_files::model::File>,
    ),
    String,
> {
    let rel = format!("groups/{group}/{group}.vgroup");
    let at = copy.join(&rel);
    let bytes = std::fs::read(&at).map_err(|e| format!("{rel}: {e}"))?;
    let f = sqlite::read(&at).map_err(|e| format!("{rel}: {}", e.message()))?;
    if std::fs::read(&at).map_err(|e| format!("{rel}: {e}"))? != bytes {
        return Err(format!(
            "{rel} was written while it was opened: open it again"
        ));
    }
    let mut nodes = Vec::new();
    for p in std::fs::read_dir(copy.join("groups").join(group).join("nodes"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "vnode"))
    {
        if let Ok(n) = sqlite::read(&p) {
            nodes.push(n);
        }
    }
    Ok((rel, bytes, f, nodes))
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
    let (c, by) = chain_for(&copy, group)?;
    let (rel, bytes, f, nodes) = read(&copy, group)?;
    let mb = mounted_by(&c, by);
    let opens = vleo_files::chain::open_group_in(&f, mb, &c.programme);
    let owner = vleo_files::chain::owner_of(mb, group)
        .unwrap_or("")
        .to_string();
    let now = results::now_utc();
    let may = me::with_key(|key, me| {
        vleo_files::chain::may_frame(
            group,
            &me.name,
            &key.public().to_hex(),
            &now,
            mb,
            &c.programme,
        )
        .map(|_| me.name.clone())
        .map_err(|e| e.message().to_string())
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
            Some(Err(why)) => format!(
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
    let mut differs = Vec::new();
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
            if r.state != State::Agrees {
                differs.push(b.id.clone());
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
            answers: vec![Answer {
                output: need("output")?,
                child: need("child")?,
                port: need("port")?,
            }],
        },
        "move" => Edit::Move {
            block: need("block")?,
            parent: need("parent")?,
            ord: get("ord").trim().parse().unwrap_or(0),
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
    let (c, by) = chain_for(&copy, group)?;
    let mb = mounted_by(&c, by);
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
