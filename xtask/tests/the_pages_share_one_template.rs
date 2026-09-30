//! Every page the tool writes is the one template, filled — and the face
//! carries its own type.
//!
//! `web/page.html` is where the markup every page shares lives. That is only
//! true while no generator writes a page of its own beside it, so this holds
//! both halves:
//!
//!   every generator → the template   each page it writes is the template's
//!                                    own text, with its slots filled
//!   the code → the template          no source but the template opens an
//!                                    HTML document, so a new generator that
//!                                    writes its own is refused here, by name
//!
//! and every page's style, script and fixed markup is a file in `web/pages`,
//! never a string in a generator;
//!
//! and the fonts the face names (`web/app.css`) are in `web/fonts`, with
//! their licence, and nothing the face or the template names is fetched from
//! anywhere else.
//!
//! Each check is a function over what it reads, and the last tests hand each
//! one a deliberately wrong input and watch it refuse.

use std::path::{Path, PathBuf};
use vleo_sheet::shell;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read(p: &str) -> String {
    std::fs::read_to_string(root().join(p)).unwrap_or_else(|e| panic!("{p}: {e}"))
}

/// Source files — Rust, JavaScript, Python — that open an HTML document of
/// their own. The filler is the one that may name the element, in its tests;
/// the forms' "save a copy" serialises the page it is, which opens nothing.
/// Test folders are left out: a test may write a page to be refused.
fn own_documents(files: &[(String, String)]) -> Vec<String> {
    files
        .iter()
        .filter(|(p, text)| p != "crates/vleo-sheet/src/shell.rs" && text.contains("<html"))
        .map(|(p, _)| p.clone())
        .collect()
}

fn sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if !matches!(name, "target" | "tests" | "node_modules" | "reference")
                    && !name.starts_with('.')
                {
                    walk(&p, root, out);
                }
            } else if name.ends_with(".rs") || name.ends_with(".js") || name.ends_with(".py") {
                let rel = p
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                if let Ok(t) = std::fs::read_to_string(&p) {
                    out.push((rel, t));
                }
            }
        }
    }
    let r = root();
    let mut out = Vec::new();
    for d in ["crates", "xtask/src", "web", "tools"] {
        walk(&r.join(d), &r, &mut out);
    }
    out
}

/// Rust sources that carry a page's style or behaviour themselves, or take a
/// page's markup, style or script from anywhere but `web/`. Style is found by
/// its `:root` rule, behaviour by the DOM calls every page script makes.
fn page_code_in_rust(files: &[(String, String)], root: &Path) -> Vec<String> {
    let mut bad = Vec::new();
    for (p, text) in files.iter().filter(|(p, _)| p.ends_with(".rs")) {
        for needle in [":root{", ":root {", "addEventListener(", "querySelector("] {
            if text.contains(needle) {
                bad.push(format!(
                    "{p} carries page code (`{needle}`) — it belongs in web/pages"
                ));
            }
        }
        for part in text.split("include_str!(\"").skip(1) {
            let target = &part[..part.find('"').unwrap_or(0)];
            if ![".css", ".js", ".html"].iter().any(|x| target.ends_with(x)) {
                continue;
            }
            let at = root.join(p).parent().unwrap().join(target);
            let under_web = at
                .canonicalize()
                .map(|a| a.starts_with(root.join("web")))
                .unwrap_or(false);
            if !under_web {
                bad.push(format!("{p} takes {target} from outside web/"));
            }
        }
    }
    bad
}

/// The markup a Rust source writes itself: every string literal that holds a
/// tag. For a generator whose every tag is in its parts file, there is none.
fn markup_in_strings(src: &str) -> Vec<String> {
    let mut found = Vec::new();
    let code = src.split("#[cfg(test)]").next().unwrap_or(src);
    for line in code.lines() {
        let l = line.trim_start();
        if l.starts_with("//") {
            continue;
        }
        let mut in_str = false;
        let mut lit = String::new();
        let mut chars = l.chars().peekable();
        while let Some(c) = chars.next() {
            match (in_str, c) {
                (false, '\'') => {
                    // a char literal, such as '"' — step over it
                    if chars.peek() == Some(&'\\') {
                        chars.next();
                    }
                    chars.next();
                    chars.next();
                }
                (false, '"') => {
                    in_str = true;
                    lit.clear();
                }
                (true, '\\') => {
                    chars.next();
                }
                (true, '"') => {
                    in_str = false;
                    let tagged = lit.char_indices().any(|(i, ch)| {
                        ch == '<'
                            && lit[i + 1..]
                                .starts_with(|n: char| n.is_ascii_lowercase() || n == '/')
                    });
                    if tagged {
                        found.push(lit.clone());
                    }
                }
                (true, ch) => lit.push(ch),
                _ => {}
            }
        }
    }
    found
}

/// What is wrong with the face's fonts: a font the stylesheet names that is
/// not in the folder, a font in the folder nobody names, a licence missing,
/// or anything fetched from elsewhere.
fn font_problems(css: &str, template: &str, folder: &[String]) -> Vec<String> {
    let mut bad = Vec::new();
    let mut named = Vec::new();
    for part in css.split("url(").skip(1) {
        let link = part
            .split(')')
            .next()
            .unwrap_or("")
            .trim_matches(['"', '\'']);
        if link.starts_with("data:") {
            continue;
        }
        match link.strip_prefix("fonts/") {
            Some(f) if folder.iter().any(|x| x == f) => named.push(f.to_string()),
            Some(f) => bad.push(format!(
                "app.css names fonts/{f}, which web/fonts does not hold"
            )),
            None => bad.push(format!("app.css fetches {link}, which is not in web/fonts")),
        }
    }
    for f in folder.iter().filter(|f| f.ends_with(".woff2")) {
        if !named.contains(f) {
            bad.push(format!("web/fonts/{f} is carried and no rule names it"));
        }
    }
    if !folder.iter().any(|f| f == "OFL.txt") {
        bad.push("web/fonts carries fonts without their licence (OFL.txt)".into());
    }
    for (what, text) in [("app.css", css), ("page.html", template)] {
        for remote in ["http://", "https://", "@import"] {
            if text.contains(remote) {
                bad.push(format!("{what} reaches outside the tool: `{remote}`"));
            }
        }
    }
    bad
}

fn the_fonts() -> Vec<String> {
    std::fs::read_dir(root().join("web/fonts"))
        .expect("web/fonts")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn no_source_but_the_template_opens_a_page() {
    let own = own_documents(&sources());
    assert!(
        own.is_empty(),
        "these write an HTML document of their own instead of filling web/page.html \
         (vleo_sheet::shell::fill): {own:?}"
    );
}

#[test]
fn every_page_a_generator_writes_is_the_template_filled() {
    let tree = vleo_sheet::load::load_all(&root()).expect("the tree loads");
    let sh = tree.sheets.get("sw_ap_design").expect("sw_ap_design");
    let m = vleo_sheet::manual::load(&root()).expect("the manual loads");
    let mut pages = vec![
        ("the node form", vleo_sheet::template::document(sh, &tree)),
        (
            "the new-node form",
            vleo_sheet::template::document_new(&tree),
        ),
        (
            "the lesson form",
            vleo_sheet::lesson_form::document(sh, &tree).expect("lesson form"),
        ),
    ];
    for role in vleo_sheet::manual::ROLES {
        pages.push((
            "a role guide",
            vleo_sheet::guide::render(&m, role, "0.0.0").expect("guide"),
        ));
        // And the guides as committed, which the tool ships.
        pages.push((
            "a committed guide",
            read(&format!("docs/roles/{role}.html")),
        ));
    }
    for (what, p) in pages {
        if let Err(e) = shell::is_filled(shell::TEMPLATE, &p) {
            panic!("{what}: {e}");
        }
    }
}

#[test]
fn every_page_s_markup_style_and_script_lives_in_web() {
    let r = root();
    let all = sources();
    let bad = page_code_in_rust(&all, &r);
    assert!(bad.is_empty(), "{bad:#?}");
    // And nothing in web/pages is a leftover no generator reads.
    let rust: String = all
        .iter()
        .filter(|(p, _)| p.ends_with(".rs"))
        .map(|(_, t)| t.as_str())
        .collect();
    for e in std::fs::read_dir(r.join("web/pages"))
        .expect("web/pages")
        .flatten()
    {
        let name = e.file_name().to_string_lossy().into_owned();
        assert!(
            rust.contains(&format!("web/pages/{name}\")")),
            "web/pages/{name} is read by no generator"
        );
    }
}

#[test]
fn a_row_page_is_its_parts_and_nothing_else() {
    let src = read("crates/vleo-sheet/src/page.rs");
    let own = markup_in_strings(&src);
    assert!(
        own.is_empty(),
        "crates/vleo-sheet/src/page.rs writes markup of its own — it belongs in \
         web/pages/row.html: {own:#?}"
    );
    // And every part there is one the generator asks for: by its name, or —
    // for a family chosen by the data, such as a claim's title — by the
    // family's prefix.
    let text: &'static str = Box::leak(read("web/pages/row.html").into_boxed_str());
    let parts = shell::Parts::parse("web/pages/row.html", text).expect("row.html reads");
    for name in parts.names() {
        let family = ["claim-title-", "kind-title-", "tier-blind-"]
            .iter()
            .find(|p| name.starts_with(*p));
        let used = match family {
            Some(p) => src.contains(&format!("\"{p}")),
            None => src.contains(&format!("\"{name}\"")),
        };
        assert!(
            used,
            "web/pages/row.html has a part `{name}` the generator never uses"
        );
    }
}

#[test]
fn the_tools_own_page_begins_as_the_template_does() {
    let t = shell::fill(&shell::Page::default());
    let head = &t[..t.find("<title>").unwrap()];
    assert!(
        read("web/index.html").starts_with(head),
        "web/index.html no longer begins as web/page.html does:\n{head}"
    );
}

#[test]
fn the_face_carries_its_own_type() {
    let bad = font_problems(&read("web/app.css"), shell::TEMPLATE, &the_fonts());
    assert!(bad.is_empty(), "{bad:#?}");
}

// --- each check, handed something wrong ---------------------------------------

#[test]
fn a_generator_writing_its_own_page_is_named() {
    let files = vec![
        ("fine.rs".to_string(), "shell::fill(&page)".to_string()),
        (
            "own.rs".to_string(),
            "o.push_str(\"<!doctype html>\\n<html lang=\\\"en\\\">\")".to_string(),
        ),
    ];
    assert_eq!(own_documents(&files), vec!["own.rs".to_string()]);
}

#[test]
fn page_code_left_in_rust_is_named() {
    let r = root();
    let files = vec![
        (
            "xtask/src/fine.rs".to_string(),
            "include_str!(\"../../web/pages/readers.css\")".to_string(),
        ),
        (
            "xtask/src/css.rs".to_string(),
            "const C: &str = \":root{--ink:#000}\";".to_string(),
        ),
        (
            "xtask/src/js.rs".to_string(),
            "\"b.addEventListener('click', f)\"".to_string(),
        ),
        (
            "xtask/src/away.rs".to_string(),
            "include_str!(\"page/form.css\")".to_string(),
        ),
    ];
    let bad = page_code_in_rust(&files, &r);
    assert_eq!(bad.len(), 3, "{bad:#?}");
    for f in ["css.rs", "js.rs", "away.rs"] {
        assert!(bad.iter().any(|b| b.contains(f)), "{f} not named: {bad:#?}");
    }
}

#[test]
fn markup_written_in_a_string_is_found() {
    let src = r#"
        // a comment with <b>markup</b> is not a string
        let q = '"';
        o.push_str(t("tabs-open"));
        o.push_str("<p class=\"x\">hi</p>");
        let a = format!("{} < {}", 1, 2);
        let b = "a </i> close";
    "#;
    let found = markup_in_strings(src);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found[0].contains("<p"), "{found:#?}");
    assert!(found[1].contains("</i>"), "{found:#?}");
}

#[test]
fn a_font_missing_unnamed_unlicensed_or_remote_is_refused() {
    let folder = the_fonts();
    let css = read("web/app.css");
    // A font the stylesheet names that is not there.
    let gone: Vec<String> = folder
        .iter()
        .filter(|f| !f.contains("700"))
        .cloned()
        .collect();
    assert!(font_problems(&css, "", &gone)
        .iter()
        .any(|e| e.contains("does not hold")));
    // A font carried that nothing names.
    let mut extra = folder.clone();
    extra.push("stray.woff2".into());
    assert!(font_problems(&css, "", &extra)
        .iter()
        .any(|e| e.contains("stray")));
    // The licence left behind.
    let unlicensed: Vec<String> = folder.iter().filter(|f| *f != "OFL.txt").cloned().collect();
    assert!(font_problems(&css, "", &unlicensed)
        .iter()
        .any(|e| e.contains("licence")));
    // Fetched from the internet.
    let remote = css.replace(
        "url(fonts/ibm-plex-mono-latin-400-normal.woff2)",
        "url(https://fonts.example/plex.woff2)",
    );
    assert!(font_problems(&remote, "", &folder)
        .iter()
        .any(|e| e.contains("outside")));
    let linked = "<link rel=\"stylesheet\" href=\"https://fonts.example/css\">";
    assert!(font_problems(&css, linked, &folder)
        .iter()
        .any(|e| e.contains("page.html")));
}
