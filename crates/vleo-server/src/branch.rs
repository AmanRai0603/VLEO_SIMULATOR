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
//! `move`, `rename`, `split`, `fold-back`, `remove`, and in the systems'
//! file `mount` and `allocate` (F6-a, `vleo_files::frame`) — with `group`,
//! `opened`, `why` and the edit's own fields, as whoever's key is open. A
//! file changed since it was opened is not written over. Carries the
//! launch's word; the drive is never written.
//!
//! `POST /v1/group/start` starts a group the systems' file mounts, as its
//! owner or their deputy (`vleo_files::start`): `groups/<g>/<g>.vgroup`,
//! new, in the copy. A group mounted and not started is said as that by
//! `GET /v1/branch`, with whose step it is.
//!
//! What the frame says around a branch: the valve above it (the branch that
//! holds the leaf it hangs on, and its owner, `frame::valve_above`), what it
//! is allocated with the sense each binds, and, in the systems' file, every
//! group it mounts below.

use super::*;
use vleo_files::breakdown::{self, Answer, Branch, Edit, Framed, Part, State};
use vleo_files::chain::{self, Registry};
use vleo_files::{frame, start};

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
    // Mounted and not started: said as that, with whose step it is.
    if group != "systems" && start::to_start(&copy, group) {
        return not_started(group, &c, mb, &copy, &now);
    }
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
    j.bool_field("started", true);
    let yours = matches!(may, Some(Ok(_)));
    j.bool_field("yours", yours);
    frame_json(&mut j, group, &c, &copy, &now);
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
    let mounts_on: Vec<(String, String)> = f
        .mounts
        .iter()
        .chain(c.systems.iter().flat_map(|s| s.file.mounts.iter()))
        .map(|m| (m.block_uid.clone(), m.group_id.clone()))
        .collect();
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
        // A group mounted on it, here or in the systems' file (W13).
        let carries = mounts_on
            .iter()
            .find(|(uid, _)| *uid == b.uid)
            .map_or("", |(_, g)| g.as_str());
        j.str_field("carries", carries);
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
            // What a fill of it keeps, where the fill gives none.
            for (k, v) in [
                ("value", &p.value),
                ("lower", &p.lower),
                ("upper", &p.upper),
                ("symbol", &p.symbol),
                ("range_reason", &p.range_reason),
                ("says", &p.says),
            ] {
                j.str_field(k, v);
            }
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

/// What the frame says around `group`: the valve above it, what it is
/// allocated with the sense each binds, and, for the systems' branch, every
/// group it mounts below. Each read from the systems' file as the copy opens
/// it; the block a group hangs on is found in whichever branch holds it.
fn frame_json(j: &mut Json, group: &str, c: &me::Chain, copy: &Path, now: &str) {
    // A frame that cannot be read is said, with why: never drawn as a
    // frame with nothing in it.
    let unread = |j: &mut Json, why: &str| {
        j.bool_field("may_mount", false);
        j.str_field("systems_opened", "");
        j.str_field("frame_refused", why);
        j.key("valve_above").raw("null");
        j.key("allocations").raw("[]");
        j.key("below").raw("[]");
    };
    let Some(sy) = &c.systems else {
        let why = c
            .left_out
            .iter()
            .find_map(|l| l.strip_prefix("systems: "))
            .unwrap_or("it is not in your copy");
        return unread(
            j,
            &format!(
                "the systems' file does not open in your copy, so the frame is not read: {why}"
            ),
        );
    };
    // What is shown and what a mount is held to are one reading: the bytes
    // read here, and the file they are, as the chain opened it.
    let (bytes, read) = match breakdown::group_file_in(copy, "systems") {
        Ok(x) => x,
        Err(e) => {
            return unread(
                j,
                &format!("the systems' file cannot be read: {}", e.message()),
            )
        }
    };
    if read != sy.file {
        return unread(
            j,
            "the systems' file changed in your copy since it was opened: open the branch again",
        );
    }
    let read_all = breakdown::nodes_in(copy, "systems", &how(now))
        .and_then(|n| breakdown::others_in(copy, "systems", &how(now)).map(|o| (n, o)));
    let (nodes, elsewhere) = match read_all {
        Ok(x) => x,
        Err(e) => {
            return unread(
                j,
                &format!(
                "a file of your copy cannot be read, so where each group hangs cannot be said: {}",
                e.message()
            ),
            )
        }
    };
    // The system engineer, or their deputy, mounts groups in the systems'
    // file and allocates to them, wherever the leaf is (`chain::may_frame`,
    // the one rule); an edit there is held to the systems' file's digest.
    let may_mount = me::with_key(|key, me| {
        chain::may_frame(
            "systems",
            &me.name,
            &key.public().to_hex(),
            now,
            &c.programme,
            &c.programme,
        )
        .is_ok()
    });
    j.bool_field("may_mount", may_mount == Some(true));
    j.str_field("systems_opened", &vleo_files::copied::digest(&bytes));
    j.str_field("frame_refused", "");
    let valve = |g: &str| frame::valve_above(g, &sy.file, &nodes, &elsewhere, &c.programme);
    let valve_json = |j: &mut Json, v: Option<frame::Valve>| match v {
        Some(v) => {
            j.raw("{");
            j.str_field("branch", &v.branch);
            j.str_field("owner", &v.owner);
            j.raw("}");
        }
        None => {
            j.raw("null");
        }
    };
    j.key("valve_above");
    valve_json(
        j,
        if group == "systems" {
            None
        } else {
            valve(group)
        },
    );
    j.key("allocations").raw("[");
    let mount = sy.file.mounts.iter().find(|m| m.group_id == group);
    let mut first = true;
    if let Some(m) = mount {
        for p in sy
            .file
            .ports
            .iter()
            .filter(|p| p.block_uid == m.block_uid && p.state == "allocated")
        {
            let closure = sy
                .file
                .closures
                .iter()
                .find(|x| x.required_ref == format!("{}.{}", p.block_uid, p.name));
            if !first {
                j.raw(",");
            }
            first = false;
            j.raw("{");
            j.str_field("name", &p.name);
            j.str_field("value", &p.value);
            j.str_field("lower", &p.lower);
            j.str_field("upper", &p.upper);
            j.str_field("unit", &p.unit);
            j.str_field(
                "bounds",
                closure
                    .and_then(|x| x.achieved_ref.split_once('.'))
                    .map_or("", |(_, port)| port),
            );
            // Read from the closure, never assumed: none is said as none.
            j.str_field("sense", closure.map_or("", |x| x.sense.as_str()));
            j.str_field("says", closure.map_or("", |x| x.says.as_str()));
            j.raw("}");
        }
    }
    j.raw("]");
    j.key("below").raw("[");
    if group == "systems" {
        for (i, m) in sy.file.mounts.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.raw("{");
            j.str_field("group", &m.group_id);
            j.str_field("block", &m.block_uid);
            j.str_field("owner", chain::owner_of(sy, &m.group_id).unwrap_or(""));
            j.bool_field(
                "started",
                copy.join(format!("groups/{0}/{0}.vgroup", m.group_id))
                    .exists(),
            );
            j.key("valve_above");
            valve_json(j, valve(&m.group_id));
            j.raw("}");
        }
    }
    j.raw("]");
}

/// `group`, mounted by `mb` and with no file in the copy yet: who starts it,
/// whether the open key's person may, and the frame around it.
fn not_started(
    group: &str,
    c: &me::Chain,
    mb: &Registry,
    copy: &Path,
    now: &str,
) -> Result<String, String> {
    let owner = chain::owner_of(mb, group).unwrap_or("").to_string();
    let may = me::with_key(|key, me| {
        chain::may_frame(
            group,
            &me.name,
            &key.public().to_hex(),
            now,
            mb,
            &c.programme,
        )
        .map(|_| ())
        .map_err(|e| e.message().to_string())
    });
    let yours = matches!(may, Some(Ok(())));
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("group", group);
    j.str_field("path", &format!("groups/{group}/{group}.vgroup"));
    j.str_field(
        "shows",
        &format!(
            "your copy: {group} is mounted in {}, and its file is not started yet.",
            mb.from
        ),
    );
    j.str_field("owner", &owner);
    j.bool_field("started", false);
    j.bool_field("yours", yours);
    j.str_field(
        "why_not",
        &match &may {
            None => "no key is open: open yours, and whether you start it is read from the chain"
                .to_string(),
            Some(Ok(())) => String::new(),
            Some(Err(why)) => format!(
                "Not yours to start: {group} is started by its owner {owner}, or their deputy. {why}"
            ),
        },
    );
    frame_json(&mut j, group, c, copy, now);
    j.key("blocks").raw("[]");
    j.key("changes").raw("[]");
    j.num_field("open", 0.0);
    j.str_field(
        "next",
        &if yours {
            format!("Start {group}: its file is yours to begin, with one heading whose question is the mount's. It shows in today's design once {group} seals a release (F5).")
        } else {
            format!("Nothing yet: {group} is mounted for {owner}, who starts its file")
        },
    );
    j.raw("}");
    Ok(j.0)
}

/// `POST /v1/group/start`.
pub(super) fn start_route(params: &str, ctx: &Ctx) -> String {
    match started(params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

fn started(params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    if me::with_key(|_, _| ()).is_none() {
        return Err(
            "no key is open: open yours, and the group is started and signed as you".into(),
        );
    }
    let g = param(params, "group").map(decode).unwrap_or_default();
    let group = group_id(&g)?;
    let copy = the_copy(ctx)?;
    let c = chain(&copy)?;
    let at = results::now_utc();
    let done = me::with_key(|key, me| {
        start::start_in_place(&copy, group, &me.name, key, &at, APP, &c.programme)
            .map(|d| (d, me.name.clone()))
            .map_err(|e| e.message().to_string())
    })
    .ok_or("no key is open: open yours, and the group is started and signed as you")??;
    let (digest, who) = done;
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("group", group);
    j.str_field("path", &format!("groups/{group}/{group}.vgroup"));
    j.str_field("opened", &digest);
    j.str_field(
        "message",
        &format!("{who} started {group}, in your copy: one heading, signed by {who}. It stays in your copy until {group} is sealed (F5), and its file reaches the drive with F5's put-back."),
    );
    j.raw("}");
    Ok(j.0)
}

/// What a port filled holds now, read back from the group file written: its
/// unit, value, range, symbol and reasons, so what the fill kept of it is
/// said, not only what was typed. Nothing for any other edit.
fn holds(copy: &std::path::Path, group: &str, e: &Edit) -> String {
    let Edit::Fill {
        block,
        part: Part::Port(p),
    } = e
    else {
        return String::new();
    };
    let Ok((_, f)) = breakdown::group_file_in(copy, group) else {
        return String::new();
    };
    let Some(q) = f
        .ports
        .iter()
        .find(|q| q.block_uid == *block && q.direction == p.direction && q.name == p.name)
    else {
        return String::new();
    };
    let unit = if q.unit.trim().is_empty() || q.unit.trim() == "1" {
        String::new()
    } else {
        format!(" {}", q.unit.trim())
    };
    let mut parts = Vec::new();
    if !q.value.trim().is_empty() {
        parts.push(format!("the value {}{unit}", q.value.trim()));
    }
    if !q.lower.trim().is_empty() {
        parts.push(format!(
            "the range {} to {}{unit}{}",
            q.lower.trim(),
            q.upper.trim(),
            if q.range_reason.trim().is_empty() {
                ", with no reason given".to_string()
            } else {
                format!(", because {}", q.range_reason.trim())
            }
        ));
    }
    if parts.is_empty() {
        parts.push(format!(
            "no value or range{}",
            if unit.is_empty() {
                String::new()
            } else {
                format!(", in{unit}")
            }
        ));
    }
    if !q.symbol.trim().is_empty() {
        parts.push(format!("the symbol {}", q.symbol.trim()));
    }
    if !q.says.trim().is_empty() {
        parts.push(format!("says {:?}", q.says.trim()));
    }
    format!(" Its port {} now holds {}.", q.name, parts.join(", "))
}

/// `GET /v1/amount?text=&unit=`: one amount as typed in a field whose unit
/// is `unit`, read as the frame's forms read it (`amount::read`): its shape,
/// its numbers as typed, its unit, and what it is said back; or why not, with
/// the fix. Nothing is written.
pub(super) fn amount_route(params: &str) -> String {
    let get = |k: &str| param(params, k).map(decode).unwrap_or_default();
    match vleo_sheet::method::amount::read(&get("text"), &get("unit")) {
        Err(why) => me::refusal(&why),
        Ok(a) => {
            let mut j = Json::new();
            j.raw("{");
            j.bool_field("ok", true);
            j.str_field("typed", &a.typed);
            j.str_field(
                "shape",
                match a.shape {
                    vleo_sheet::method::amount::Shape::Value => "value",
                    vleo_sheet::method::amount::Shape::Range => "range",
                    vleo_sheet::method::amount::Shape::Spread => "spread",
                },
            );
            for (k, v) in [
                ("nominal", &a.nominal),
                ("lower", &a.lower),
                ("upper", &a.upper),
            ] {
                match v {
                    Some(v) => j.str_field(k, v),
                    None => j.key(k).raw("null"),
                };
            }
            j.str_field("unit", &a.unit);
            j.str_field("said", &a.said);
            j.raw("}");
            j.0
        }
    }
}

/// A port's `amount` as typed, read by the library into its unit, value and
/// ends (`breakdown::with_amount`), and what it is, said back; the port as it
/// is when none is typed. An amount and its ends typed apart are refused,
/// since either could be meant.
fn amount_into(
    name: &str,
    port: vleo_files::model::Port,
    typed: &str,
    allocation: bool,
) -> Result<(vleo_files::model::Port, String), String> {
    if typed.trim().is_empty() {
        return Ok((port, String::new()));
    }
    if [&port.value, &port.lower, &port.upper]
        .iter()
        .any(|v| !v.trim().is_empty())
    {
        return Err(format!(
            "{name}'s port {} is given an amount and a value or ends beside it: type the amount, as 250 km, 1 to 2 km or 55 ± 10% mN, or the value and ends, not both",
            port.name
        ));
    }
    breakdown::with_amount(&port, typed, allocation).map_err(|e| e.message().to_string())
}

/// The edit `name` asks for, from its fields, and an amount typed in it,
/// said back.
fn edit_of(name: &str, params: &str) -> Result<(Edit, String), String> {
    let get = |k: &str| param(params, k).map(decode).unwrap_or_default();
    let need = |k: &str| {
        let v = get(k);
        if v.trim().is_empty() {
            Err(format!("{name} needs {k}"))
        } else {
            Ok(v.trim().to_string())
        }
    };
    let mut said = String::new();
    let edit = match name {
        "add" => Edit::Add {
            parent: need("parent")?,
            id: need("id")?,
            question: get("question"),
        },
        "fill" => Edit::Fill {
            block: need("block")?,
            part: if !get("question").trim().is_empty() {
                // A fill is one part: an amount beside a question would be
                // dropped without a word.
                if !get("amount").trim().is_empty() {
                    return Err(format!(
                        "{name} fills one part at a time, and is given a question and an amount: fill the question, then the port"
                    ));
                }
                Part::Question(get("question"))
            } else {
                let (port, s) = amount_into(
                    name,
                    vleo_files::model::Port {
                        direction: need("direction")?,
                        name: need("name")?,
                        symbol: get("symbol").trim().to_string(),
                        port_type: need("type")?,
                        unit: get("unit"),
                        value: get("value"),
                        lower: get("lower"),
                        upper: get("upper"),
                        range_reason: get("range_reason"),
                        state: get("state"),
                        maturity: get("maturity"),
                        says: get("says"),
                        ..Default::default()
                    },
                    &get("amount"),
                    false,
                )?;
                said = s;
                Part::Port(Box::new(port))
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
        // The frame, in the systems' file (F6-a): the group is `mounts`, as
        // `group` is the branch the edit is made in.
        "mount" => Edit::Mount {
            block: need("block")?,
            group: need("mounts")?,
            owner: need("owner")?,
        },
        "allocate" => Edit::Allocate {
            block: need("block")?,
            port: {
                let (port, s) = amount_into(
                    name,
                    vleo_files::model::Port {
                        name: need("name")?,
                        port_type: need("type")?,
                        unit: get("unit"),
                        value: get("value"),
                        lower: get("lower"),
                        upper: get("upper"),
                        range_reason: get("range_reason"),
                        says: get("says"),
                        ..Default::default()
                    },
                    &get("amount"),
                    true,
                )?;
                said = s;
                Box::new(port)
            },
            bounds: need("bounds")?,
            // Never defaulted: an allocation with no sense is refused by
            // the library, by name.
            sense: get("sense"),
            says: get("closure_says"),
        },
        other => return Err(format!("{other:?} is not an edit of a branch")),
    };
    Ok((edit, said))
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
    let (e, said) = edit_of(name, params)?;
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
                    "{who} made the edit {:?} to {group}'s breakdown, in your copy{}.{}{} It stays in your copy until {group} is sealed (F5).",
                    e.name(),
                    reproduced
                        .map(|r| format!(": {}", r.says()))
                        .unwrap_or_default(),
                    if said.is_empty() {
                        String::new()
                    } else {
                        format!(" The amount is {said}.")
                    },
                    holds(&copy, group, &e)
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
