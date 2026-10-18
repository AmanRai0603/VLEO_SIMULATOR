//! The design released, at the system engineer's valve (docs/briefs/F6.md,
//! F6-b; W8; `vleo_files::design`).
//!
//! `GET /v1/design/integration` gives the integration checks of today's
//! design as they stand: for each group a line for each of W8's five
//! checks, held or failing with why, and beside them the red marks, each
//! closure that does not close and each issue open. Every release is read
//! again from the drive and checked through the chain as a build checks it
//! (`today::read_again`), so the checks are of the bytes a released design
//! would hold.
//!
//! `POST /v1/design/release` releases it as whoever's key is open, with
//! `version`, `changed`, and `understood`, every red mark marked understood
//! by its id, comma between: refused whole, by the library, when any line
//! fails or any red mark is neither raised nor understood, and for anyone
//! but the system engineer or their deputy. It is written into the copy,
//! under the copy's lock: `design/design-<version>.vleo`, new, never over;
//! `design/NOTES.md`, newest first; and `readable/`. Put-back takes them to
//! the drive. Carries the launch's word.
//!
//! The checks and the release are of today's design, built from the drive,
//! through the chain as the drive holds it; the red marks are the health
//! map's on the design as declared (`inputs=defaults`), never a saved case,
//! with each group whose latest release is refused and its last good held.
//! A design released in the copy goes back before the next is released. Not here: the pointer `design/design.vleo` and the archive (F7),
//! previews and their answers (F6-c), the daily snapshot.

use super::*;
use vleo_files::design::{self, Asked, Held, Integration, Margin, Mark, Part, Taken, Whole};
use vleo_files::meta::DesignVersion;
use vleo_modules::health::{health, is_closure, State};

const APP: &str = concat!("vleo ", env!("CARGO_PKG_VERSION"));

/// One group's part of today's design, read again from the drive: its
/// release's path there, version, how its seal checked, bytes and file, or
/// why none is taken.
struct Read {
    group: String,
    taken: Result<(String, String, String, Vec<u8>, vleo_files::model::File), String>,
}

/// Everything the checks and the release are made from.
struct Today {
    drive: PathBuf,
    released: Option<String>,
    parts: Vec<Read>,
    marks: Vec<Mark>,
}

impl Today {
    fn integration(&self) -> Integration {
        let parts: Vec<Part> = self
            .parts
            .iter()
            .map(|p| Part {
                group: &p.group,
                taken: match &p.taken {
                    Ok((_, version, chain, _, file)) => Ok(Taken {
                        version,
                        chain,
                        file,
                    }),
                    Err(why) => Err(why.as_str()),
                },
            })
            .collect();
        design::integration(&parts, self.released.as_deref().unwrap_or(""))
    }
}

/// The chain as the drive holds it, opened from the drive's own files.
fn chain_of(drive: &Path, ctx: &Ctx) -> Result<me::Chain, String> {
    match me::registries(drive, &copy::dir(ctx), "") {
        Ok(c) => Ok(c),
        Err(me::NotRead::FirstUse(..)) => Err(
            "this computer has not accepted this database's anchor: open your key on it first"
                .into(),
        ),
        Err(me::NotRead::Why(why)) => Err(format!(
            "the drive's files do not open through the chain, so nothing is checked: {why}"
        )),
    }
}

/// Today's design, as the checks read it: every group with a release on the
/// drive, and every group the systems' file mounts without one, and the
/// chain it was read through.
fn today(ctx: &Ctx) -> Result<(Today, me::Chain), String> {
    let report = built_from().ok_or(
        "the design open is the design's own files, not today's design: the design is released from today's design, built from the drive",
    )?;
    let drive = PathBuf::from(&report.drive);
    let c = chain_of(&drive, ctx)?;
    let systems = c.systems.as_ref().ok_or(
        "the systems' file on the drive does not check through the chain, so the groups it mounts are not known",
    )?;
    let released = design::latest_in(&drive.join("design"), &c.programme)
        .map_err(|e| e.message().to_string())?;
    let mut parts = Vec::new();
    let mut marks = Vec::new();
    for g in &report.groups {
        let taken = match &g.taken {
            None => Err(g.said()),
            Some(t) => today::read_again(&drive, &g.id, &t.path)
                .and_then(|(bytes, file, chain)| {
                    let said = |k: &str| file.meta.get(k).cloned().unwrap_or_default();
                    if said("version") != t.version || said("fingerprint") != t.fingerprint {
                        return Err(format!(
                            "{} {} on the drive is not the release today's design took: start the tool again, so today's design is built from the drive as it is",
                            g.id, t.version
                        ));
                    }
                    let under = t
                        .path
                        .strip_prefix(&drive)
                        .map_err(|_| format!("{} is not on the drive", t.path.display()))?
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy().into_owned())
                        .collect::<Vec<_>>()
                        .join("/");
                    // A newer release refused is said on the seal's line, and
                    // is a red mark: its last good one is held in its place.
                    let chain = match g.refused.first() {
                        Some((v, why)) => {
                            marks.push(Mark {
                                id: format!("group:{}", g.id),
                                said: format!(
                                    "{}'s latest, {v}, is refused ({why}); {} {} is held in its place",
                                    g.id, g.id, t.version
                                ),
                                raised: None,
                            });
                            format!("{chain}; its latest, {v}, is refused, so its last good release is held")
                        }
                        None => chain,
                    };
                    Ok((under, t.version.clone(), chain, bytes, file))
                })
                .map_err(|why| format!("{} {} is not read again from the drive: {why}", g.id, t.version)),
        };
        parts.push(Read {
            group: g.id.clone(),
            taken,
        });
    }
    // A group the systems' file mounts and that has no release on the drive
    // is not in the design released: it is said, never left out.
    for m in &systems.file.mounts {
        if !parts.iter().any(|p| p.group == m.group_id) {
            parts.push(Read {
                group: m.group_id.clone(),
                taken: Err(format!(
                    "{} is mounted, and the drive holds no release of it",
                    m.group_id
                )),
            });
        }
    }
    marks.extend(red_marks(ctx, &drive));
    Ok((
        Today {
            marks,
            drive,
            released,
            parts,
        },
        c,
    ))
}

/// The case the red marks and the health map are read on: the design as
/// declared, never the case someone last saved, so every computer gives the
/// same (W6).
fn declared_case(ctx: &Ctx) -> vleo_bus::Case {
    let g = vleo_modules::engine();
    let mut case = build_case("inputs=defaults", ctx);
    case.target = g.nodes.first().map_or("", |n| n.id).to_string();
    case.mode = RunMode::All;
    case
}

/// The red marks of today's design: each closure that fails, is refused or
/// is blocked on the design as declared, raised when an issue open names it;
/// each issue open on the drive or in the copy, the drive's first; and each
/// issue file that does not read, said, never passed over.
fn red_marks(ctx: &Ctx, drive: &Path) -> Vec<Mark> {
    let mut open: Vec<(String, vleo_files::model::Issue)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for dir in [drive.join("issues"), copy::dir(ctx).join("issues")] {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut names: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "vissue"))
            .collect();
        names.sort();
        for p in names {
            let name = p
                .file_name()
                .map_or(String::new(), |n| n.to_string_lossy().into_owned());
            if seen.contains(&name) {
                continue;
            }
            seen.push(name.clone());
            match vleo_files::sqlite::read(&p) {
                Ok(f) => {
                    for i in f.issues.into_iter().filter(|i| i.closed_by.is_empty()) {
                        open.push((name.clone(), i));
                    }
                }
                Err(e) => out.push(Mark {
                    id: format!("issue:{}", name.trim_end_matches(".vissue")),
                    said: format!("the issue file {name} does not read: {}", e.message()),
                    raised: None,
                }),
            }
        }
    }
    let g = vleo_modules::engine();
    let map = health(g, &declared_case(ctx));
    for n in &map.nodes {
        if !is_closure(g, n.node)
            || !matches!(n.state, State::Fails | State::Refused | State::Blocked)
        {
            continue;
        }
        let id = g.nodes[n.node as usize].id;
        out.push(Mark {
            id: format!("closure:{id}"),
            said: format!("{id} {}: {}", n.state.name(), n.why),
            raised: open
                .iter()
                .find(|(_, i)| i.place == id)
                .map(|(_, i)| i.number),
        });
    }
    for (name, i) in &open {
        out.push(Mark {
            id: format!("issue:{}", name.trim_end_matches(".vissue")),
            said: format!("issue {} on {}, open: {}", i.number, i.group_id, i.what),
            raised: None,
        });
    }
    out
}

/// The version after `latest` this month, as the page offers it; after a
/// latest released in a later month than this computer's clock, the next in
/// its month.
fn next_version(latest: Option<&str>) -> String {
    let now = results::now_utc();
    let year = now.get(0..4).and_then(|y| y.parse().ok()).unwrap_or(2026);
    let month = now.get(5..7).and_then(|m| m.parse().ok()).unwrap_or(1);
    match latest.and_then(DesignVersion::parse) {
        Some(v) if (v.year, v.month) > (year, month) => v.next(v.year, v.month).to_string(),
        Some(v) => v.next(year, month).to_string(),
        None => format!("{year}.{month}.1"),
    }
}

/// `GET /v1/design/integration`.
pub(super) fn integration_route(ctx: &Ctx) -> String {
    let t = match today(ctx) {
        Ok((t, _)) => t,
        Err(why) => return me::refusal(&why),
    };
    let i = t.integration();
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("drive", &t.drive.display().to_string());
    j.str_field("released", t.released.as_deref().unwrap_or(""));
    j.str_field("next_version", &next_version(t.released.as_deref()));
    j.bool_field("holds", i.holds());
    j.key("lines").open_arr();
    for l in &i.lines {
        j.open_obj();
        j.str_field("group", &l.group);
        j.str_field("check", l.check.name());
        j.bool_field("holds", l.holds);
        j.str_field("says", &l.says);
        j.close_obj();
    }
    j.close_arr();
    j.key("marks").open_arr();
    for m in &t.marks {
        j.open_obj();
        j.str_field("id", &m.id);
        j.str_field("said", &m.said);
        match m.raised {
            Some(n) => j.num_field("raised", n as f64),
            None => j.key("raised").raw("null"),
        };
        j.close_obj();
    }
    j.close_arr();
    j.str_field(
        "message",
        &if i.holds() {
            format!(
                "every check holds for {} group{}: the design can be released once every red mark is understood",
                t.parts.len(),
                if t.parts.len() == 1 { "" } else { "s" }
            )
        } else if i.lines.is_empty() {
            "today's design holds no group, so there is nothing to release".to_string()
        } else {
            format!(
                "{} of {} lines fail: the design is not released until each holds",
                i.lines.iter().filter(|l| !l.holds).count(),
                i.lines.len()
            )
        },
    );
    j.raw("}");
    j.0
}

/// `POST /v1/design/release`.
pub(super) fn release_route(params: &str, ctx: &Ctx) -> String {
    match released(params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

/// The bytes of the file at `path` and the file those bytes read as: read
/// once, and parsed from what was read (`today::of_bytes`), so the two are
/// one.
fn whole(path: &Path) -> Result<(Vec<u8>, vleo_files::model::File), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let f = today::of_bytes(&bytes, |at| {
        vleo_files::sqlite::read(at).map_err(|e| format!("{}: {}", path.display(), e.message()))
    })?;
    Ok((bytes, f))
}

/// A released design in the copy's `design/` that the drive does not hold,
/// by its path, if there is one.
fn not_put_back(copy: &Path, drive: &Path) -> Result<Option<String>, String> {
    let dir = copy.join("design");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(None);
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .filter(|n| n.starts_with("design-") && n.ends_with(".vleo"))
        .collect();
    names.sort();
    for n in names {
        let (mine, theirs) = (dir.join(&n), drive.join("design").join(&n));
        let same = match (std::fs::read(&mine), std::fs::read(&theirs)) {
            (Ok(a), Ok(b)) => a == b,
            (Err(e), _) => return Err(format!("{}: {e}", mine.display())),
            (Ok(_), Err(_)) => false,
        };
        if !same {
            return Ok(Some(format!("design/{n}")));
        }
    }
    Ok(None)
}

fn released(params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    let no_key = "no key is open: open yours, and the design is released and signed as you";
    if me::with_key(|_, _| ()).is_none() {
        return Err(no_key.into());
    }
    let get = |k: &str| param(params, k).map(decode).unwrap_or_default();
    let version = get("version").trim().to_string();
    let changed = get("changed");
    let understood: Vec<String> = get("understood")
        .split(',')
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
        .collect();
    let copy = copy::dir(ctx);
    let record = copy::record(&copy)?.ok_or(
        "a design is released in your copy of the database, then put back: copy it from the drive first",
    )?;
    let (t, c) = today(ctx)?;
    let drive =
        std::fs::canonicalize(&t.drive).map_err(|e| format!("{}: {e}", t.drive.display()))?;
    if Path::new(&record.drive) != drive {
        return Err(format!(
            "your copy came from {}, and today's design is built from {}: release it in a copy of the drive it is built from",
            record.drive,
            drive.display()
        ));
    }
    // A design released in the copy and not put back yet is the one the
    // next follows: it goes back first, so two never follow the same one.
    if let Some(waiting) = not_put_back(&copy, &t.drive)? {
        return Err(format!(
            "your copy holds {waiting}, released and not put back on the drive yet: put it back, then release the next on a copy taken after it"
        ));
    }
    let checks = t.integration();
    let (pb, pf) = whole(&t.drive.join("groups/programme/programme.vgroup"))?;
    let (sb, sf) = whole(&t.drive.join("groups/systems/systems.vgroup"))?;
    if pf != c.programme.file || c.systems.as_ref().map(|s| &s.file) != Some(&sf) {
        return Err("the programme's or the systems' file changed on the drive while it was read: try again".into());
    }
    let releases: Vec<(&str, &str, &str, Whole)> = t
        .parts
        .iter()
        .filter_map(|p| p.taken.as_ref().ok().map(|x| (p, x)))
        .map(|(p, (path, version, _, bytes, file))| {
            (
                p.group.as_str(),
                version.as_str(),
                file.meta.get("fingerprint").map_or("", String::as_str),
                Whole { path, bytes },
            )
        })
        .collect();
    let held = Held {
        releases,
        programme: Whole {
            path: "groups/programme/programme.vgroup",
            bytes: &pb,
        },
        systems: Whole {
            path: "groups/systems/systems.vgroup",
            bytes: &sb,
        },
    };
    let margins = margins(ctx);
    let asked = Asked {
        version: &version,
        previous: t.released.as_deref(),
        changed: &changed,
        marks: &t.marks,
        understood: &understood,
        margins: &margins,
    };
    let at = results::now_utc();
    let (f, who) = me::with_key(|key, me| {
        design::release(
            &checks,
            &held,
            &asked,
            &me.name,
            key,
            &at,
            APP,
            &c.programme,
        )
        .map(|f| (f, me.name.clone()))
        .map_err(|e| e.message().to_string())
    })
    .ok_or(no_key)??;
    // Into the copy, under its lock: the released design new, then its
    // record.
    let _held = vleo_files::save::lock(&copy).map_err(|e| e.message().to_string())?;
    let name = format!("design/design-{version}.vleo");
    let at_file = copy.join(&name);
    if at_file.exists() {
        return Err(format!(
            "your copy holds {name} already: a released design is never written over"
        ));
    }
    let io = |p: &Path, e: std::io::Error| format!("{}: {e}", p.display());
    std::fs::create_dir_all(copy.join("design")).map_err(|e| io(&copy.join("design"), e))?;
    vleo_files::sqlite::write(&f, &at_file).map_err(|e| e.message().to_string())?;
    // Its record, written after it; when that fails the released design is
    // taken out again, so the copy never holds one without its record.
    let undo = |why: String| {
        let _ = std::fs::remove_file(&at_file);
        format!("the design is not released in your copy, nothing of it is kept: {why}")
    };
    let notes_at = copy.join("design/NOTES.md");
    let before = match std::fs::read_to_string(&notes_at) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(undo(io(&notes_at, e))),
    };
    let body = before.strip_prefix(NOTES_HEAD).unwrap_or(&before);
    // What changed is the system engineer's words, quoted, so no line of
    // it reads as a heading of the notes.
    let quoted: String = changed.trim().lines().map(|l| format!("> {l}\n")).collect();
    let mut entry = format!(
        "## {version}\n\nReleased by {who}, {at}, after {}.\n\n{quoted}",
        t.released.as_deref().unwrap_or("none"),
    );
    for m in &t.marks {
        entry.push_str(&format!(
            "\n- {}: {}",
            m.said,
            match m.raised {
                Some(n) => format!("raised as issue {n}"),
                None => "understood".to_string(),
            }
        ));
    }
    if !t.marks.is_empty() {
        entry.push('\n');
    }
    std::fs::write(
        &notes_at,
        format!("{NOTES_HEAD}{entry}\n{}", body.trim_start()),
    )
    .map_err(|e| undo(io(&notes_at, e)))?;
    let by_group: Vec<(&str, &vleo_files::model::File)> = t
        .parts
        .iter()
        .filter_map(|p| p.taken.as_ref().ok().map(|x| (p.group.as_str(), &x.4)))
        .collect();
    let mut readable = design::readable(&f, &by_group)
        .into_iter()
        .map(|(n, csv)| (n.to_string(), csv))
        .collect::<Vec<_>>();
    readable.push(("Health.csv".to_string(), health_csv(ctx, &version)));
    std::fs::create_dir_all(copy.join("readable"))
        .map_err(|e| undo(io(&copy.join("readable"), e)))?;
    let mut files = vec![name.clone(), "design/NOTES.md".to_string()];
    for (n, csv) in &readable {
        let p = copy.join("readable").join(n);
        std::fs::write(&p, csv).map_err(|e| undo(io(&p, e)))?;
        files.push(format!("readable/{n}"));
    }
    let failing: Vec<&str> = t
        .marks
        .iter()
        .filter(|m| m.id.starts_with("closure:"))
        .map(|m| m.said.as_str())
        .collect();
    let announce = format!(
        "Design {version} is released: {}{}",
        changed.trim().trim_end_matches('.'),
        if failing.is_empty() {
            ". Nothing fails.".to_string()
        } else {
            format!(". Still failing: {}.", failing.join("; "))
        }
    );
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("version", &version);
    j.str_field("path", &name);
    strings(&mut j, "files", &files);
    j.str_field("announce", &announce);
    j.str_field(
        "message",
        &format!(
            "{who} released design {version}, in your copy as {name}, with its notes and readable/. It reaches everyone once it is put back on the drive."
        ),
    );
    j.raw("}");
    Ok(j.0)
}

const NOTES_HEAD: &str = "# Released designs\n\nWhat changed in each, newest first.\n\n";

/// The health map of the design released, as CSV: each node's state and
/// why on the design as declared, the engine's.
/// Every closure of today's design as the engine answers it on the design as
/// declared: what the released design keeps as `margins.csv`, so a budget is
/// read across releases and never computed again.
fn margins(ctx: &Ctx) -> Vec<Margin> {
    let g = vleo_modules::engine();
    let map = health(g, &declared_case(ctx));
    map.nodes
        .iter()
        .filter(|n| is_closure(g, n.node))
        .map(|n| Margin {
            node: g.nodes[n.node as usize].id.to_string(),
            state: n.state.name().to_string(),
            margin: n.margin,
            why: n.why.clone(),
        })
        .collect()
}

/// `GET /v1/design/budgets`: every closure's margin across the designs
/// released on the drive today's design is built from, read from each one's
/// `margins.csv` as it was released, oldest first; nothing is computed
/// again. A design released before margins were kept is said, never left
/// out, and a design on the drive that does not read back refuses the whole
/// answer by name.
pub(super) fn budgets_route(ctx: &Ctx) -> String {
    let Some(report) = built_from() else {
        return me::refusal(
            "the design open is the design's own files, not today's design: budgets are read from the designs released on the drive",
        );
    };
    let drive = PathBuf::from(&report.drive);
    let c = match chain_of(&drive, ctx) {
        Ok(c) => c,
        Err(why) => return me::refusal(&why),
    };
    let designs = match design::released_in(
        &[&drive.join("design"), &drive.join("design/archive")],
        &c.programme,
        APP,
    ) {
        Ok(d) => d,
        Err(e) => {
            return me::refusal(&format!(
                "the released designs are not read: {}",
                e.message()
            ))
        }
    };
    let (budgets, without) = match design::budgets(&designs) {
        Ok(b) => b,
        // Read back through `open_held`, which reads its margins: never
        // here.
        Err(why) => return me::refusal(&why),
    };
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("drive", &drive.display().to_string());
    j.key("designs").open_arr();
    for o in &designs {
        j.open_obj();
        j.str_field("version", &o.version);
        if without.contains(&o.version) {
            j.bool_field("kept", false);
            j.str_field(
                "says",
                &format!(
                    "design {} was released before margins were kept, so its budgets are not known",
                    o.version
                ),
            );
        } else {
            j.bool_field("kept", true);
            j.str_field(
                "application",
                o.design
                    .meta
                    .get("oldest_application")
                    .map_or("", String::as_str),
            );
        }
        j.close_obj();
    }
    j.close_arr();
    j.key("closures").open_arr();
    for b in &budgets {
        j.open_obj();
        j.str_field("node", &b.node);
        j.key("answers").open_arr();
        for (version, m) in &b.answers {
            j.open_obj();
            j.str_field("version", version);
            j.str_field("state", &m.state);
            match m.margin {
                Some(v) => j.num_field("margin", v),
                None => j.key("margin").raw("null"),
            };
            j.str_field("why", &m.why);
            j.close_obj();
        }
        j.close_arr();
        j.close_obj();
    }
    j.close_arr();
    let kept = designs.len() - without.len();
    j.str_field(
        "message",
        &if designs.is_empty() {
            "no design is released on the drive yet, so there are no budgets across releases".to_string()
        } else {
            format!(
                "{} closure{} across {} of {} released design{}: each margin as the engine answered it when its design was released, on the design as declared, never computed again",
                budgets.len(),
                if budgets.len() == 1 { "" } else { "s" },
                kept,
                designs.len(),
                if designs.len() == 1 { "" } else { "s" }
            )
        },
    );
    j.raw("}");
    j.0
}

fn health_csv(ctx: &Ctx, version: &str) -> String {
    let g = vleo_modules::engine();
    let map = health(g, &declared_case(ctx));
    let rows: Vec<Vec<String>> = map
        .nodes
        .iter()
        .map(|n| {
            let d = &g.nodes[n.node as usize];
            vec![
                d.id.to_string(),
                d.parent.to_string(),
                n.state.name().to_string(),
                n.margin.map_or(String::new(), |m| m.to_string()),
                n.why.clone(),
                version.to_string(),
            ]
        })
        .collect();
    vleo_files::csv::write(
        &["node", "parent", "state", "margin", "why", "design"],
        &rows,
    )
}
