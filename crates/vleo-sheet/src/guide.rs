//! The three role guides — `docs/roles/{user,maintainer,developer}.html`.
//!
//! GENERATED FROM THE MANUAL, NEVER WRITTEN BY HAND. The manual is the one
//! place a command, a step or a role is described, and it is held true by
//! `the_manual_is_true` (every name against the thing it names) and by
//! `tools/manual_check.py` (every command run as written). A guide written
//! separately would be a second description, and the first time the two
//! disagreed nobody would know which to believe. So a guide is a *view* of the
//! manual for one role, rendered by `cargo run -p xtask -- guides`, and the
//! pipeline's regeneration diff fails when a committed guide differs from what
//! the manual now says.
//!
//! EACH GUIDE FOLLOWS docs/EXPLAINING.md, the standard every surface here
//! does, laid out after *Eight Loops, One Beam*: the answer first (E1); the
//! role said simply, then the real thing, then where the simple picture breaks
//! (E2, E4); the common wrong idea corrected (E5); overview before detail (E6);
//! every block marked with its kind (E8); Learn · Read · Expert (E9); a worked
//! example (E10, in the maintainer's); predict before you look (E11); and every
//! command marked with what running it does (E15). One self-contained file each:
//! no network, no build step, opens from disk or from inside the kit.

use crate::manual::{Check, Manual, Role, Section, Who};
use crate::{Error, ErrorKind};

use crate::text::html as esc;

/// The guide's parts (`web/pages/guide.html`), read once.
fn parts() -> &'static crate::shell::Parts {
    static P: std::sync::OnceLock<crate::shell::Parts> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        crate::shell::Parts::parse(
            "web/pages/guide.html",
            include_str!("../../../web/pages/guide.html"),
        )
        .unwrap_or_else(|e| panic!("{e}"))
    })
}

/// A part as written.
fn t(name: &str) -> &'static str {
    parts().text(name)
}

/// A part with its slots filled.
fn f(name: &str, slots: &[(&str, &str)]) -> String {
    parts().fill(name, slots)
}

/// The manual's own light markup — `code`, **bold**, *italic*, a blank line
/// between paragraphs — the same rules the browser's Manual view applies.
fn inline(text: &str) -> String {
    let e = esc(text);
    let mut out = String::new();
    let mut rest = e.as_str();
    // `code` first, so nothing inside it is read as emphasis.
    while let Some(i) = rest.find('`') {
        let (before, after) = rest.split_at(i);
        out.push_str(&emphasis(before));
        match after[1..].find('`') {
            Some(j) => {
                out.push_str(&f("md-code", &[("text", &after[1..1 + j])]));
                rest = &after[2 + j..];
            }
            None => {
                out.push_str(&emphasis(after));
                rest = "";
            }
        }
    }
    out.push_str(&emphasis(rest));
    out
}

fn emphasis(s: &str) -> String {
    let mut out = String::new();
    let mut bold = false;
    let mut it = s.split("**").peekable();
    while let Some(part) = it.next() {
        out.push_str(&italic(part));
        if it.peek().is_some() {
            out.push_str(t(if bold { "b-close" } else { "b-open" }));
            bold = !bold;
        }
    }
    if bold {
        out.push_str(t("b-close"));
    }
    out
}

fn italic(s: &str) -> String {
    let parts: Vec<&str> = s.split('*').collect();
    if parts.len() < 3 {
        return s.to_string();
    }
    let mut out = String::new();
    for (i, p) in parts.iter().enumerate() {
        if i > 0 {
            out.push_str(t(if i % 2 == 1 { "i-open" } else { "i-close" }));
        }
        out.push_str(p);
    }
    if parts.len().is_multiple_of(2) {
        out.push_str(t("i-close"));
    }
    out
}

fn paragraphs(text: &str) -> String {
    text.split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| {
            f(
                "para",
                &[(
                    "text",
                    &inline(&p.split_whitespace().collect::<Vec<_>>().join(" ")),
                )],
            )
        })
        .collect()
}

/// What running a command does, in the words a careful reader needs before
/// pasting it (E15).
fn check_note(c: Option<Check>) -> &'static str {
    match c {
        Some(Check::Exits) => t("check-exits"),
        Some(Check::Fails) => t("check-fails"),
        Some(Check::Serves) => t("check-serves"),
        Some(Check::Writes) => t("check-writes"),
        Some(Check::Ci) => t("check-ci"),
        Some(Check::Probe) => t("check-probe"),
        None => "",
    }
}

fn kind_badges(kind: &str) -> String {
    kind.split('+')
        .map(str::trim)
        .map(|k| f("kind-badge", &[("kind", &esc(k))]))
        .collect()
}

fn anchor(id: &str) -> String {
    format!(
        "s-{}",
        id.replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "-")
    )
}

fn section(s: &Section) -> String {
    let mut steps = String::new();
    for st in &s.steps {
        let mut li = f("step-say", &[("text", &inline(&st.say))]);
        if let Some(ui) = &st.ui {
            li.push_str(&f("step-ui", &[("ui", &esc(ui))]));
        }
        if let Some(run) = &st.run {
            li.push_str(&f(
                "step-cmd",
                &[("run", &esc(run)), ("note", check_note(st.check))],
            ));
            if let Some(why) = &st.why {
                li.push_str(&f("step-why", &[("text", &inline(why))]));
            }
            if let Some(expect) = &st.expect {
                li.push_str(&f("step-expect", &[("text", &esc(expect))]));
            }
        }
        steps.push_str(&f("li", &[("text", &li)]));
    }
    f(
        "section",
        &[
            ("id", &anchor(&s.id)),
            (
                "text",
                &esc(&format!("{} {} {}", s.title, s.answer, s.body).to_lowercase()),
            ),
            ("title", &esc(&s.title)),
            ("kinds", &kind_badges(&s.kind)),
            ("answer", &inline(&s.answer)),
            ("body", &paragraphs(&s.body)),
            (
                "steps",
                &if steps.is_empty() {
                    String::new()
                } else {
                    f("steps", &[("steps", &steps)])
                },
            ),
        ],
    )
}

/// Whether a block written for `who` belongs in the guide for `role`.
fn for_role(who: Who, role: &str) -> bool {
    who == Who::Everyone || who.name() == role
}

/// The loop, drawn once, with this role's part lit (E6, E7: the whole first,
/// its parts named where they sit).
fn loop_svg(role: &str) -> String {
    let on = |id: &str| if id == role { " on" } else { "" };
    f(
        "loop",
        &[
            ("on_user", on("user")),
            ("on_maintainer", on("maintainer")),
            ("on_developer", on("developer")),
        ],
    )
}

fn role_intro(r: &Role) -> String {
    let li = |xs: &[String]| {
        xs.iter()
            .map(|x| f("li", &[("text", &inline(x))]))
            .collect::<String>()
    };
    f(
        "intro",
        &[
            ("answer", &inline(&r.answer)),
            ("simply", &inline(&r.simply)),
            ("svg", &loop_svg(&r.id)),
            ("does", &li(&r.does)),
            ("breaks", &inline(&r.breaks)),
            ("never", &li(&r.never)),
            ("predict", &inline(&r.predict)),
            ("reveal", &inline(&r.reveal)),
            ("wrong", &esc(&r.wrong)),
            ("right", &inline(&r.right)),
        ],
    )
}

/// Render the guide for one role. `version` is the release it describes.
pub fn render(m: &Manual, role: &str, version: &str) -> Result<String, Error> {
    let r = m.roles.iter().find(|r| r.id == role).ok_or_else(|| {
        Error::new(
            ErrorKind::Malformed,
            format!("the manual describes no role '{role}'"),
        )
    })?;

    // The work: every section for this role or for everyone, layer by layer.
    let mut toc = String::new();
    let mut work = String::new();
    for l in &m.layers {
        let secs: Vec<&Section> = l
            .sections
            .iter()
            .filter(|s| for_role(s.who, role))
            .collect();
        if secs.is_empty() {
            continue;
        }
        toc.push_str(&f("toc-layer", &[("title", &esc(&l.title))]));
        work.push_str(&f(
            "layer-open",
            &[("title", &esc(&l.title)), ("lede", &paragraphs(&l.lede))],
        ));
        for s in secs {
            toc.push_str(&f(
                "toc-item",
                &[("id", &anchor(&s.id)), ("title", &esc(&s.title))],
            ));
            work.push_str(&section(s));
        }
        work.push_str(t("layer-close"));
    }

    let commands: String = m
        .commands
        .iter()
        .filter(|c| for_role(c.who, role))
        .map(|c| {
            f(
                "command",
                &[
                    (
                        "tool",
                        &esc(t(if c.tool == "xtask" {
                            "tool-xtask"
                        } else {
                            "tool-vleo"
                        })),
                    ),
                    ("usage", &esc(&c.usage)),
                    ("what", &inline(&c.what)),
                    ("effect", &esc(&c.effect)),
                ],
            )
        })
        .collect();
    let cannot: String = m
        .cannot
        .iter()
        .filter(|c| for_role(c.who, role))
        .map(|c| {
            f(
                "cannot",
                &[
                    ("what", &inline(&c.what)),
                    ("why", &inline(&c.why)),
                    ("instead", &inline(&c.instead)),
                ],
            )
        })
        .collect();
    let others: String = crate::manual::ROLES
        .iter()
        .filter(|o| **o != role)
        .map(|o| {
            // The file keeps the role's id; the link reads as its title.
            let title = m
                .roles
                .iter()
                .find(|r| r.id == *o)
                .map_or(*o, |r| r.title.as_str());
            f(
                "other-guide",
                &[("role", o), ("title", &esc(&title.to_lowercase()))],
            )
        })
        .collect::<Vec<_>>()
        .join(t("others-sep"));

    let cannot_block = if cannot.is_empty() {
        String::new()
    } else {
        f("cannot-block", &[("rows", &cannot)])
    };
    let body = f(
        "body",
        &[
            ("version", &esc(version)),
            ("lower", &esc(&r.title.to_lowercase())),
            ("others", &others),
            ("intro", &role_intro(r)),
            ("toc", &toc),
            ("work", &work),
            ("commands", &commands),
            ("cannot_block", &cannot_block),
            ("js", JS),
        ],
    );
    Ok(crate::shell::fill(&crate::shell::Page {
        title: &f("title", &[("title", &r.title)]),
        head: &f("head", &[("css", CSS)]),
        body_attrs: t("body-attrs"),
        body: &body,
    }))
}

const CSS: &str = include_str!("../../../web/pages/guide.css");

const JS: &str = include_str!("../../../web/pages/guide.js");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_light_markup_is_the_manuals() {
        assert_eq!(
            inline("run `a <b>` now"),
            "run <code>a &lt;b&gt;</code> now"
        );
        assert_eq!(inline("**bold** and *it*"), "<b>bold</b> and <i>it</i>");
        assert_eq!(inline("`**not bold**`"), "<code>**not bold**</code>");
        assert_eq!(
            paragraphs("one\ntwo\n\nthree"),
            "<p>one two</p><p>three</p>"
        );
    }
}
