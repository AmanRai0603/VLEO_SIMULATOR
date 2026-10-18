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
//! The checks and the release are of today's design, built from the drive;
//! the red marks are the health map's on the default case, as Explore draws
//! it. Not here: the pointer `design/design.vleo` and the archive (F7),
//! previews and their answers (F6-c), the daily snapshot.

use super::*;
use vleo_files::design::{self, Asked, Held, Integration, Mark, Part, Taken, Whole};
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

/// Today's design, as the checks read it: every group with a release on the
/// drive, and every group the systems' file mounts without one.
fn today(ctx: &Ctx) -> Result<Today, String> {
    let report = built_from().ok_or(
        "the design open is the design's own files, not today's design: the design is released from today's design, built from the drive",
    )?;
    let drive = PathBuf::from(&report.drive);
    let released = design::latest_in(&drive.join("design")).map_err(|e| e.message().to_string())?;
    let mut parts = Vec::new();
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
    if let Ok(c) = me::registries(&drive, &copy::dir(ctx), "") {
        if let Some(sy) = &c.systems {
            for m in &sy.file.mounts {
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
        }
    }
    Ok(Today {
        marks: marks(ctx, &drive),
        drive,
        released,
        parts,
    })
}

/// The red marks of today's design: each closure that fails, is refused or
/// is blocked on the default case, raised when an issue open names it; and
/// each issue open on the drive or in the copy.
fn marks(ctx: &Ctx, drive: &Path) -> Vec<Mark> {
    let mut open: Vec<(String, vleo_files::model::Issue)> = Vec::new();
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
            if open.iter().any(|(n, _)| *n == name) {
                continue;
            }
            if let Ok(f) = vleo_files::sqlite::read(&p) {
                for i in f.issues.into_iter().filter(|i| i.closed_by.is_empty()) {
                    open.push((name.clone(), i));
                }
            }
        }
    }
    let g = vleo_modules::engine();
    let mut case = build_case("", ctx);
    case.target = g.nodes.first().map_or("", |n| n.id).to_string();
    case.mode = RunMode::All;
    let map = health(g, &case);
    let mut out = Vec::new();
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

/// The version after `latest` this month, as the page offers it.
fn next_version(latest: Option<&str>) -> String {
    let now = results::now_utc();
    let year = now.get(0..4).and_then(|y| y.parse().ok()).unwrap_or(2026);
    let month = now.get(5..7).and_then(|m| m.parse().ok()).unwrap_or(1);
    match latest.and_then(DesignVersion::parse) {
        Some(v) => v.next(year, month).to_string(),
        None => format!("{year}.{month}.1"),
    }
}

/// `GET /v1/design/integration`.
pub(super) fn integration_route(ctx: &Ctx) -> String {
    let t = match today(ctx) {
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

/// The bytes of the file at `path` and the file they read as, read twice so
/// they are one.
fn whole(path: &Path) -> Result<(Vec<u8>, vleo_files::model::File), String> {
    let read = || std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()));
    let bytes = read()?;
    let f = vleo_files::sqlite::read(path)
        .map_err(|e| format!("{}: {}", path.display(), e.message()))?;
    if read()? != bytes {
        return Err(format!(
            "{} changed while it was read: try again",
            path.display()
        ));
    }
    Ok((bytes, f))
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
    let t = today(ctx)?;
    let drive =
        std::fs::canonicalize(&t.drive).map_err(|e| format!("{}: {e}", t.drive.display()))?;
    if Path::new(&record.drive) != drive {
        return Err(format!(
            "your copy came from {}, and today's design is built from {}: release it in a copy of the drive it is built from",
            record.drive,
            drive.display()
        ));
    }
    let c =
        match me::registries(&t.drive, &copy, "") {
            Ok(c) => c,
            Err(me::NotRead::FirstUse(..)) => return Err(
                "this computer has not accepted this database's anchor: open your key on it first"
                    .into(),
            ),
            Err(me::NotRead::Why(why)) => return Err(why),
        };
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
    let asked = Asked {
        version: &version,
        previous: t.released.as_deref(),
        changed: &changed,
        marks: &t.marks,
        understood: &understood,
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
    let notes_at = copy.join("design/NOTES.md");
    let before = match std::fs::read_to_string(&notes_at) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(io(&notes_at, e)),
    };
    let body = before.strip_prefix(NOTES_HEAD).unwrap_or(&before);
    let mut entry = format!(
        "## {version}\n\nReleased by {who}, {at}, after {}.\n\n{}\n",
        t.released.as_deref().unwrap_or("none"),
        changed.trim()
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
    .map_err(|e| io(&notes_at, e))?;
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
    std::fs::create_dir_all(copy.join("readable")).map_err(|e| io(&copy.join("readable"), e))?;
    let mut files = vec![name.clone(), "design/NOTES.md".to_string()];
    for (n, csv) in &readable {
        let p = copy.join("readable").join(n);
        std::fs::write(&p, csv).map_err(|e| io(&p, e))?;
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
/// why on the default case, the engine's.
fn health_csv(ctx: &Ctx, version: &str) -> String {
    let g = vleo_modules::engine();
    let mut case = build_case("", ctx);
    case.target = g.nodes.first().map_or("", |n| n.id).to_string();
    case.mode = RunMode::All;
    let map = health(g, &case);
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
