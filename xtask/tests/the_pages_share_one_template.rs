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
