//! The manual the tool shows about itself.
//!
//! One file, `docs/manual.toml`, read by the daemon and served to the browser's
//! Manual view, and read by the two checks that hold it true. It lives in this
//! crate for the reason the form does: this is the one implementation both
//! faces read, so a second copy cannot quietly disagree with the first.
//!
//! A MANUAL IS PROSE, AND PROSE DRIFTS. This repository's own README already
//! quotes a row count four hundred out of date, an example that stopped
//! answering, and `cargo xtask` twelve hundred times without the alias that
//! makes it work. So the manual is not trusted to stay right — it is checked:
//!
//!   * `xtask/tests/the_manual_is_true.rs`, in `cargo test`, holds every NAME it
//!     uses against the thing it names — each command against the dispatch that
//!     runs it, each route against the router, each variable against the code
//!     that reads it, each label against the page that shows it, each folder
//!     against the tree — and in BOTH directions, so something added and not
//!     documented fails as surely as something documented and removed.
//!   * `tools/manual_check.py`, in the pipeline, RUNS every command it tells a
//!     person to paste, exactly as written, and holds each to the exit code and
//!     the output the manual says it gives.
//!
//! What this module does is load the file and refuse a malformed one by name:
//! a step that says to run something without saying what running it proves is a
//! step neither check can hold to anything.

use std::path::Path;

/// Who a section is for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Who {
    /// Reads the design, asks what-if questions, fills a sheet's words and
    /// numbers, proposes edits. Writes no Rust.
    User,
    /// Everything a user does, and the tree itself: rows, holes, fixtures,
    /// checks, merges, data.
    Developer,
    Everyone,
}

impl Who {
    pub fn name(&self) -> &'static str {
        match self {
            Who::User => "user",
            Who::Developer => "developer",
            Who::Everyone => "everyone",
        }
    }
    fn parse(s: &str) -> Option<Who> {
        match s {
            "user" => Some(Who::User),
            "developer" => Some(Who::Developer),
            "everyone" => Some(Who::Everyone),
            _ => None,
        }
    }
}

/// What running a step's command proves, and so what the pipeline does with it.
///
/// Never defaulted. A command in a manual is a claim — "paste this and you get
/// that" — and a claim with no stated test is the kind that goes stale without
/// anybody noticing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Check {
    /// Run it; it must exit 0, and print `expect` if one is given.
    Exits,
    /// Run it; it must exit NON-zero. The step is showing a refusal, and a
    /// refusal that stopped refusing is the drift.
    Fails,
    /// Start it; it must come up and answer. For the daemon.
    Serves,
    /// It changes files or configuration, so it is not run. `why` says what it
    /// changes. Its NAME is still checked against the real command.
    Writes,
    /// The pipeline already runs this exact command as one of its own steps.
    /// Checked by finding it, verbatim, in the workflow.
    Ci,
    /// A request to the running daemon. Run against the one `Serves` started.
    Probe,
}

impl Check {
    pub fn name(&self) -> &'static str {
        match self {
            Check::Exits => "exits",
            Check::Fails => "fails",
            Check::Serves => "serves",
            Check::Writes => "writes",
            Check::Ci => "ci",
            Check::Probe => "probe",
        }
    }
    fn parse(s: &str) -> Option<Check> {
        match s {
            "exits" => Some(Check::Exits),
            "fails" => Some(Check::Fails),
            "serves" => Some(Check::Serves),
            "writes" => Some(Check::Writes),
            "ci" => Some(Check::Ci),
            "probe" => Some(Check::Probe),
            _ => None,
        }
    }
}

/// One thing to do.
pub struct Step {
    /// What to do, in words. May carry `code` and **bold**.
    pub say: String,
    /// A label a person clicks or reads on the page. Must exist in `web/`.
    pub ui: Option<String>,
    /// A command to paste, exactly as it is to be typed.
    pub run: Option<String>,
    pub check: Option<Check>,
    /// Text the output must contain.
    pub expect: Option<String>,
    /// For a step that is not run: what it changes.
    pub why: Option<String>,
}

pub struct Section {
    pub id: String,
    pub title: String,
    pub who: Who,
    pub body: String,
    pub steps: Vec<Step>,
}

pub struct Layer {
    pub id: String,
    pub title: String,
    pub lede: String,
    pub sections: Vec<Section>,
}

/// One command of one of the two binaries.
pub struct Command {
    /// `xtask` or `vleo`.
    pub tool: String,
    pub name: String,
    /// Exactly as the binary's own `help` prints it — checked against it.
    pub usage: String,
    pub what: String,
    pub who: Who,
    /// `reads`, `writes` or `irreversible`.
    pub effect: String,
}

/// One route the daemon answers.
pub struct Route {
    pub method: String,
    pub path: String,
    pub what: String,
    /// Refused unless the daemon was started with `VLEO_ALLOW_WRITE=1`.
    pub writes: bool,
}

pub struct Env {
    pub name: String,
    pub what: String,
    pub default: String,
}

pub struct Folder {
    pub name: String,
    pub what: String,
    pub who: Who,
}

pub struct Doc {
    pub path: String,
    pub what: String,
}

/// Something a person might reasonably try to do somewhere, cannot, and where
/// it is done instead. The "instead" is what makes it an answer rather than a
/// wall.
pub struct Cannot {
    /// `browser` or `terminal`.
    pub place: String,
    pub what: String,
    pub why: String,
    pub instead: String,
    pub who: Who,
}

pub struct Manual {
    pub layers: Vec<Layer>,
    pub commands: Vec<Command>,
    pub routes: Vec<Route>,
    pub env: Vec<Env>,
    pub folders: Vec<Folder>,
    pub docs: Vec<Doc>,
    pub cannot: Vec<Cannot>,
}

/// Where the manual lives.
pub fn path(root: &Path) -> std::path::PathBuf {
    root.join("docs").join("manual.toml")
}

fn s(v: &toml::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(|x| x.to_string())
}

fn need(v: &toml::Value, key: &str, at: &str) -> Result<String, String> {
    match s(v, key) {
        Some(x) if !x.trim().is_empty() => Ok(x),
        _ => Err(format!("{at}: `{key}` is missing or blank")),
    }
}

fn who(v: &toml::Value, at: &str) -> Result<Who, String> {
    let w = need(v, "who", at)?;
    Who::parse(&w).ok_or_else(|| format!("{at}: who = \"{w}\" — one of user, developer, everyone"))
}

fn table<'a>(v: &'a toml::Value, key: &str) -> Vec<&'a toml::Value> {
    v.get(key)
        .and_then(|x| x.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

/// Whether a command is shown with a placeholder a person replaces, like
/// `<node>`. Such a command cannot be run as written, so it may not claim to be.
pub fn has_placeholder(cmd: &str) -> bool {
    let mut open = false;
    for c in cmd.chars() {
        match c {
            '<' => open = true,
            '>' if open => return true,
            ' ' => open = false,
            _ => {}
        }
    }
    false
}

/// Load the manual, refusing a malformed one by name.
pub fn load(root: &Path) -> Result<Manual, String> {
    let p = path(root);
    let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    parse(&text)
}

/// Parse the manual's text. Separate from `load` so a check can hand it a
/// deliberately broken one.
pub fn parse(text: &str) -> Result<Manual, String> {
    let v: toml::Value = text
        .parse()
        .map_err(|e| format!("docs/manual.toml is not TOML: {e}"))?;
    let mut ids = std::collections::BTreeSet::new();

    let mut layers = Vec::new();
    for l in table(&v, "layer") {
        let lid = need(l, "id", "a [[layer]]")?;
        let at = format!("layer '{lid}'");
        let mut sections = Vec::new();
        for sec in table(l, "section") {
            let sid = need(sec, "id", &format!("a section of {at}"))?;
            if !ids.insert(sid.clone()) {
                return Err(format!(
                    "section id '{sid}' is used twice — a link to it would be ambiguous"
                ));
            }
            let at = format!("section '{sid}'");
            let mut steps = Vec::new();
            for (n, st) in table(sec, "step").into_iter().enumerate() {
                let at = format!("{at}, step {}", n + 1);
                let run = s(st, "run");
                let check = match s(st, "check") {
                    Some(c) => Some(
                        Check::parse(&c)
                            .ok_or_else(|| format!("{at}: check = \"{c}\" is not one of exits, fails, serves, writes, ci, probe"))?,
                    ),
                    None => None,
                };
                // A command with no stated check is a claim nothing holds to
                // anything; a check with no command has nothing to check.
                match (&run, check) {
                    (Some(_), None) => {
                        return Err(format!(
                            "{at}: a command with no `check` — say whether running it exits, \
                             fails, serves, writes, is run by the pipeline, or probes the daemon"
                        ))
                    }
                    (None, Some(_)) => return Err(format!("{at}: a `check` with no `run`")),
                    _ => {}
                }
                if let (Some(r), Some(c)) = (&run, check) {
                    if has_placeholder(r)
                        && matches!(
                            c,
                            Check::Exits | Check::Fails | Check::Serves | Check::Probe
                        )
                    {
                        return Err(format!(
                            "{at}: `{r}` has a placeholder a person must replace, so it cannot \
                             be run as written — it cannot be `{}`",
                            c.name()
                        ));
                    }
                    if c == Check::Writes
                        && s(st, "why").map(|w| w.trim().is_empty()).unwrap_or(true)
                    {
                        return Err(format!(
                            "{at}: `{r}` is not run by the check because it writes — `why` must say what it changes"
                        ));
                    }
                }
                let ui = s(st, "ui");
                let say = need(st, "say", &at)?;
                steps.push(Step {
                    say,
                    ui,
                    run,
                    check,
                    expect: s(st, "expect"),
                    why: s(st, "why"),
                });
            }
            sections.push(Section {
                id: sid.clone(),
                title: need(sec, "title", &at)?,
                who: who(sec, &at)?,
                body: s(sec, "body").unwrap_or_default(),
                steps,
            });
        }
        if sections.is_empty() {
            return Err(format!("{at} has no sections"));
        }
        layers.push(Layer {
            id: lid.clone(),
            title: need(l, "title", &at)?,
            lede: s(l, "lede").unwrap_or_default(),
            sections,
        });
    }
    if layers.is_empty() {
        return Err("docs/manual.toml has no [[layer]]".into());
    }

    let mut commands = Vec::new();
    for c in table(&v, "command") {
        let name = need(c, "name", "a [[command]]")?;
        let at = format!("command '{name}'");
        let tool = need(c, "tool", &at)?;
        if tool != "xtask" && tool != "vleo" {
            return Err(format!("{at}: tool = \"{tool}\" — one of xtask, vleo"));
        }
        let effect = need(c, "effect", &at)?;
        if !matches!(effect.as_str(), "reads" | "writes" | "irreversible") {
            return Err(format!(
                "{at}: effect = \"{effect}\" — one of reads, writes, irreversible"
            ));
        }
        let usage = need(c, "usage", &at)?;
        if usage.split_whitespace().next() != Some(name.as_str()) {
            return Err(format!(
                "{at}: usage `{usage}` must begin with the command's own name"
            ));
        }
        commands.push(Command {
            tool,
            name,
            usage,
            what: need(c, "what", &at)?,
            who: who(c, &at)?,
            effect,
        });
    }

    let mut routes = Vec::new();
    for r in table(&v, "route") {
        let path = need(r, "path", "a [[route]]")?;
        let at = format!("route '{path}'");
        let method = need(r, "method", &at)?;
        if method != "GET" && method != "POST" {
            return Err(format!("{at}: method = \"{method}\" — GET or POST"));
        }
        routes.push(Route {
            method,
            path,
            what: need(r, "what", &at)?,
            writes: r.get("writes").and_then(|x| x.as_bool()).unwrap_or(false),
        });
    }

    let mut env = Vec::new();
    for e in table(&v, "env") {
        let name = need(e, "name", "an [[env]]")?;
        let at = format!("env '{name}'");
        env.push(Env {
            what: need(e, "what", &at)?,
            default: need(e, "default", &at)?,
            name,
        });
    }

    let mut folders = Vec::new();
    for f in table(&v, "folder") {
        let name = need(f, "name", "a [[folder]]")?;
        let at = format!("folder '{name}'");
        folders.push(Folder {
            what: need(f, "what", &at)?,
            who: who(f, &at)?,
            name,
        });
    }

    let mut docs = Vec::new();
    for d in table(&v, "doc") {
        let path = need(d, "path", "a [[doc]]")?;
        let at = format!("doc '{path}'");
        docs.push(Doc {
            what: need(d, "what", &at)?,
            path,
        });
    }

    let mut cannot = Vec::new();
    for c in table(&v, "cannot") {
        let what = need(c, "what", "a [[cannot]]")?;
        let at = format!("cannot '{what}'");
        let place = need(c, "place", &at)?;
        if place != "browser" && place != "terminal" {
            return Err(format!("{at}: place = \"{place}\" — browser or terminal"));
        }
        cannot.push(Cannot {
            place,
            why: need(c, "why", &at)?,
            // WHERE IT IS DONE INSTEAD IS NOT OPTIONAL. A "you cannot" with
            // nothing after it is a wall, and a person who meets a wall goes
            // looking for a way round.
            instead: need(c, "instead", &at)?,
            who: who(c, &at)?,
            what,
        });
    }

    Ok(Manual {
        layers,
        commands,
        routes,
        env,
        folders,
        docs,
        cannot,
    })
}

use crate::form::jq;

fn opt(v: &Option<String>) -> String {
    v.as_deref().map(jq).unwrap_or_else(|| "null".into())
}

fn arr<T>(items: &[T], f: impl Fn(&T) -> String) -> String {
    let parts: Vec<String> = items.iter().map(f).collect();
    format!("[{}]", parts.join(",\n"))
}

/// The manual as JSON, for the face.
///
/// Only what the file says. What is true of the running tool — whether it will
/// accept an edit, whose name an edit carries, how many rows there are — is
/// added by the daemon, because only the daemon knows it and a manual that
/// states it from a file is a manual that is wrong the day it is copied.
pub fn json(m: &Manual) -> String {
    let steps = |st: &[Step]| {
        arr(st, |x| {
            format!(
                "{{\"say\":{},\"ui\":{},\"run\":{},\"check\":{},\"expect\":{},\"why\":{}}}",
                jq(&x.say),
                opt(&x.ui),
                opt(&x.run),
                x.check
                    .map(|c| jq(c.name()))
                    .unwrap_or_else(|| "null".into()),
                opt(&x.expect),
                opt(&x.why)
            )
        })
    };
    let layers = arr(&m.layers, |l| {
        format!(
            "{{\"id\":{},\"title\":{},\"lede\":{},\"sections\":{}}}",
            jq(&l.id),
            jq(&l.title),
            jq(&l.lede),
            arr(&l.sections, |s| format!(
                "{{\"id\":{},\"title\":{},\"who\":{},\"body\":{},\"steps\":{}}}",
                jq(&s.id),
                jq(&s.title),
                jq(s.who.name()),
                jq(&s.body),
                steps(&s.steps)
            ))
        )
    });
    format!(
        "{{\"layers\":{layers},\n\"commands\":{},\n\"routes\":{},\n\"env\":{},\n\"folders\":{},\n\"docs\":{},\n\"cannot\":{}}}",
        arr(&m.commands, |c| format!(
            "{{\"tool\":{},\"name\":{},\"usage\":{},\"what\":{},\"who\":{},\"effect\":{}}}",
            jq(&c.tool),
            jq(&c.name),
            jq(&c.usage),
            jq(&c.what),
            jq(c.who.name()),
            jq(&c.effect)
        )),
        arr(&m.routes, |r| format!(
            "{{\"method\":{},\"path\":{},\"what\":{},\"writes\":{}}}",
            jq(&r.method),
            jq(&r.path),
            jq(&r.what),
            r.writes
        )),
        arr(&m.env, |e| format!(
            "{{\"name\":{},\"what\":{},\"default\":{}}}",
            jq(&e.name),
            jq(&e.what),
            jq(&e.default)
        )),
        arr(&m.folders, |f| format!(
            "{{\"name\":{},\"what\":{},\"who\":{}}}",
            jq(&f.name),
            jq(&f.what),
            jq(f.who.name())
        )),
        arr(&m.docs, |d| format!(
            "{{\"path\":{},\"what\":{}}}",
            jq(&d.path),
            jq(&d.what)
        )),
        arr(&m.cannot, |c| format!(
            "{{\"place\":{},\"what\":{},\"why\":{},\"instead\":{},\"who\":{}}}",
            jq(&c.place),
            jq(&c.what),
            jq(&c.why),
            jq(&c.instead),
            jq(c.who.name())
        )),
    )
}
