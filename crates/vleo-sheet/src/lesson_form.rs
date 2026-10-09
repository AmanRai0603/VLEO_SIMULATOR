//! A lesson's form: one HTML file the expert who knows a row opens anywhere,
//! fills, checks as they type, and sends back — the lesson's counterpart of a
//! node form (see [`crate::template`]).
//!
//! THE CHECK IN THE PAGE IS THE GATE'S. The form carries the checker every node
//! form carries (`web/method.wasm.gz`), which holds [`crate::lesson::report`],
//! and a table of the tree's rows, so a widget naming a row that is not there
//! is refused while the author types, by the same function the gate runs.
//!
//! Three blocks travel inside the file:
//!
//! | block | holds | read by |
//! |---|---|---|
//! | `vleo-lesson-json` | the lesson as the page edits it | the page, when the saved copy is opened again |
//! | `vleo-lesson` | the lesson as TOML, in a hidden textarea | `xtask lesson check|apply`, which writes it beside the row |
//! | `vleo-lesson-rows` | `node <id>` and the tree's rows | the check |
//!
//! Saving a filled copy rewrites the first two. Nothing in the page talks to a
//! network, and nothing in it writes a file anywhere but the reader's download.

use crate::lesson::{self, Lesson, CLAIMS, KINDS, STATIONS};
use crate::load::Tree;
use crate::model::Sheet;
use crate::{Error, ErrorKind};

/// The form for a row's lesson — its lesson as it stands, or an empty one to
/// start from.
pub fn document(sh: &Sheet, tree: &Tree) -> Result<String, Error> {
    let l = match lesson::of(sh) {
        Some(r) => r?,
        None => Lesson {
            node: sh.id.clone(),
            kind: "explanation".into(),
            ..Default::default()
        },
    };
    let toml_text = sh.lesson.clone().unwrap_or_default();
    Ok(page(sh, tree, &l, &toml_text))
}

/// The lesson a filled form carries: the TOML block, and the row it is for.
/// Accepts a bare `lesson.toml` too, with the row given.
pub fn from_file(text: &str, node: Option<&str>) -> Result<(String, String), Error> {
    if !text.contains("id=\"vleo-lesson\"") {
        let node = node.ok_or_else(|| {
            Error::new(
                ErrorKind::Malformed,
                "this is not a lesson form; for a bare lesson.toml, name its row with --for <node>",
            )
        })?;
        return Ok((node.to_string(), text.to_string()));
    }
    let block = |id: &str| -> Option<String> {
        let open = format!("id=\"{id}\">");
        let a = text.find(&open)? + open.len();
        let b = text[a..].find("</script>")? + a;
        Some(text[a..b].to_string())
    };
    let rows = block("vleo-lesson-rows")
        .ok_or_else(|| Error::new(ErrorKind::Malformed, "the form has lost its row table"))?;
    let from_form = rows
        .lines()
        .find_map(|l| l.strip_prefix("node "))
        .map(|s| s.trim().to_string())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::Malformed,
                "the form does not say which row it is for",
            )
        })?;
    if let Some(n) = node {
        if n != from_form {
            return Err(Error::new(
                ErrorKind::Malformed,
                format!("the form is for '{from_form}', not '{n}'"),
            ));
        }
    }
    // The lesson travels in a textarea, HTML-escaped: a textarea ends only at
    // its own closing tag, and nothing escaped can be one — whatever the
    // lesson says, it cannot end or open an element of the page.
    let open = "id=\"vleo-lesson\">";
    let a = text
        .find(open)
        .ok_or_else(|| Error::new(ErrorKind::Malformed, "the form has lost its lesson block"))?
        + open.len();
    let b = text[a..].find("</textarea>").ok_or_else(|| {
        Error::new(
            ErrorKind::Malformed,
            "the form's lesson block is not closed",
        )
    })? + a;
    Ok((from_form, unhe(&text[a..b])))
}

fn page(sh: &Sheet, tree: &Tree, l: &Lesson, toml_text: &str) -> String {
    let rows = format!("node {}\n{}", sh.id, lesson::rows_block(tree));
    let list = |v: &[&str]| v.join(",");
    let wasm = crate::template::base64(&crate::template::checker(tree));
    let (kinds, stations, claims) = (list(KINDS), list(STATIONS), list(CLAIMS));
    let json = lesson::json(l).replace('<', "\\u003c");
    let body = f(
        "body",
        &[
            ("label", &he(&sh.label)),
            ("id", &he(&sh.id)),
            ("subsystem", &he(&sh.subsystem)),
            ("owner", &he(&sh.owner)),
            ("question", &he(&sh.question)),
            ("kinds", &kinds),
            ("stations", &stations),
            ("claims", &claims),
            ("json", &json),
            ("toml", &he(toml_text)),
            ("rows", &rows),
            ("wasm", &wasm),
            ("js", LESSON_JS),
        ],
    );
    crate::shell::fill(&crate::shell::Page {
        title: &f("title", &[("label", &sh.label)]),
        head: &f("head", &[("css", PAGE_CSS), ("lesson_css", LESSON_CSS)]),
        body: &body,
        ..Default::default()
    })
}

fn unhe(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

fn he(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The form's parts (`web/pages/lesson-form.html`), read once.
fn parts() -> &'static crate::shell::Parts {
    static P: std::sync::OnceLock<crate::shell::Parts> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        crate::shell::Parts::parse(
            "web/pages/lesson-form.html",
            include_str!("../../../web/pages/lesson-form.html"),
        )
        .unwrap_or_else(|e| panic!("{e}"))
    })
}

/// A part with its slots filled.
fn f(name: &str, slots: &[(&str, &str)]) -> String {
    parts().fill(name, slots)
}
/// The node form's look, so the two forms read as one family.
const PAGE_CSS: &str = include_str!("../../../web/pages/node-form.css");
const LESSON_CSS: &str = include_str!("../../../web/pages/lesson-form.css");
/// The page's behaviour: draw the lesson as fields, check it with the gate's
/// own check, and save a copy with its blocks rewritten. No network.
const LESSON_JS: &str = include_str!("../../../web/pages/lesson-form.js");
