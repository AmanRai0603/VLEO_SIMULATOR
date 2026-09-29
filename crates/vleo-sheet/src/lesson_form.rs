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

/// The form for a row's lesson — its lesson as it stands, or an empty one to
/// start from.
pub fn document(sh: &Sheet, tree: &Tree) -> Result<String, String> {
    let l = match lesson::load(&sh.dir, &sh.id) {
        Some(r) => r?,
        None => Lesson {
            node: sh.id.clone(),
            kind: "explanation".into(),
            ..Default::default()
        },
    };
    let toml_text = std::fs::read_to_string(sh.dir.join(lesson::FILE)).unwrap_or_default();
    Ok(page(sh, tree, &l, &toml_text))
}

/// The lesson a filled form carries: the TOML block, and the row it is for.
/// Accepts a bare `lesson.toml` too, with the row given.
pub fn from_file(text: &str, node: Option<&str>) -> Result<(String, String), String> {
    if !text.contains("id=\"vleo-lesson\"") {
        let node = node.ok_or(
            "this is not a lesson form; for a bare lesson.toml, name its row with --for <node>",
        )?;
        return Ok((node.to_string(), text.to_string()));
    }
    let block = |id: &str| -> Option<String> {
        let open = format!("id=\"{id}\">");
        let a = text.find(&open)? + open.len();
        let b = text[a..].find("</script>")? + a;
        Some(text[a..b].to_string())
    };
    let rows = block("vleo-lesson-rows").ok_or("the form has lost its row table")?;
    let from_form = rows
        .lines()
        .find_map(|l| l.strip_prefix("node "))
        .map(|s| s.trim().to_string())
        .ok_or("the form does not say which row it is for")?;
    if let Some(n) = node {
        if n != from_form {
            return Err(format!("the form is for '{from_form}', not '{n}'"));
        }
    }
    // The lesson travels in a textarea, HTML-escaped: a textarea ends only at
    // its own closing tag, and nothing escaped can be one — whatever the
    // lesson says, it cannot end or open an element of the page.
    let open = "id=\"vleo-lesson\">";
    let a = text
        .find(open)
        .ok_or("the form has lost its lesson block")?
        + open.len();
    let b = text[a..]
        .find("</textarea>")
        .ok_or("the form's lesson block is not closed")?
        + a;
    Ok((from_form, unhe(&text[a..b])))
}

fn page(sh: &Sheet, tree: &Tree, l: &Lesson, toml_text: &str) -> String {
    let rows = format!("node {}\n{}", sh.id, lesson::rows_block(tree));
    let list = |v: &[&str]| v.join(",");
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{label} — lesson form</title>\n<style>\n{css}{lcss}</style>\n</head>\n<body>\n\
         <header class=\"nf-head\"><p class=\"nf-kicker\">VLEO design tool · lesson form</p>\n\
         <h1>{label}</h1>\n<p class=\"nf-id\"><code>{id}</code> · {subsystem} · owner {owner}</p>\n\
         <section class=\"nf-intro\"><p><b>Answer first.</b> Write how this row is taught: the plain \
         words, the real relation and where it comes from, where it stops being true, a slider to try \
         it with, and a question to check yourself. Nobody writes HTML and nobody writes a formula for \
         the page to compute — a widget names rows, and the tool computes them.</p>\n\
         <p class=\"nf-muted\">The row asks: {question}</p>\n\
         <p class=\"nf-muted\">The check below is the gate's own, run in this page. When it passes, \
         press <b>save a filled copy</b> and send that file back; a developer places it with \
         <code>cargo run -p xtask -- lesson apply &lt;file&gt;</code>.</p></section></header>\n\
         <div class=\"nf-bar\"><button class=\"nf-primary\" id=\"ls-save\" type=\"button\">save a filled copy</button>\
         <button id=\"ls-toml\" type=\"button\">download lesson.toml</button>\
         <span class=\"nf-count\" id=\"ls-state\"></span></div>\n\
         <main id=\"ls-main\"></main>\n\
         <section class=\"ls-check\" id=\"ls-check\"><h2>The check</h2><div id=\"ls-problems\"></div></section>\n\
         <script type=\"application/json\" id=\"vleo-lesson-schema\">{{\"kinds\":\"{kinds}\",\"stations\":\"{stations}\",\"claims\":\"{claims}\"}}</script>\n\
         <script type=\"application/json\" id=\"vleo-lesson-json\">{json}</script>\n\
         <textarea hidden id=\"vleo-lesson\">{toml}</textarea>\n\
         <script type=\"text/plain\" id=\"vleo-lesson-rows\">{rows}</script>\n\
         <script type=\"application/octet-stream\" id=\"vleo-method-wasm\">{wasm}</script>\n\
         <script>\n{js}</script>\n</body>\n</html>\n",
        label = he(&sh.label),
        id = he(&sh.id),
        subsystem = he(&sh.subsystem),
        owner = he(&sh.owner),
        question = he(&sh.question),
        css = PAGE_CSS,
        lcss = LESSON_CSS,
        kinds = list(KINDS),
        stations = list(STATIONS),
        claims = list(CLAIMS),
        json = lesson::json(l).replace('<', "\\u003c"),
        toml = he(toml_text),
        rows = rows,
        wasm = crate::template::base64(&crate::template::checker(tree)),
        js = LESSON_JS,
    )
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

/// The node form's look, so the two forms read as one family.
const PAGE_CSS: &str = include_str!("form_page/page.css");
const LESSON_CSS: &str = include_str!("lesson_page/lesson.css");
/// The page's behaviour: draw the lesson as fields, check it with the gate's
/// own check, and save a copy with its blocks rewritten. No network.
const LESSON_JS: &str = include_str!("lesson_page/lesson.js");
