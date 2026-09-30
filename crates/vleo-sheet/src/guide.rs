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

use crate::escape::html as esc;
use crate::manual::{Check, Manual, Role, Section, Who};

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
                out.push_str("<code>");
                out.push_str(&after[1..1 + j]);
                out.push_str("</code>");
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
            out.push_str(if bold { "</b>" } else { "<b>" });
            bold = !bold;
        }
    }
    if bold {
        out.push_str("</b>");
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
            out.push_str(if i % 2 == 1 { "<i>" } else { "</i>" });
        }
        out.push_str(p);
    }
    if parts.len().is_multiple_of(2) {
        out.push_str("</i>");
    }
    out
}

fn paragraphs(text: &str) -> String {
    text.split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| {
            format!(
                "<p>{}</p>",
                inline(&p.split_whitespace().collect::<Vec<_>>().join(" "))
            )
        })
        .collect()
}

/// What running a command does, in the words a careful reader needs before
/// pasting it (E15).
fn check_note(c: Option<Check>) -> &'static str {
    match c {
        Some(Check::Exits) => "<span class=\"tag ok\">safe to run — only reads</span>",
        Some(Check::Fails) => {
            "<span class=\"tag refuse\">shows a refusal — it is meant to fail here</span>"
        }
        Some(Check::Serves) => "<span class=\"tag ok\">starts the tool</span>",
        Some(Check::Writes) => "<span class=\"tag writes\">changes files</span>",
        Some(Check::Ci) => "<span class=\"tag ok\">the pipeline runs this too</span>",
        Some(Check::Probe) => "<span class=\"tag ok\">asks the running tool</span>",
        None => "",
    }
}

fn kind_badges(kind: &str) -> String {
    kind.split('+')
        .map(str::trim)
        .map(|k| format!("<span class=\"kind k-{k}\">{k}</span>", k = esc(k)))
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
        let mut li = format!("<div class=\"say\">{}</div>", inline(&st.say));
        if let Some(ui) = &st.ui {
            li.push_str(&format!(
                "<div class=\"ui\">look for <span class=\"uilabel\">{}</span></div>",
                esc(ui)
            ));
        }
        if let Some(run) = &st.run {
            li.push_str(&format!(
                "<div class=\"cmd\"><code>{}</code><button class=\"copy\" type=\"button\" \
                 data-copy=\"{}\" aria-label=\"copy the command\">copy</button></div>{}",
                esc(run),
                esc(run),
                check_note(st.check)
            ));
            if let Some(why) = &st.why {
                li.push_str(&format!("<div class=\"why\">{}</div>", inline(why)));
            }
            if let Some(expect) = &st.expect {
                li.push_str(&format!(
                    "<div class=\"why\">it says: <code>{}</code></div>",
                    esc(expect)
                ));
            }
        }
        steps.push_str(&format!("<li>{li}</li>"));
    }
    format!(
        "<section class=\"sec\" id=\"{id}\" data-text=\"{text}\">\
         <h3>{title} {kinds}</h3>\
         <div class=\"answer\"><span class=\"lbl\">answer first</span>{answer}</div>\
         <div class=\"body d-lr\">{body}</div>\
         {steps}\
         </section>",
        id = anchor(&s.id),
        text = esc(&format!("{} {} {}", s.title, s.answer, s.body).to_lowercase()),
        title = esc(&s.title),
        kinds = kind_badges(&s.kind),
        answer = inline(&s.answer),
        body = paragraphs(&s.body),
        steps = if steps.is_empty() {
            String::new()
        } else {
            format!("<ol class=\"steps\">{steps}</ol>")
        }
    )
}

/// Whether a block written for `who` belongs in the guide for `role`.
fn for_role(who: Who, role: &str) -> bool {
    who == Who::Everyone || who.name() == role
}

/// The loop, drawn once, with this role's part lit (E6, E7: the whole first,
/// its parts named where they sit).
fn loop_svg(role: &str) -> String {
    let lane = |id: &str, y: i32, label: &str| {
        let on = if id == role { " on" } else { "" };
        format!(
            "<g class=\"lane{on}\"><rect x=\"4\" y=\"{y}\" width=\"712\" height=\"54\" rx=\"4\"/>\
             <text x=\"16\" y=\"{}\" class=\"who\">{label}</text></g>",
            y + 32
        )
    };
    let step = |x: i32, y: i32, w: i32, text: &str| {
        format!(
            "<g class=\"step\"><rect x=\"{x}\" y=\"{}\" width=\"{w}\" height=\"30\" rx=\"3\"/>\
             <text x=\"{}\" y=\"{}\" text-anchor=\"middle\">{text}</text></g>",
            y + 12,
            x + w / 2,
            y + 32
        )
    };
    let mut g = String::new();
    g.push_str(&lane("user", 4, "user"));
    g.push_str(&lane("maintainer", 64, "maintainer"));
    g.push_str(&lane("developer", 124, "developer"));
    g.push_str(&step(120, 4, 120, "fills a node form"));
    g.push_str(&step(470, 4, 130, "tries · approves"));
    g.push_str(&step(120, 64, 90, "take"));
    g.push_str(&step(230, 64, 110, "preview"));
    g.push_str(&step(360, 64, 90, "approve"));
    g.push_str(&step(470, 64, 90, "merge"));
    g.push_str(&step(580, 64, 110, "ship · share"));
    g.push_str(&step(230, 124, 360, "code a form needs · the tool itself"));
    format!(
        "<svg class=\"loop\" viewBox=\"0 0 720 184\" role=\"img\" aria-label=\"The loop: a user \
         fills a node form; the maintainer takes it, previews it, records the approval, merges and \
         ships; the user tries the preview and approves; a developer writes what needs code. \
         Your part is lit.\">{g}\
         <path class=\"flow\" d=\"M180 50 V64 M275 94 V118 M300 94 C 360 40 450 40 530 50 \
         M530 50 C 480 60 420 70 405 76\"/></svg>"
    )
}

fn role_intro(r: &Role) -> String {
    let li = |xs: &[String]| {
        xs.iter()
            .map(|x| format!("<li>{}</li>", inline(x)))
            .collect::<String>()
    };
    format!(
        "<section class=\"af\"><span class=\"lbl\">answer first <span class=\"tag decl\">declared · \
         the repository's rules</span></span><p>{answer}</p></section>\
         <div class=\"eyebrow\">your role, in four steps</div>\
         <div class=\"cgrid\">\
         <div class=\"card s1 d-lr\"><div class=\"n\">1</div><h3>Say it simply</h3><p>{simply}</p>\
         <p class=\"tag ill\">illustrative — an analogy, see step 3 for where it stops</p></div>\
         <div class=\"card s2\"><div class=\"n\">2</div><h3>Now the real thing</h3>{svg}\
         <h4>You do</h4><ul>{does}</ul></div>\
         <div class=\"card s3\"><div class=\"n\">3</div><h3>Where the simple picture breaks</h3>\
         <p>{breaks}</p><h4>You never</h4><ul class=\"never\">{never}</ul></div>\
         <div class=\"card s4 d-l\"><div class=\"n\">4</div><h3>Try it · predict first</h3>\
         <p>{predict}</p><details><summary>Say your answer, then open</summary><p>{reveal}</p>\
         </details></div>\
         </div>\
         <section class=\"wrong\"><span class=\"lbl\">common wrong idea</span>\
         <p class=\"w\">“{wrong}”</p><p>{right}</p></section>",
        answer = inline(&r.answer),
        simply = inline(&r.simply),
        svg = loop_svg(&r.id),
        does = li(&r.does),
        breaks = inline(&r.breaks),
        never = li(&r.never),
        predict = inline(&r.predict),
        reveal = inline(&r.reveal),
        wrong = esc(&r.wrong),
        right = inline(&r.right),
    )
}

/// Render the guide for one role. `version` is the release it describes.
pub fn render(m: &Manual, role: &str, version: &str) -> Result<String, String> {
    let r = m
        .roles
        .iter()
        .find(|r| r.id == role)
        .ok_or_else(|| format!("the manual describes no role '{role}'"))?;

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
        toc.push_str(&format!("<li class=\"tl\">{}</li>", esc(&l.title)));
        work.push_str(&format!(
            "<div class=\"layer\"><h2>{}</h2><div class=\"lede d-lr\">{}</div>",
            esc(&l.title),
            paragraphs(&l.lede)
        ));
        for s in secs {
            toc.push_str(&format!(
                "<li><a href=\"#{}\">{}</a></li>",
                anchor(&s.id),
                esc(&s.title)
            ));
            work.push_str(&section(s));
        }
        work.push_str("</div>");
    }

    let commands: String = m
        .commands
        .iter()
        .filter(|c| for_role(c.who, role))
        .map(|c| {
            format!(
                "<tr><td><code>{} {}</code></td><td>{}</td><td>{}</td></tr>",
                esc(if c.tool == "xtask" {
                    "cargo run -p xtask --"
                } else {
                    "vleo"
                }),
                esc(&c.usage),
                inline(&c.what),
                esc(&c.effect)
            )
        })
        .collect();
    let cannot: String = m
        .cannot
        .iter()
        .filter(|c| for_role(c.who, role))
        .map(|c| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                inline(&c.what),
                inline(&c.why),
                inline(&c.instead)
            )
        })
        .collect();
    let others: String = crate::manual::ROLES
        .iter()
        .filter(|o| **o != role)
        .map(|o| format!("<a href=\"{o}.html\">{o}</a>"))
        .collect::<Vec<_>>()
        .join(" · ");

    Ok(format!(
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>VLEO · {title} guide</title>
<meta name="generator" content="cargo run -p xtask -- guides — from docs/manual.toml; edit the manual, never this file">
<style>{css}</style></head>
<body data-depth="read">
<header class="top">
<div class="eyebrow">VLEO design tool · version {version} · the {role} guide</div>
<h1>Guide for the {lower}</h1>
<p class="kinds"><span class="kind k-tutorial">tutorial</span><span class="kind k-how-to">how-to</span><span class="kind k-reference">reference</span><span class="kind k-explanation">explanation</span> — each block below says which it is</p>
<div class="controls">
<span class="grp" role="group" aria-label="how much this page explains">depth
<button type="button" data-depth="learn">Learn</button><button type="button" data-depth="read" class="sel">Read</button><button type="button" data-depth="expert">Expert</button></span>
<input type="search" id="q" placeholder="find a step, a command, a word…" aria-label="find in this guide">
<span class="others">other guides: {others}</span>
</div>
<p class="depth-note">Learn keeps every step, including a question to answer before you look. Read shows the answer, the plain words and the real thing. Expert keeps the answers and the commands.</p>
</header>
<main>
{intro}
<div class="eyebrow">the work, step by step</div>
<div class="work">
<nav class="toc" aria-label="contents"><ul>{toc}</ul></nav>
<div class="secs">{work}<p class="none" hidden>Nothing in this guide matches.</p></div>
</div>
<div class="eyebrow">reference</div>
<section class="ref"><h3>Every command for this role <span class="kind k-reference">reference</span></h3>
<table><thead><tr><th>command</th><th>what it does</th><th>effect</th></tr></thead><tbody>{commands}</tbody></table></section>
{cannot_block}
</main>
<footer><p>Generated from <code>docs/manual.toml</code> by <code>cargo run -p xtask -- guides</code>. Every command and label above is checked against the tool by the repository's tests (<code>the_manual_is_true</code>, <code>tools/manual_check.py</code>) — edit the manual, never this file. The standard it follows is <code>docs/EXPLAINING.md</code>.</p></footer>
<script>{js}</script>
</body></html>
"#,
        title = esc(&r.title),
        lower = esc(&r.title.to_lowercase()),
        role = esc(role),
        version = esc(version),
        others = others,
        intro = role_intro(r),
        toc = toc,
        work = work,
        commands = commands,
        cannot_block = if cannot.is_empty() {
            String::new()
        } else {
            format!(
                "<section class=\"ref\"><h3>What you cannot do here, and where it is done \
                 <span class=\"kind k-reference\">reference</span></h3><table><thead><tr><th>you \
                 cannot</th><th>why</th><th>instead</th></tr></thead><tbody>{cannot}</tbody>\
                 </table></section>"
            )
        },
        css = CSS,
        js = JS,
    ))
}

const CSS: &str = include_str!("../assets/role_guide.css");

const JS: &str = include_str!("../assets/role_guide.js");

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
