//! One classic script from the ES modules of `web/js`, for a page that is
//! opened from a file and so cannot load modules: the group and node pages
//! (`xtask group-app`). Each module becomes a scope that returns its exports,
//! in the order its imports need; the code is the tool's own, unchanged.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

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
        "/* Bundled by `cargo run -p xtask -- group-app` from web/js — generated, never edited. */\n\
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// The bundle holds every module the entry needs, dependencies first, and
    /// nothing in it is still an import or an export.
    #[test]
    fn the_bundle_is_every_module_in_order_and_no_module_syntax() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let b = bundle(&root, "group.js").unwrap();
        let at = |m: &str| {
            b.find(&format!("__m[\"{m}\"] = (function"))
                .unwrap_or_else(|| panic!("{m} is not in the bundle"))
        };
        for (before, after) in [
            ("dom.js", "group.js"),
            ("chart.js", "figures.js"),
            ("figures.js", "gview.js"),
            ("gview.js", "group.js"),
            ("depth.js", "group.js"),
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
        let from = at("gview.js");
        let comp = &b[from..from + 1 + b[from + 1..].find("\n__m[").unwrap_or(b.len() - from - 1)];
        let ret = comp
            .lines()
            .rev()
            .find(|l| l.starts_with("return {"))
            .unwrap_or("");
        for name in ["groupPage", "nodePage", "mountFigures"] {
            assert!(
                ret.contains(name),
                "gview's exports do not include {name}: {ret}"
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
}
