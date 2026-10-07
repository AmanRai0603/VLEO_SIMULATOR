//! The readers' docs folder: every row's page and every lesson, read with no
//! tool running — from a shared drive or any internal web server.
//!
//! Three things make it work without the tool:
//!
//! - **One classic script.** A page opened from a file cannot load ES modules,
//!   so the modules a page needs (`web/js/readers.js` and what it imports) are
//!   bundled into `assets/vleo.js` by [`bundle`]: each module becomes a scope
//!   that returns its exports, in the order its imports need. The code is the
//!   tool's own, unchanged — the component library, the figure player, the
//!   chart — so a lesson here is drawn exactly as in the tool.
//! - **The engine in the page, and the design it runs.** `crates/vleo-kernel-wasm`,
//!   built here and carried in `assets/kernel.js` with the design beside it —
//!   the tree's sheets converted to their files, as the rows the page hands
//!   the engine (`vleo_files::rows::encode_design`) — answers a lesson's
//!   widgets with the same relations the tool runs, from the graph those files
//!   make. A row that reads reference data refuses by name.
//! - **The pages the tool already generates.** Each row's page is its
//!   generated fragment (`generated/fragments/<id>.html`); nothing is written
//!   twice.
//!
//! Nothing here is committed: the folder is built, like the kit, and shared.

use super::*;
use crate::pipeline::{OnStop, Run};

/// `xtask readers [--out <dir>]`.
pub(super) fn cmd_readers(root: &Path, args: &[&str]) -> Result<(), String> {
    let out = args
        .iter()
        .position(|a| *a == "--out")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/readers"));
    // Every row's page is its generated fragment. Without them the folder would
    // be 1396 pages each saying it has none — refused here, before the build.
    if !root.join("generated/fragments").is_dir() {
        return Err(
            "no generated/fragments: each row's page in the folder is its generated \
                    page — build them first with `cargo run -p xtask -- assemble`"
                .into(),
        );
    }
    let mut run = Run::start(root, "readers", args, 5);
    let retry = format!("cargo run -p xtask -- readers --out {}", out.display());
    let kernel = run.step(
        "build the engine for the browser",
        OnStop::new("nothing written", retry.clone()),
        || {
            let k = build_kernel(root)?;
            let said = format!("{} KB, compressed", k.len() / 1024);
            Ok((k, said))
        },
    )?;
    let script = run.step(
        "bundle the page script",
        OnStop::new("nothing written", retry.clone()),
        || {
            let b = bundle(root, "readers.js")?;
            let said = format!("{} KB, from web/js", b.len() / 1024);
            Ok((b, said))
        },
    )?;
    let tree = read(root)?;
    let design = run.step(
        "read the design the pages run",
        OnStop::new("nothing written", retry.clone()),
        || {
            let (files, _) = vleo_files::convert::read_folder(&root.join("design"))
                .map_err(|e| format!("the design folder does not open: {e}"))?;
            let n = files.len();
            let d = gzip(&vleo_files::rows::encode_design(&files))?;
            let said = format!("{n} files, {} KB compressed", d.len() / 1024);
            Ok((d, said))
        },
    )?;
    let (pages, lessons) = run.step(
        "write the pages",
        OnStop::new(
            format!(
                "{} may be partly written; it is rebuilt whole",
                out.display()
            ),
            retry.clone(),
        ),
        || {
            let (p, l) = write_folder(root, &tree, &out, &script, &kernel, &design)?;
            Ok(((p, l), format!("{p} row pages, {l} with a lesson")))
        },
    )?;
    run.step(
        "check every page has what it links",
        OnStop::new(
            format!("{} is written but incomplete", out.display()),
            retry,
        ),
        || {
            let missing = broken_links(&out)?;
            if missing.is_empty() {
                Ok(((), "every link and asset is in the folder".into()))
            } else {
                Err(format!("missing: {}", missing.join(", ")))
            }
        },
    )?;
    run.done(&format!(
        "readers: {} — {pages} rows, {lessons} lessons. Open index.html, or put the folder on a \
         shared drive or an internal web server.",
        out.display()
    ));
    Ok(())
}

/// The kernel compiled for the browser, gzipped with no name or time in the
/// header so the same build gives the same bytes.
fn build_kernel(root: &Path) -> Result<Vec<u8>, String> {
    let manifest = root.join("crates/vleo-kernel-wasm/Cargo.toml");
    let mut cmd = std::process::Command::new("cargo");
    cmd.args([
        "build",
        "--release",
        "--target",
        "wasm32-unknown-unknown",
        "--manifest-path",
    ])
    .arg(&manifest)
    .current_dir(root);
    if root.join("crates/vleo-kernel-wasm/Cargo.lock").exists() {
        cmd.arg("--locked");
    }
    if !cmd.status().map_err(|e| e.to_string())?.success() {
        return Err(
            "the engine did not build for the browser — is the wasm32-unknown-unknown \
                    target installed? `rustup target add wasm32-unknown-unknown`"
                .into(),
        );
    }
    let built = root.join(
        "crates/vleo-kernel-wasm/target/wasm32-unknown-unknown/release/vleo_kernel_wasm.wasm",
    );
    gzip(&fs::read(&built).map_err(|e| format!("{}: {e}", built.display()))?)
}

/// Bytes gzipped with no name or time in the header, so the same bytes give
/// the same file.
fn gzip(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new("gzip")
        .args(["-9", "-n", "-c"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("gzip could not be run: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("gzip took no input")?;
    let input = bytes.to_vec();
    // Written from its own thread, so gzip's output never waits on its input.
    let writer = std::thread::spawn(move || stdin.write_all(&input));
    let out = child
        .wait_with_output()
        .map_err(|e| format!("gzip could not be run: {e}"))?;
    writer
        .join()
        .map_err(|_| "gzip's input could not be written".to_string())?
        .map_err(|e| format!("gzip's input could not be written: {e}"))?;
    if !out.status.success() {
        return Err("gzip failed".into());
    }
    Ok(out.stdout)
}

/// One classic script from an ES module in `web/js` and every module it
/// imports: each module a scope returning its exports, dependencies first.
///
/// It reads the forms the face is written in — `import { a, b } from './x.js'`,
/// `export { a } from './x.js'`, and `export` before a `function`, `async
/// function`, `const`, `let` or `class` — and refuses anything else it meets
/// on an import or export line, rather than bundle something it did not
/// understand.
pub(crate) fn bundle(root: &Path, entry: &str) -> Result<String, String> {
    let dir = root.join("web/js");
    let mut order: Vec<String> = Vec::new();
    let mut seen = BTreeSet::new();
    fn visit(
        dir: &Path,
        name: &str,
        seen: &mut BTreeSet<String>,
        order: &mut Vec<String>,
        stack: &mut Vec<String>,
    ) -> Result<(), String> {
        if seen.contains(name) {
            return Ok(());
        }
        if stack.iter().any(|s| s == name) {
            return Err(format!("web/js: an import cycle through {name}"));
        }
        stack.push(name.to_string());
        let text = fs::read_to_string(dir.join(name)).map_err(|e| format!("web/js/{name}: {e}"))?;
        for (_, from) in imports(&text, name)? {
            visit(dir, &from, seen, order, stack)?;
        }
        stack.pop();
        seen.insert(name.to_string());
        order.push(name.to_string());
        Ok(())
    }
    visit(&dir, entry, &mut seen, &mut order, &mut Vec::new())?;
    let mut o = String::from(
        "/* Bundled by `cargo run -p xtask -- readers` from web/js — generated, never edited. */\n\
         (function () {\n'use strict';\nconst __m = {};\n",
    );
    for name in &order {
        let text = fs::read_to_string(dir.join(name)).map_err(|e| e.to_string())?;
        o.push_str(&format!("__m[{name:?}] = (function () {{\n"));
        o.push_str(&module_body(&text, name)?);
        o.push_str("})();\n");
    }
    o.push_str("})();\n");
    Ok(o)
}

/// The `(names, module)` of every import and re-export in a module.
fn imports(text: &str, name: &str) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for stmt in statements(text) {
        let s = stmt.trim_start();
        if s.starts_with("import ") || (s.starts_with("export {") && s.contains(" from ")) {
            let (names, from) = parse_from(s).ok_or_else(|| {
                format!("web/js/{name}: an import this bundler does not read: {s}")
            })?;
            out.push((names, from));
        }
    }
    Ok(out)
}

/// Import and export statements, each whole even when it spans lines; every
/// other line on its own.
fn statements(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut open: Option<String> = None;
    for line in text.lines() {
        if let Some(mut cur) = open.take() {
            cur.push('\n');
            cur.push_str(line);
            if line.contains(';') {
                out.push(cur);
            } else {
                open = Some(cur);
            }
            continue;
        }
        let t = line.trim_start();
        if (t.starts_with("import ") || t.starts_with("export {")) && !line.contains(';') {
            open = Some(line.to_string());
        } else {
            out.push(line.to_string());
        }
    }
    if let Some(cur) = open {
        out.push(cur);
    }
    out
}

fn parse_from(s: &str) -> Option<(String, String)> {
    let a = s.find('{')? + 1;
    let b = s.find('}')?;
    let names = s[a..b].split_whitespace().collect::<Vec<_>>().join(" ");
    let rest = &s[b..];
    let q = rest.find("'./")? + 3;
    let e = rest[q..].find('\'')? + q;
    Some((names, rest[q..e].to_string()))
}

/// A module's code as the body of a scope that returns its exports.
fn module_body(text: &str, name: &str) -> Result<String, String> {
    let mut body = String::new();
    let mut exported: Vec<String> = Vec::new();
    for stmt in statements(text) {
        let s = stmt.trim_start();
        if s.starts_with("import ") {
            let (names, from) = parse_from(s).ok_or("unread import")?;
            if names.contains(" as ") {
                return Err(format!(
                    "web/js/{name}: `as` in an import is not bundled: {s}"
                ));
            }
            body.push_str(&format!("const {{ {names} }} = __m[{from:?}];\n"));
        } else if s.starts_with("export {") {
            let (names, from) = parse_from(s).ok_or_else(|| {
                format!("web/js/{name}: an export this bundler does not read: {s}")
            })?;
            body.push_str(&format!("const {{ {names} }} = __m[{from:?}];\n"));
            exported.extend(
                names
                    .split(',')
                    .map(|n| n.trim().to_string())
                    .filter(|n| !n.is_empty()),
            );
        } else if let Some(rest) = stmt.strip_prefix("export ") {
            let decl = rest.trim_start();
            let after = ["async function ", "function ", "const ", "let ", "class "]
                .iter()
                .find_map(|k| decl.strip_prefix(k))
                .ok_or_else(|| {
                    format!("web/js/{name}: an export this bundler does not read: {s}")
                })?;
            let id: String = after
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '$')
                .collect();
            exported.push(id);
            body.push_str(rest);
            body.push('\n');
        } else {
            body.push_str(&stmt);
            body.push('\n');
        }
    }
    body.push_str(&format!("return {{ {} }};\n", exported.join(", ")));
    Ok(body)
}

/// Every row's page, the index, and the assets. Returns how many rows and how
/// many lessons.
fn write_folder(
    root: &Path,
    tree: &Tree,
    out: &Path,
    script: &str,
    kernel: &[u8],
    design: &[u8],
) -> Result<(usize, usize), String> {
    let _ = fs::remove_dir_all(out);
    let (assets, rows_dir) = (out.join("assets"), out.join("rows"));
    for d in [&assets, &rows_dir] {
        fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let w = |p: PathBuf, t: &str| fs::write(&p, t).map_err(|e| format!("{}: {e}", p.display()));
    fs::copy(root.join("web/app.css"), assets.join("app.css")).map_err(|e| e.to_string())?;
    // The face's type, beside the stylesheet that names it, with its licence.
    let fonts = assets.join("fonts");
    fs::create_dir_all(&fonts).map_err(|e| format!("{}: {e}", fonts.display()))?;
    for e in fs::read_dir(root.join("web/fonts"))
        .map_err(|e| format!("web/fonts: {e}"))?
        .flatten()
    {
        fs::copy(e.path(), fonts.join(e.file_name())).map_err(|e| e.to_string())?;
    }
    w(assets.join("vleo.js"), script)?;
    w(
        assets.join("kernel.js"),
        &format!(
            "/* The engine, compiled for the browser (crates/vleo-kernel-wasm), gzipped and base64-encoded. */\nwindow.VLEO_KERNEL = \"{}\";\n\
             /* The design it runs: the tree's sheets converted to their files, as their rows (vleo_files::rows::encode_design), gzipped and base64-encoded. */\nwindow.VLEO_DESIGN = \"{}\";\n",
            vleo_sheet::template::base64(kernel),
            vleo_sheet::template::base64(design)
        ),
    )?;
    let mut rows = 0;
    let mut lessons = Vec::new();
    for sh in tree.ordered() {
        let frag = fs::read_to_string(
            root.join("generated/fragments")
                .join(format!("{}.html", sh.id)),
        )
        .unwrap_or_else(|_| f("no-page", &[("id", &he(&sh.id))]));
        let lesson = match vleo_sheet::lesson::load(&sh.dir, &sh.id) {
            Some(Ok(l)) if vleo_sheet::lesson::problems(&l, tree).is_empty() => Some(l),
            Some(Ok(_)) | Some(Err(_)) => {
                return Err(format!(
                    "{}: its lesson does not pass its check — run the gate",
                    sh.id
                ))
            }
            None => None,
        };
        w(
            rows_dir.join(format!("{}.html", sh.id)),
            &row_page(sh, tree, &frag, lesson.as_ref()),
        )?;
        rows += 1;
        if let Some(l) = lesson {
            lessons.push((sh.id.clone(), l.title.clone(), l.answer.clone()));
        }
    }
    w(out.join("index.html"), &index_page(tree, &lessons))?;
    Ok((rows, lessons.len()))
}

/// The few lines of layout a readers' page needs that the tool's shell does
/// not: a header of its own and a column to read in.
const READERS_CSS: &str = include_str!("../../web/pages/readers.css");

/// A readers' page: the one page template (`web/page.html`), with the tool's
/// stylesheet linked and the folder's own header.
fn shell(title: &str, up: &str, body: &str, tail: &str) -> String {
    vleo_sheet::shell::fill(&vleo_sheet::shell::Page {
        title,
        head: &f("head", &[("up", up), ("css", READERS_CSS)]),
        body_attrs: t("body-attrs"),
        body: &f(
            "body",
            &[("up", up), ("body", body), ("tail", tail.trim_end())],
        ),
    })
}

fn row_page(
    sh: &vleo_sheet::model::Sheet,
    tree: &Tree,
    fragment: &str,
    lesson: Option<&vleo_sheet::lesson::Lesson>,
) -> String {
    let mut body = fragment.to_string();
    let mut tail = String::new();
    if let Some(l) = lesson {
        body.push_str(t("lesson-seg"));
        // The rows its widgets name, and only those: what a slider needs to
        // know of a row — its range, its unit and the factor to SI.
        let ids: BTreeSet<&String> = l
            .widgets
            .iter()
            .flat_map(|w| w.inputs.iter().chain(&w.outputs))
            .collect();
        let rows: Vec<String> = ids
            .iter()
            .filter_map(|id| tree.sheets.get(*id))
            .map(|s| {
                let u = vleo_units::Unit::from_name(&s.unit).unwrap_or(vleo_units::Unit::One);
                format!(
                    "{{\"id\":{},\"label\":{},\"unit\":{},\"lo\":{},\"hi\":{},\"factor\":{}}}",
                    js(&s.id),
                    js(&s.label),
                    js(u.symbol()),
                    num(s.lower),
                    num(s.upper),
                    num(u.si_factor())
                )
            })
            .collect();
        tail.push_str(&f(
            "lesson-tail",
            &[
                ("rows", &rows.join(",").replace('<', "\\u003c")),
                (
                    "lesson",
                    &vleo_sheet::lesson::json(l).replace('<', "\\u003c"),
                ),
            ],
        ));
    }
    tail.push_str(&f("script", &[("up", "../")]));
    shell(
        &f("title-row", &[("label", &sh.label)]),
        "../",
        &body,
        &tail,
    )
}

fn index_page(tree: &Tree, lessons: &[(String, String, String)]) -> String {
    let mut b = t("index-intro").to_string();
    if lessons.is_empty() {
        b.push_str(t("lessons-none"));
    } else {
        b.push_str(t("lessons-open"));
        for (id, title, answer) in lessons {
            b.push_str(&f(
                "lesson-item",
                &[("id", id), ("title", &he(title)), ("answer", &he(answer))],
            ));
        }
        b.push_str(t("list-close"));
    }
    for layer in 1..=4 {
        let rows: Vec<&vleo_sheet::model::Sheet> = tree
            .ordered()
            .into_iter()
            .filter(|s| s.layer == layer)
            .collect();
        if rows.is_empty() {
            continue;
        }
        b.push_str(&f("layer-open", &[("layer", &layer.to_string())]));
        for s in rows {
            b.push_str(&f(
                "row-item",
                &[
                    ("id", &he(&s.id)),
                    ("label", &he(&s.label)),
                    ("seeded", if s.is_seeded() { t("seeded") } else { "" }),
                ],
            ));
        }
        b.push_str(t("list-close"));
    }
    shell(t("title-index"), "", &b, &f("script", &[("up", "")]))
}

/// The readers' pages' parts (`web/pages/readers.html`), read once.
fn parts() -> &'static vleo_sheet::shell::Parts {
    static P: std::sync::OnceLock<vleo_sheet::shell::Parts> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        vleo_sheet::shell::Parts::parse(
            "web/pages/readers.html",
            include_str!("../../web/pages/readers.html"),
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

/// Every `href` and `src` a page points into the folder with, and every
/// `url(...)` a stylesheet does, that is not there.
fn broken_links(out: &Path) -> Result<Vec<String>, String> {
    let mut missing = BTreeSet::new();
    let mut pages = vec![out.join("index.html")];
    for dir in ["rows", "assets"] {
        for e in fs::read_dir(out.join(dir))
            .map_err(|e| format!("{dir}: {e}"))?
            .flatten()
        {
            let p = e.path();
            if dir == "rows" || p.extension().is_some_and(|x| x == "css") {
                pages.push(p);
            }
        }
    }
    for p in pages {
        let text = fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let base = p.parent().unwrap_or(out);
        for attr in ["href=\"", "src=\"", "url("] {
            for part in text.split(attr).skip(1) {
                let link = part
                    .split(['"', ')'])
                    .next()
                    .unwrap_or("")
                    .trim_matches('\'');
                if link.is_empty() || link.starts_with('#') || link.contains(':') {
                    continue;
                }
                if !base.join(link).exists() {
                    missing.insert(format!(
                        "{link} (from {})",
                        p.file_name().unwrap_or_default().to_string_lossy()
                    ));
                }
            }
        }
    }
    Ok(missing.into_iter().take(12).collect())
}

fn he(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn js(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn num(v: f64) -> String {
    if v.is_finite() {
        format!("{v:?}")
    } else {
        "null".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bundle holds every module the entry needs, dependencies first, and
    /// nothing in it is still an import or an export.
    #[test]
    fn the_bundle_is_every_module_in_order_and_no_module_syntax() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let b = bundle(&root, "readers.js").unwrap();
        let at = |m: &str| {
            b.find(&format!("__m[\"{m}\"] = (function"))
                .unwrap_or_else(|| panic!("{m} is not in the bundle"))
        };
        for (before, after) in [
            ("dom.js", "state.js"),
            ("chart.js", "figures.js"),
            ("figures.js", "components.js"),
            ("components.js", "readers.js"),
            ("depth.js", "readers.js"),
        ] {
            assert!(
                at(before) < at(after),
                "{before} comes after {after}, which needs it"
            );
        }
        for l in b.lines() {
            let t = l.trim_start();
            assert!(
                !t.starts_with("import ") && !t.starts_with("export "),
                "module syntax left in the bundle: {l}"
            );
        }
        let from = at("components.js");
        let comp = &b[from..from + 1 + b[from + 1..].find("\n__m[").unwrap_or(b.len() - from - 1)];
        let ret = comp
            .lines()
            .rev()
            .find(|l| l.starts_with("return {"))
            .unwrap_or("");
        for name in ["renderLesson", "useEngine", "answerFirst"] {
            assert!(
                ret.contains(name),
                "components' exports do not include {name}: {ret}"
            );
        }
    }

    /// Syntax the bundler does not read is refused, not bundled half-understood.
    #[test]
    fn an_import_it_does_not_read_is_refused() {
        assert!(module_body("import { a as b } from './x.js';\n", "t.js").is_err());
        assert!(module_body("export default function f() {}\n", "t.js").is_err());
        assert!(imports("import * as all from './x.js';\n", "t.js").is_err());
        let body = module_body(
            "import {\n  a,\n  b,\n} from './x.js';\nexport const c = a + b;\n",
            "t.js",
        )
        .unwrap();
        assert!(
            body.contains("const { a, b, } = __m[\"x.js\"];") && body.ends_with("return { c };\n"),
            "{body}"
        );
    }

    /// A readers' page is the one page template, filled, with the tool's
    /// stylesheet linked from where the page sits.
    #[test]
    fn a_readers_page_is_the_template_filled() {
        let p = shell(
            "a <row>",
            "../",
            "<p>body</p>",
            "<script src=\"x.js\"></script>\n",
        );
        vleo_sheet::shell::is_filled(vleo_sheet::shell::TEMPLATE, &p).unwrap();
        assert!(p.contains("<title>a &lt;row&gt;</title>"), "{p}");
        assert!(p.contains("href=\"../assets/app.css\""), "{p}");
        assert!(p.contains("<body class=\"readers\">"), "{p}");
    }

    /// The folder's last step finds a page's missing link, a missing script
    /// and a font its stylesheet names that is not there — and passes the
    /// same folder once they are.
    #[test]
    fn the_link_check_finds_what_a_page_names_and_the_folder_lacks() {
        let out = std::env::temp_dir().join(format!("vleo-readers-links-{}", std::process::id()));
        let _ = fs::remove_dir_all(&out);
        fs::create_dir_all(out.join("rows")).unwrap();
        fs::create_dir_all(out.join("assets/fonts")).unwrap();
        fs::write(
            out.join("index.html"),
            "<a href=\"rows/a.html\">a</a> <a href=\"#top\">top</a> <a href=\"https://x.org\">x</a>",
        )
        .unwrap();
        fs::write(
            out.join("rows/a.html"),
            "<a href=\"b.html\">b</a><script src=\"../assets/vleo.js\"></script>",
        )
        .unwrap();
        fs::write(
            out.join("assets/app.css"),
            "@font-face{src:url(fonts/f.woff2) format(\"woff2\")} i{background:url(\"data:image/png;base64,AA\")}",
        )
        .unwrap();
        let missing = broken_links(&out).unwrap();
        for want in ["b.html", "../assets/vleo.js", "fonts/f.woff2"] {
            assert!(
                missing.iter().any(|m| m.starts_with(want)),
                "{want} not found in {missing:?}"
            );
        }
        assert_eq!(missing.len(), 3, "{missing:?}");
        fs::write(out.join("rows/b.html"), "").unwrap();
        fs::write(out.join("assets/vleo.js"), "").unwrap();
        fs::write(out.join("assets/fonts/f.woff2"), "").unwrap();
        assert_eq!(broken_links(&out).unwrap(), Vec::<String>::new());
        let _ = fs::remove_dir_all(&out);
    }
}
