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
//! the drive. Carries the launch's word. The released design keeps
//! `margins.csv`, every closure as the engine answers it on the design as
//! declared, and is refused when the engine runs another design than the
//! one the drive's releases built (`engine_is_today_s`).
//!
//! `GET /v1/design/budgets` gives every closure's margin across the designs
//! released on the drive, read from each one's `margins.csv` as it was
//! released, never computed again.
//!
//! The checks and the release are of today's design, built from the drive,
//! through the chain as the drive holds it; the red marks are the health
//! map's on the design as declared (`inputs=defaults`), never a saved case,
//! with each group whose latest release is refused and its last good held.
//! A design released in the copy goes back before the next is released.
//!
//! `GET /v1/design/previews` gives, for each group whose release in today's
//! design is not the one in the design released now, the preview of it on
//! that design where there is one, and every preview on the drive and in
//! the copy, read back, with what each group it names has said, unanswered
//! first; the integration route gives them too. `POST /v1/design/preview`,
//! with `group`, writes one in the copy as whoever's key is open, the
//! system engineer or their deputy: two designs built and run on the
//! design as declared, the design released now and the same with the
//! group's release in it, and every value outside the group answered
//! differently held in the preview (F6-c items 3, 4 and 10). Previews are
//! built only when asked for, never on a reading of the checks. Carries the
//! launch's word.
//!
//! `POST /v1/design/decide`, with `group`, `version`, `answering` and
//! `decision`, `next` or `anyway`, decides an objection on the drive by an
//! issue in the copy, as whoever's key is open, the system engineer or their
//! deputy (F6-c3b); an allocation changed decides one through the frame, by
//! `reallocate`. The release reads every preview, every objection and what
//! was decided of it from the drive and the copy, and the library refuses it
//! while any is wanting. An issue deciding an objection is that objection's
//! red mark, never one of its own. Carries the launch's word.
//!
//! Not here: the pointer `design/design.vleo` and the archive (F7), the
//! daily snapshot.

use super::*;
use vleo_files::chain::Registry;
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
    /// Every preview as it stands and every objection decided (F6-c3b).
    standing: Objections,
    /// The objections that are red marks, each raised by its programme
    /// manager's issue, when the previews ask nothing more.
    objected: Vec<Mark>,
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
    let regs: Vec<&Registry> = std::iter::once(&c.programme)
        .chain(c.systems.iter())
        .chain(c.groups.iter())
        .collect();
    marks.extend(red_marks(ctx, &drive, &regs, &c.left_out));
    // An issue the previews count as an objection's raised mark is that
    // objection's (`design::asked_of`), never a mark of its own; every other
    // issue, a next version asked for, a decision of a stale objection or
    // one that does not count among them, is said as it is.
    let held: Vec<(String, String)> = parts
        .iter()
        .filter_map(|p| {
            p.taken
                .as_ref()
                .ok()
                .map(|t| (p.group.clone(), t.1.clone()))
        })
        .collect();
    let holding: Vec<(&str, &str)> = held.iter().map(|(g, v)| (g.as_str(), v.as_str())).collect();
    let standing = objections_now(&drive, &copy::dir(ctx), &c, released.as_deref(), &holding);
    let objected = design::asked_of(
        &standing.previews,
        &standing.unread,
        &standing.decided,
        released.as_deref(),
        &holding,
    )
    .map(|(m, _)| m)
    .unwrap_or_default();
    let raised: Vec<String> = objected
        .iter()
        .filter_map(|m| m.raised)
        .map(|n| format!("issue:{n}-programme"))
        .collect();
    marks.retain(|m| !raised.contains(&m.id));
    Ok((
        Today {
            marks,
            drive,
            released,
            parts,
            standing,
            objected,
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
/// each issue open on the drive or in the copy, the drive's first, that
/// checks through the chain, signed by its raiser with a key one of `regs`
/// counts; and each issue file that does not read, or does not check, said,
/// never passed over.
fn red_marks(ctx: &Ctx, drive: &Path, regs: &[&Registry], left_out: &[String]) -> Vec<Mark> {
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
                // An issue counts only as its raiser signed it; one that
                // does not check is said with why, never passed over.
                // A file that does not check, or does not read, is a mark of
                // its own, never the issue's: one marked understood as the
                // issue is not understood as a file changed since.
                Ok(f) => match vleo_files::chain::check_issue(&f, regs) {
                    Ok(()) => {
                        // A row from before F6-d is not what its raiser
                        // signed: it is listed, and neither closes the issue
                        // nor raises a closure's mark.
                        let signed = f.meta.contains_key(vleo_files::chain::SIGNED_ROWS);
                        for mut i in f.issues.into_iter() {
                            if !signed {
                                i.closed_by.clear();
                                i.place.clear();
                                i.what = format!("{} (raised before its row was signed)", i.what);
                            }
                            if i.closed_by.is_empty() {
                                open.push((name.clone(), i));
                            }
                        }
                    }
                    Err(e) => out.push(Mark {
                        id: format!("issue-file:{}", name.trim_end_matches(".vissue")),
                        said: format!(
                            "the issue file {name} does not check through the chain: {}{}",
                            e.message(),
                            if left_out.is_empty() {
                                String::new()
                            } else {
                                format!("; left out of the chain: {}", left_out.join("; "))
                            }
                        ),
                        raised: None,
                    }),
                },
                Err(e) => out.push(Mark {
                    id: format!("issue-file:{}", name.trim_end_matches(".vissue")),
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
    let (t, c) = match today(ctx) {
        Ok(t) => t,
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
    // Beside the checks, the previews and their answers (F6-c).
    previews_into(&mut j, ctx, &t.drive, &c);
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
    // Every preview as it stands, and every objection to one with what was
    // decided of it: the library refuses the release while any is wanting.
    let standing = &t.standing;
    // The engine's answers on the design as declared, read once for the
    // margins kept and for readable/Health.csv, of the design released.
    let today_is = built_from().map(|r| r.fingerprint).unwrap_or_default();
    engine_is_today_s(engine_built_for().as_deref(), &today_is)?;
    let map = health(vleo_modules::engine(), &declared_case(ctx));
    let margins = margins(&map);
    let asked = Asked {
        version: &version,
        previous: t.released.as_deref(),
        changed: &changed,
        marks: &t.marks,
        understood: &understood,
        margins: &margins,
        previews: &standing.previews,
        unread: &standing.unread,
        decided: &standing.decided,
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
    for m in t.marks.iter().chain(t.objected.iter()) {
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
    readable.push(("Health.csv".to_string(), health_csv(&map, &version)));
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

/// Every closure of the design the engine runs, from its health map on the
/// design as declared: what the released design keeps as `margins.csv`, so
/// a budget is read across releases and never computed again.
fn margins(map: &vleo_modules::health::Health) -> Vec<Margin> {
    let g = vleo_modules::engine();
    map.nodes
        .iter()
        .filter(|n| is_closure(g, n.node))
        .map(|n| {
            let d = &g.nodes[n.node as usize];
            Margin {
                node: d.id.to_string(),
                group: d.parent.to_string(),
                state: n.state.name().to_string(),
                margin: n.margin,
                why: n.why.clone(),
            }
        })
        .collect()
}

/// Whether the engine runs the design `today` was built from: the margins a
/// released design keeps are the engine's, and are never computed again, so
/// an engine installed for another design refuses the release by name.
/// `engine` is the fingerprint of today's design when the engine was
/// installed (`engine_built_for`), `today` its fingerprint now.
fn engine_is_today_s(engine: Option<&str>, today: &str) -> Result<(), String> {
    match engine {
        Some(e) if e == today => Ok(()),
        Some(e) => Err(format!(
            "the engine runs the design whose fingerprint is {}…, and today's design is now {}…: its margins would not be this design's, so it is not released. Start the tool again, so the engine runs today's design",
            e.chars().take(16).collect::<String>(),
            today.chars().take(16).collect::<String>()
        )),
        None => Err(
            "the engine was not installed for today's design: its margins would not be this design's, so it is not released. Start the tool again, so the engine runs today's design"
                .to_string(),
        ),
    }
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
    let b = match design::budgets(&designs) {
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
        match b.kept.iter().find(|(v, _)| v == &o.version) {
            None => {
                j.bool_field("kept", false);
                j.str_field(
                    "says",
                    &format!(
                    "design {} was released before margins were kept, so its budgets are not known",
                    o.version
                ),
                );
            }
            // The application whose engine gave its margins, as its rows
            // say it.
            Some((_, application)) => {
                j.bool_field("kept", true);
                j.str_field("application", application);
            }
        }
        j.close_obj();
    }
    j.close_arr();
    j.key("closures").open_arr();
    for c in &b.closures {
        j.open_obj();
        j.str_field("node", &c.node);
        j.str_field("group", &c.group);
        j.key("answers").open_arr();
        for (version, m) in &c.answers {
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
    let kept = b.kept.len();
    j.str_field(
        "message",
        &if designs.is_empty() {
            "no design is released on the drive yet, so there are no budgets across releases".to_string()
        } else if kept == 0 {
            format!(
                "none of the {} released design{} kept margins: each was released before they were, so no budget across releases is known",
                designs.len(),
                if designs.len() == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "{} closure{} across {} of {} released design{}: each margin as the engine answered it when its design was released, on the design as declared, never computed again",
                b.closures.len(),
                if b.closures.len() == 1 { "" } else { "s" },
                kept,
                designs.len(),
                if designs.len() == 1 { "" } else { "s" }
            )
        },
    );
    j.raw("}");
    j.0
}

/// The health map of the design released, as CSV: each node's state and
/// why on the design as declared, the engine's.
fn health_csv(map: &vleo_modules::health::Health, version: &str) -> String {
    let g = vleo_modules::engine();
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

// ── Previews and their answers (F6-c items 3, 4 and 10) ──

/// Each graph a preview built, by the design's root, the fingerprint of the
/// design's own files under it, and the digest of what it took into them: a
/// graph lives as long as the program (`opened::graph` keeps it), so a design
/// asked again is not built again. Each design previewed keeps one, never
/// freed, and a preview is built only when asked for (docs/briefs/F6.md,
/// F6-c, *Risks*).
static GRAPHS: std::sync::Mutex<Vec<(String, &'static vleo_modules::Graph)>> =
    std::sync::Mutex::new(Vec::new());

fn graph_of(
    t: &today::Today,
    ctx: &Ctx,
    what: &str,
) -> Result<&'static vleo_modules::Graph, String> {
    let key = format!(
        "{}\0{}\0{}",
        ctx.root.display(),
        ctx.design.as_ref().map_or("", |d| d.fingerprint.as_str()),
        t.written
    );
    let mut held = GRAPHS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, g)) = held.iter().find(|(k, _)| *k == key) {
        return Ok(g);
    }
    let tree = vleo_sheet::load::load_all_from(&*t.files, &ctx.root)
        .map_err(|e| format!("{what} does not load: {e}"))?;
    let g = vleo_modules::opened::graph(&tree).map_err(|e| format!("{what} does not run: {e}"))?;
    held.push((key, g));
    Ok(g)
}

/// The design released now, read back from the drive through the chain
/// (`design::open_held`), or why there is none.
fn released_now(drive: &Path, programme: &Registry) -> Result<design::Opened, String> {
    let v = design::latest_in(&drive.join("design"), programme)
        .map_err(|e| e.message().to_string())?
        .ok_or("no design is released yet: a preview is on the design released now, so none is written before the first")?;
    let f = vleo_files::sqlite::read(&drive.join(format!("design/design-{v}.vleo")))
        .map_err(|e| format!("design {v} does not read: {}", e.message()))?;
    design::open_held(&f, programme, APP).map_err(|e| e.message().to_string())
}

/// What a preview of `group`'s release in today's design is made from: the
/// release, as the drive holds it and where; the design released now; every
/// value outside the group that taking it in moves, on the design as
/// declared; and how each design ran.
struct Previewed {
    path: String,
    bytes: Vec<u8>,
    version: String,
    design: design::Opened,
    moved: Vec<design::Move>,
    impact: today::Impact,
}

/// `first` is asked where the preview of the group's release, its version and
/// the design it is on, would be written, before anything is built.
fn previewed(
    group: &str,
    ctx: &Ctx,
    c: &me::Chain,
    first: &dyn Fn(&str, &str, &str) -> Result<String, String>,
) -> Result<Previewed, String> {
    let report = built_from().ok_or(
        "the design open is the design's own files, not today's design: a preview is of a release in today's design, built from the drive",
    )?;
    let drive = PathBuf::from(&report.drive);
    let design = released_now(&drive, &c.programme)?;
    let t = report
        .groups
        .iter()
        .find(|g| g.id == group)
        .and_then(|g| g.taken.as_ref())
        .ok_or_else(|| {
            format!("today's design takes no release of {group}: there is nothing to preview")
        })?;
    let (bytes, file, _) = today::read_again(&drive, group, &t.path).map_err(|why| {
        format!(
            "{group} {} is not read again from the drive: {why}",
            t.version
        )
    })?;
    if file.meta.get("fingerprint") != Some(&t.fingerprint) {
        return Err(format!(
            "{group} {} on the drive is not the release today's design took: start the tool again, so today's design is built from the drive as it is",
            t.version
        ));
    }
    design::to_preview(&design, group, &t.version, &t.fingerprint)
        .map_err(|e| e.message().to_string())?;
    first(group, &t.version, &design.version)?;
    let path = t
        .path
        .strip_prefix(&drive)
        .map_err(|_| format!("{} is not on the drive", t.path.display()))?
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    let base = crate::base_files()
        .ok_or("the design's own files are not open, so no released design is built")?;
    let now = today::build_held(base.clone(), &ctx.root, &drive, &design.design)?;
    let with = today::build_held_with(
        base,
        &ctx.root,
        &drive,
        &design.design,
        Some(&today::One {
            group,
            version: &t.version,
            fingerprint: &t.fingerprint,
            path: &path,
            bytes: &bytes,
        }),
    )?;
    let before = graph_of(&now, ctx, &format!("design {}", design.version))?;
    let after = graph_of(
        &with,
        ctx,
        &format!("design {} with {group} {}", design.version, t.version),
    )?;
    // Inside the group: its release's blocks, the one the design holds, and
    // every node of it the design's own files hold.
    let mut inside: std::collections::BTreeSet<String> = std::iter::once(group.to_string())
        .chain(file.blocks.iter().map(|b| b.id.clone()))
        .chain(
            design
                .releases
                .iter()
                .filter(|r| r.group == group)
                .flat_map(|r| r.back.file.blocks.iter().map(|b| b.id.clone())),
        )
        .collect();
    inside.extend(
        today::design_nodes(&ctx.root, group)?
            .iter()
            .flat_map(|n| n.blocks.iter().map(|b| b.id.clone())),
    );
    let impact = today::impact(before, after, &declared_case(ctx), &|id| {
        inside.contains(id)
    })?;
    // A value is the group's it hangs under that a mount names, however
    // deep a heading inside it holds it; the nearest heading only where no
    // subsystem's mount is above it.
    let mounted: std::collections::BTreeSet<&str> = c
        .programme
        .file
        .mounts
        .iter()
        .chain(c.systems.iter().flat_map(|s| s.file.mounts.iter()))
        .map(|m| m.group_id.as_str())
        .filter(|g| !["systems", "programme"].contains(g))
        .collect();
    let mounted_over = |node: &str| -> Option<String> {
        let mut at = node.to_string();
        for _ in 0..64 {
            if mounted.contains(at.as_str()) {
                return Some(at);
            }
            let g = if after.find(&at).is_some() {
                after
            } else {
                before
            };
            let parent = g.nodes[g.find(&at)? as usize].parent;
            if parent.is_empty() {
                return None;
            }
            at = parent.to_string();
        }
        None
    };
    let moved = impact
        .moved
        .iter()
        .map(|m| design::Move {
            value: m.id.clone(),
            node: m.id.split('.').next().unwrap_or(&m.id).to_string(),
            group: mounted_over(m.id.split('.').next().unwrap_or(&m.id))
                .unwrap_or_else(|| m.group.clone()),
            before: m.before,
            after: m.after,
            before_why: m.before_why.clone(),
            after_why: m.after_why.clone(),
        })
        .collect();
    Ok(Previewed {
        path,
        bytes,
        version: t.version.clone(),
        design,
        moved,
        impact,
    })
}

/// `POST /v1/design/preview`.
pub(super) fn preview_route(params: &str, ctx: &Ctx) -> String {
    match previewing(params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

fn previewing(params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    let no_key = "no key is open: open yours, and the preview is written and signed as you";
    if me::with_key(|_, _| ()).is_none() {
        return Err(no_key.into());
    }
    let group = param(params, "group").map(decode).unwrap_or_default();
    let group = group.trim();
    if !vleo_files::breakdown::valid_id(group) {
        return Err(format!("{group:?} is not a group's id"));
    }
    let copy = copy::dir(ctx);
    let record = copy::record(&copy)?.ok_or(
        "a preview is written in your copy of the database, then put back: copy it from the drive first",
    )?;
    let report = built_from()
        .ok_or("the design open is not today's design: a preview is of a release in it")?;
    let drive = PathBuf::from(&report.drive);
    let c = chain_of(&drive, ctx)?;
    let canonical =
        std::fs::canonicalize(&drive).map_err(|e| format!("{}: {e}", drive.display()))?;
    if Path::new(&record.drive) != canonical {
        return Err(format!(
            "your copy came from {}, and today's design is built from {}: write the preview in a copy of the drive it is built from",
            record.drive,
            canonical.display()
        ));
    }
    // Who may write one is asked before two designs are built for it.
    let may = me::with_key(|_, me| {
        (
            me.name.clone(),
            c.programme.acts_as(&me.name, "system engineer"),
        )
    })
    .ok_or(no_key)?;
    if !may.1 {
        return Err(format!(
            "{} may not write a preview: the system engineer, or their deputy, writes one (W7)",
            may.0
        ));
    }
    let parents: Vec<&Registry> = std::iter::once(&c.programme)
        .chain(c.systems.iter())
        .collect();
    // A preview is never written over: where it goes is asked before two
    // designs are built, and again under the copy's lock before it is
    // written. It goes at the first place free; a place taken by one that
    // opens refuses it, and one taken by a file that no longer opens, as when
    // its writer stops being the system engineer, is passed for the next.
    let place = |g: &str, v: &str, d: &str| -> Result<String, String> {
        for n in 1..=999 {
            let rel = vleo_files::chain::preview_path_n(g, v, d, n);
            let mut taken = false;
            for (whose, at) in [
                ("your copy", copy.join(&rel)),
                ("the drive", drive.join(&rel)),
            ] {
                if !at.exists() {
                    continue;
                }
                taken = true;
                let opens = vleo_files::sqlite::read(&at)
                    .is_ok_and(|f| design::open_preview(&f, &parents, &c.programme, APP).is_ok());
                if opens {
                    return Err(format!(
                        "{whose} holds {rel} already, and it opens: a preview is never written over"
                    ));
                }
            }
            if !taken {
                return Ok(rel);
            }
        }
        Err(format!(
            "every place a preview of {g} {v} on design {d} is written is taken"
        ))
    };
    let p = previewed(group, ctx, &c, &place)?;
    let at = results::now_utc();
    let (f, who) = me::with_key(|key, me| {
        design::preview(
            &Whole {
                path: &p.path,
                bytes: &p.bytes,
            },
            &p.design,
            &p.moved,
            &parents,
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
    // Whom it names is what the file signed says, read back as it is read.
    let named = design::open_preview(&f, &parents, &c.programme, APP)
        .map_err(|e| e.message().to_string())?
        .named;
    let _held = vleo_files::save::lock(&copy).map_err(|e| e.message().to_string())?;
    let rel = place(group, &p.version, &p.design.version)?;
    let at_file = copy.join(&rel);
    if let Some(dir) = at_file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    vleo_files::sqlite::write(&f, &at_file).map_err(|e| e.message().to_string())?;
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("path", &rel);
    j.str_field("group", group);
    j.str_field("version", &p.version);
    j.str_field("based_on", &p.design.version);
    strings(&mut j, "named", &named);
    moved_json(&mut j, &p.moved);
    for (k, ran) in [("released", &p.impact.today), ("with", &p.impact.view)] {
        j.key(k).open_obj();
        j.num_field("ran", ran.ran as f64);
        j.key("blocked").open_arr();
        for (node, why) in &ran.blocked {
            j.open_obj();
            j.str_field("node", node);
            j.str_field("why", why);
            j.close_obj();
        }
        j.close_arr();
        j.close_obj();
    }
    j.str_field(
        "message",
        &format!(
            "{who} wrote the preview of {group} {} on design {}, in your copy: {rel}. {} It reaches them once it is put back on the drive.",
            p.version,
            p.design.version,
            if named.is_empty() {
                format!(
                    "It moves {} outside {group}, and names no subsystem's group to answer: what moves is the systems' or the programme's, shown and not answered.",
                    plural(p.moved.len(), "value")
                )
            } else {
                format!(
                    "It moves {} outside {group}, and names {} to answer.",
                    plural(p.moved.len(), "value"),
                    named.join(", ")
                )
            }
        ),
    );
    j.raw("}");
    Ok(j.0)
}

fn plural(n: usize, what: &str) -> String {
    format!("{n} {what}{}", if n == 1 { "" } else { "s" })
}

fn moved_json(j: &mut Json, moved: &[design::Move]) {
    j.key("moved").open_arr();
    for m in moved {
        j.open_obj();
        j.str_field("value", &m.value);
        j.str_field("group", &m.group);
        for (k, v, w) in [
            ("before", m.before, &m.before_why),
            ("after", m.after, &m.after_why),
        ] {
            match v {
                Some(x) => j.num_field(k, x),
                None => j.key(k).raw("null"),
            };
            if let Some(w) = w {
                j.str_field(&format!("{k}_why"), w);
            }
        }
        j.close_obj();
    }
    j.close_arr();
}

/// Every preview file in `integration/` under `folder`, by its path there,
/// or why the folder does not read: a folder that does not read is said,
/// never taken for one holding none.
pub(crate) fn previews_in(folder: &Path) -> Result<Vec<String>, String> {
    let top = folder.join("integration");
    let io = |at: &Path, e: std::io::Error| format!("{}: {e}", at.display());
    let entries = match std::fs::read_dir(&top) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(io(&top, e)),
    };
    let mut out = Vec::new();
    for d in entries {
        let d = d.map_err(|e| io(&top, e))?;
        if !d.path().is_dir() {
            continue;
        }
        let dir = d.file_name().to_string_lossy().into_owned();
        for e in std::fs::read_dir(d.path()).map_err(|e| io(&d.path(), e))? {
            let name = e
                .map_err(|e| io(&d.path(), e))?
                .file_name()
                .to_string_lossy()
                .into_owned();
            if name.starts_with("preview-") && name.ends_with(".vleo") {
                out.push(format!("integration/{dir}/{name}"));
            }
        }
    }
    out.sort();
    Ok(out)
}

/// A preview found in `integration/`: its path, whether the drive holds it
/// or the copy only, the file where it reads, and what it opens as, or why
/// it does not.
pub(crate) struct Listed {
    pub(crate) path: String,
    pub(crate) on_drive: bool,
    pub(crate) file: Option<vleo_files::model::File>,
    pub(crate) opened: Result<design::OpenedPreview, String>,
}

/// Every preview on the drive, and in the copy where the drive does not hold
/// it, each read back through the chain of `parents` (`design::open_preview`)
/// and held to the place its meta names; and each `integration/` that does
/// not read, said with whose it is.
pub(crate) fn previews(
    drive: &Path,
    copy: &Path,
    parents: &[&Registry],
    programme: &Registry,
) -> (Vec<Listed>, Vec<(bool, String)>) {
    let (mut out, mut unread): (Vec<Listed>, Vec<(bool, String)>) = (Vec::new(), Vec::new());
    for (folder, on_drive) in [(drive, true), (copy, false)] {
        if !on_drive && folder == drive {
            continue;
        }
        let paths = match previews_in(folder) {
            Ok(p) => p,
            Err(why) => {
                unread.push((on_drive, why));
                continue;
            }
        };
        for rel in paths {
            if !on_drive && out.iter().any(|l| l.path == rel) {
                continue;
            }
            let file =
                vleo_files::sqlite::read(&folder.join(&rel)).map_err(|e| e.message().to_string());
            let opened = file
                .clone()
                .and_then(|f| {
                    let o = design::open_preview(&f, parents, programme, APP)
                        .map_err(|e| e.message().to_string())?;
                    let place = vleo_files::chain::preview_path(&o.group, &o.version, &o.based_on);
                    if !vleo_files::chain::is_preview_path(&rel, &o.group, &o.version, &o.based_on)
                    {
                        return Err(format!(
                            "it says it is the preview of {} {} on design {}, so it is {place}",
                            o.group, o.version, o.based_on
                        ));
                    }
                    Ok(o)
                })
                .map_err(|why| format!("{rel} does not open through the chain: {why}"));
            out.push(Listed {
                path: rel,
                on_drive,
                file: file.ok(),
                opened,
            });
        }
    }
    (out, unread)
}

/// What each group the preview `p` names has said, from the drive's answers
/// alone, unanswered first; and, for each, whether an answer is in the copy
/// and not on the drive.
pub(crate) fn answered_on(
    p: &design::OpenedPreview,
    drive: &Path,
    copy: &Path,
    parents: &[&Registry],
    programme: &Registry,
) -> Vec<(String, design::Answered, bool)> {
    let read = |path: &str| {
        let at = drive.join(path);
        at.is_file()
            .then(|| vleo_files::sqlite::read(&at).map_err(|e| e.message().to_string()))
    };
    let mut said = design::answers(p, &read, parents, programme);
    said.sort_by_key(|(_, a)| !matches!(a, design::Answered::Unanswered));
    said.into_iter()
        .map(|(g, a)| {
            let at = vleo_files::answer::path_of(&p.group, &p.version, &g);
            let in_copy = !drive.join(&at).is_file() && copy.join(&at).is_file();
            (g, a, in_copy)
        })
        .collect()
}

/// The previews and their answers, into `j` as `pending` and `previews`:
/// each group whose release in today's design is not the one in the design
/// released now, and the preview of it on that design where one opens; and
/// every preview on the drive, and in the copy where the drive does not hold
/// it, read back, each with what each group it names has said on the drive,
/// unanswered first. A preview that does not open, or an `integration/` that
/// does not read, is listed with why, never passed over. Nothing is
/// computed: a preview is built only when asked for.
fn previews_into(j: &mut Json, ctx: &Ctx, drive: &Path, c: &me::Chain) {
    let copy = copy::dir(ctx);
    let released = released_now(drive, &c.programme);
    j.str_field(
        "previews_on",
        released.as_ref().map_or("", |d| d.version.as_str()),
    );
    if let Err(why) = &released {
        j.str_field("previews_said", why);
    }
    let parents: Vec<&Registry> = std::iter::once(&c.programme)
        .chain(c.systems.iter())
        .collect();
    let (listed, unread) = previews(drive, &copy, &parents, &c.programme);
    // Only what was decided is read here: the previews that do not open are
    // listed below as they are.
    let report = built_from();
    let held: Vec<(&str, &str)> = report
        .iter()
        .flat_map(|r| r.groups.iter())
        .filter_map(|g| {
            g.taken
                .as_ref()
                .map(|t| (g.id.as_str(), t.version.as_str()))
        })
        .collect();
    let on = released.as_ref().ok().map(|d| d.version.as_str());
    let decided = objections_now(drive, &copy, c, on, &held).decided;
    j.key("pending").open_arr();
    if let (Ok(d), Some(report)) = (&released, &report) {
        for g in &report.groups {
            let Some(t) = &g.taken else { continue };
            if d.releases
                .iter()
                .any(|r| r.group == g.id && r.fingerprint == t.fingerprint)
            {
                continue;
            }
            let of = listed.iter().find(|l| {
                l.opened.as_ref().is_ok_and(|p| {
                    p.group == g.id && p.version == t.version && p.based_on == d.version
                })
            });
            j.open_obj();
            j.str_field("group", &g.id);
            j.str_field("version", &t.version);
            j.str_field(
                "held",
                d.releases
                    .iter()
                    .find(|r| r.group == g.id)
                    .map_or("", |r| r.version.as_str()),
            );
            j.str_field("preview", of.map_or("", |l| l.path.as_str()));
            j.close_obj();
        }
    }
    j.close_arr();
    j.key("previews").open_arr();
    for (on_drive, why) in &unread {
        j.open_obj();
        j.str_field("path", "integration/");
        j.str_field("in", if *on_drive { "the drive" } else { "your copy" });
        j.bool_field("opens", false);
        j.str_field("why", &format!("integration/ does not read: {why}"));
        j.close_obj();
    }
    for l in &listed {
        j.open_obj();
        j.str_field("path", &l.path);
        j.str_field(
            "in",
            if l.on_drive {
                "the drive"
            } else {
                "your copy, not put back yet"
            },
        );
        match &l.opened {
            Err(why) => {
                j.bool_field("opens", false);
                j.str_field("why", why);
            }
            Ok(p) => {
                j.bool_field("opens", true);
                j.str_field("group", &p.group);
                j.str_field("version", &p.version);
                j.str_field("based_on", &p.based_on);
                j.str_field("writer", &p.writer);
                j.bool_field(
                    "stale",
                    released.as_ref().map_or(true, |d| d.version != p.based_on),
                );
                strings(j, "named", &p.named);
                moved_json(j, &p.moved);
                j.key("answers").open_arr();
                for (g, a, in_copy) in answered_on(p, drive, &copy, &parents, &c.programme) {
                    j.open_obj();
                    j.str_field("group", &g);
                    j.str_field("said", a.name());
                    match &a {
                        design::Answered::Object(b) => {
                            j.str_field("because", b);
                            // What was decided of it, or that nothing is.
                            let id = design::objection_id(&p.group, &p.version, &g);
                            j.str_field("objection", &id);
                            let found = decided.iter().find(|d| d.objection().id() == id);
                            let d = found.and_then(|d| d.decision());
                            j.str_field("decision", &d.map_or("undecided".into(), |d| d.said()));
                            // What a reallocation's why says to decide it.
                            if let Some(o) = found {
                                j.str_field(
                                    "reallocation_names",
                                    &design::reallocation_names(o.objection()),
                                );
                            }
                        }
                        design::Answered::Refused(w) => {
                            j.str_field("why", w);
                        }
                        _ => {}
                    }
                    j.bool_field("in_copy", in_copy);
                    j.close_obj();
                }
                j.close_arr();
            }
        }
        j.close_obj();
    }
    j.close_arr();
}

/// `GET /v1/design/previews`.
pub(super) fn previews_route(ctx: &Ctx) -> String {
    let Some(report) = built_from() else {
        return me::refusal(
            "the design open is the design's own files, not today's design: previews are of releases in today's design, built from the drive",
        );
    };
    let drive = PathBuf::from(&report.drive);
    let c = match chain_of(&drive, ctx) {
        Ok(c) => c,
        Err(why) => return me::refusal(&why),
    };
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    previews_into(&mut j, ctx, &drive, &c);
    j.raw("}");
    j.0
}

// ── The decision on an objection (F6-c items 5 and 6) ──

/// Every preview as it stands, from the drive and the copy, each with what
/// its groups said on the drive; why each preview that does not open, and
/// may be one the release must hear, does not; and every objection to one,
/// with what was decided of it, read from the issues of the drive and the
/// copy.
pub(crate) struct Objections {
    pub(crate) previews: Vec<design::Standing>,
    pub(crate) unread: Vec<String>,
    pub(crate) decided: Vec<design::Decided>,
}

/// The release and the design a preview's path names it is of and on,
/// `integration/<g>-<v>/preview-<g>-<v>-on-<design>.vleo`, when it names
/// them.
fn named_by(path: &str) -> Option<(String, String, String)> {
    let name = path.rsplit('/').next()?;
    let (gv, design) = name
        .strip_prefix("preview-")?
        .strip_suffix(".vleo")?
        .split_once("-on-")?;
    let (g, v) = gv.rsplit_once('-')?;
    // A design's version holds no `-`: one after it is the place's sequence
    // (F6's answer (a)).
    let design = design
        .rsplit_once('-')
        .filter(|(_, n)| n.parse::<u32>().is_ok())
        .map_or(design, |(d, _)| d);
    Some((g.to_string(), v.to_string(), design.to_string()))
}

/// A preview found, by its path: the release, its version and the design it
/// is on, as it opens, or why it does not.
type Seen<'a> = (&'a str, Result<(&'a str, &'a str, &'a str), &'a str>);

/// Why each preview of `seen` that does not open may be one the release
/// waits on: its path names a release today's design holds, `holding`, on
/// the design released now, `released`, or names nothing it can be read by.
/// One whose release and design another preview of `seen` opens for is
/// passed: that one is written again in its place, as when its writer
/// stopped being the system engineer.
fn not_opening(seen: &[Seen], released: Option<&str>, holding: &[(&str, &str)]) -> Vec<String> {
    let mut out = Vec::new();
    for (path, opened) in seen {
        let Err(why) = opened else { continue };
        let named = named_by(path);
        let live = named.as_ref().is_none_or(|(g, v, d)| {
            Some(d.as_str()) == released && holding.iter().any(|(hg, hv)| hg == g && hv == v)
        });
        let again = named.as_ref().is_some_and(|(g, v, d)| {
            seen.iter()
                .any(|(_, o)| *o == Ok((g.as_str(), v.as_str(), d.as_str())))
        });
        if live && !again {
            out.push(why.to_string());
        }
    }
    out
}

/// Every issue file of the drive's `issues/` and the copy's, the drive's
/// first, each read, by its name; one that does not read is left out here
/// and said with the red marks, where every issue file is read.
fn issue_files(drive: &Path, copy: &Path) -> Vec<vleo_files::model::File> {
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for dir in [drive.join("issues"), copy.join("issues")] {
        let mut names: Vec<PathBuf> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
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
            seen.push(name);
            if let Ok(f) = vleo_files::sqlite::read(&p) {
                out.push(f);
            }
        }
    }
    out
}

pub(crate) fn objections_now(
    drive: &Path,
    copy: &Path,
    c: &me::Chain,
    released: Option<&str>,
    holding: &[(&str, &str)],
) -> Objections {
    let parents: Vec<&Registry> = std::iter::once(&c.programme)
        .chain(c.systems.iter())
        .collect();
    let (listed, not_read) = previews(drive, copy, &parents, &c.programme);
    // A preview that does not open asks the release what one unanswered does
    // when its path says it is of a release today's design holds, on the
    // design released now, or says nothing it can be read by; and so does an
    // integration/ that does not read.
    let mut unread: Vec<String> = not_read
        .into_iter()
        .map(|(on_drive, why)| {
            format!(
                "{}'s integration/ does not read: {why}",
                if on_drive { "the drive" } else { "your copy" }
            )
        })
        .collect();
    let seen: Vec<Seen> = listed
        .iter()
        .map(|l| {
            (
                l.path.as_str(),
                l.opened
                    .as_ref()
                    .map(|p| (p.group.as_str(), p.version.as_str(), p.based_on.as_str()))
                    .map_err(String::as_str),
            )
        })
        .collect();
    unread.extend(not_opening(&seen, released, holding));
    let mut previews = Vec::new();
    let mut objections = Vec::new();
    for p in listed.iter().filter_map(|l| l.opened.as_ref().ok()) {
        let answers = answered_on(p, drive, copy, &parents, &c.programme);
        for (g, a, _) in &answers {
            if let design::Answered::Object(because) = a {
                // What the decision names: the answer on the drive, by its
                // digest.
                let at = drive.join(vleo_files::answer::path_of(&p.group, &p.version, g));
                let digest = vleo_files::sqlite::read(&at)
                    .ok()
                    .and_then(|f| vleo_files::chain::digest(&f, vleo_files::chain::FILE).ok())
                    .unwrap_or_default();
                objections.push(design::Objection {
                    group: p.group.clone(),
                    version: p.version.clone(),
                    answering: g.clone(),
                    because: because.clone(),
                    answer: digest,
                });
            }
        }
        previews.push(design::Standing {
            group: p.group.clone(),
            version: p.version.clone(),
            based_on: p.based_on.clone(),
            answers: answers.into_iter().map(|(g, a, _)| (g, a)).collect(),
        });
    }
    let issues = issue_files(drive, copy);
    Objections {
        previews,
        unread,
        decided: design::decisions(
            &objections,
            &issues,
            c.systems.as_ref().map(|r| &r.file),
            &parents,
            &c.programme,
        ),
    }
}

/// `POST /v1/design/decide`.
pub(super) fn decide_route(params: &str, ctx: &Ctx) -> String {
    match deciding(params, ctx) {
        Ok(j) => j,
        Err(why) => me::refusal(&why),
    }
}

fn deciding(params: &str, ctx: &Ctx) -> Result<String, String> {
    me::carries_session(params)?;
    let no_key = "no key is open: open yours, and the decision is written and signed as you";
    if me::with_key(|_, _| ()).is_none() {
        return Err(no_key.into());
    }
    let get = |k: &str| param(params, k).map(decode).unwrap_or_default();
    let (group, version, answering) = (get("group"), get("version"), get("answering"));
    // An allocation changed is a decision made by Reallocate and Put back,
    // never written here: what this says is how.
    let decide = match get("decision").trim() {
        "next" => Some(design::Decide::NextVersion),
        "anyway" => Some(design::Decide::ReleaseAnyway),
        "allocation" => None,
        other => {
            return Err(format!(
                "{other:?} is not a decision: next (ask for its next version), anyway (release anyway) or allocation (change an allocation)"
            ))
        }
    };
    let copy = copy::dir(ctx);
    let record = copy::record(&copy)?.ok_or(
        "a decision is written in your copy of the database, then put back: copy it from the drive first",
    )?;
    let report = built_from()
        .ok_or("the design open is not today's design: an objection is to a release in it")?;
    let drive = PathBuf::from(&report.drive);
    let c = chain_of(&drive, ctx)?;
    let canonical =
        std::fs::canonicalize(&drive).map_err(|e| format!("{}: {e}", drive.display()))?;
    if Path::new(&record.drive) != canonical {
        return Err(format!(
            "your copy came from {}, and today's design is built from {}: decide in a copy of the drive it is built from",
            record.drive,
            canonical.display()
        ));
    }
    let id = design::objection_id(group.trim(), version.trim(), answering.trim());
    let (t, _) = today(ctx)?;
    let d = t
        .standing
        .decided
        .iter()
        .find(|d| d.objection().id() == id)
        .ok_or_else(|| {
            format!("no answer on the drive objects as {id}: an objection is decided once its answer is on the drive")
        })?;
    if let Some(done) = d.decision() {
        return Err(format!("{id} is decided already: {}", done.said()));
    }
    if d.objection().answer.is_empty() {
        return Err(format!(
            "{id}'s answer on the drive does not read, so its digest cannot be named: nothing is decided"
        ));
    }
    let Some(decide) = decide else {
        return Err(format!(
            "an allocation changed decides {id} from the drive, and nothing is written here: Reallocate in the systems' branch, its why saying {:?}, then Put back the systems' file",
            design::reallocation_names(d.objection())
        ));
    };
    let parents: Vec<&Registry> = std::iter::once(&c.programme)
        .chain(c.systems.iter())
        .collect();
    let _held = copy::lock(&copy)?;
    let dir = copy.join("issues");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let number =
        crate::issues::highest(&dir).max(crate::issues::highest(&drive.join("issues"))) + 1;
    let at = results::now_utc();
    let (f, who) = me::with_key(|key, me| {
        design::decision_issue(
            d.objection(),
            decide,
            number as i64,
            &me.name,
            key,
            &at,
            APP,
            &parents,
            &c.programme,
        )
        .map(|f| (f, me.name.clone()))
        .map_err(|e| e.message().to_string())
    })
    .ok_or(no_key)??;
    let group_id = f.meta.get("group_id").cloned().unwrap_or_default();
    let name = format!("{number}-{group_id}.vissue");
    let path = dir.join(&name);
    if path.exists() {
        return Err(format!(
            "{} is already there, so nothing is written over it: decide again",
            path.display()
        ));
    }
    vleo_files::sqlite::write(&f, &path)
        .map_err(|e| format!("{}: {}", path.display(), e.message()))?;
    let to = f.issues[0].addressed_to.clone();
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("objection", &id);
    j.str_field("file", &format!("issues/{name}"));
    j.num_field("number", number as f64);
    j.str_field("addressed_to", &to);
    j.str_field(
        "message",
        &format!(
            "{who} decided {id}: {}, in issue {number} to {to}, in your copy as issues/{name}. It goes to the drive when issue files go back.",
            match decide {
                design::Decide::NextVersion => "ask for its next version",
                design::Decide::ReleaseAnyway => "release anyway",
            }
        ),
    );
    j.raw("}");
    Ok(j.0)
}

#[cfg(test)]
mod tests {
    use super::{engine_is_today_s, not_opening};

    #[test]
    fn a_preview_that_does_not_open_holds_the_release_unless_written_again() {
        let on = Some("2026.10.1");
        let holding = [("l3_solar", "1.1")];
        let first = "integration/l3_solar-1.1/preview-l3_solar-1.1-on-2026.10.1.vleo";
        let second = "integration/l3_solar-1.1/preview-l3_solar-1.1-on-2026.10.1-2.vleo";
        let dead = Err("its writer is not the system engineer");
        let opens = Ok(("l3_solar", "1.1", "2026.10.1"));
        // Of a release held, on the design released now, at either place.
        for at in [first, second] {
            assert_eq!(
                not_opening(&[(at, dead)], on, &holding),
                ["its writer is not the system engineer"],
                "{at}"
            );
        }
        // Named by nothing it can be read by.
        assert_eq!(
            not_opening(&[("integration/x/preview-of.vleo", dead)], on, &holding).len(),
            1
        );
        // Stale: another design, or a release today's design does not hold.
        for at in [
            "integration/l3_solar-1.1/preview-l3_solar-1.1-on-2026.9.1-2.vleo",
            "integration/l3_solar-1.0/preview-l3_solar-1.0-on-2026.10.1.vleo",
        ] {
            assert!(not_opening(&[(at, dead)], on, &holding).is_empty(), "{at}");
        }
        // Written again at the next place: the one that does not open is
        // passed.
        assert!(not_opening(&[(first, dead), (second, opens)], on, &holding).is_empty());
    }

    #[test]
    fn a_design_is_released_only_with_the_margins_of_the_engine_that_runs_it() {
        assert!(engine_is_today_s(Some("abc"), "abc").is_ok());
        let e = engine_is_today_s(Some("abc"), "def").unwrap_err();
        assert!(
            e.contains("the engine runs the design whose fingerprint is abc…, and today's design is now def…"),
            "{e}"
        );
        let e = engine_is_today_s(None, "def").unwrap_err();
        assert!(e.contains("not installed for today's design"), "{e}");
    }
}
