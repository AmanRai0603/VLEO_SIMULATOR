//! The group application and the two documents of the group folder pattern,
//! all built from one source: `groups/SPEC.toml`.
//!
//! `web/group.html` is the page a group opens their folder in. It is one file
//! — the tool's stylesheet with its fonts inlined, the pattern as JSON, and
//! the face's modules bundled — so it runs from a double-click, offline, from
//! a Google Drive folder or an email attachment alike. `docs/GROUP_FOLDER.md`
//! is the pattern for a person to read, and `groups/skill/vleo-group-folder/SKILL.md` the same
//! rules for an assistant that helps a member fill their folder. Each is
//! generated here and nowhere else, so the rule a group is told, the rule an
//! assistant follows and the rule the page checks are one text. `--check`
//! says whether the committed three are current; the gate runs it.

use super::*;

const SPEC: &str = "groups/SPEC.toml";
const OUT: &str = "web/group.html";
const NODE_OUT: &str = "web/node.html";
const DOC: &str = "docs/GROUP_FOLDER.md";
const SKILL: &str = "groups/skill/vleo-group-folder/SKILL.md";
const SCHEMA: &str = "groups/schema.sql";
const VENDOR: &str = "web/vendor/sqlite";

pub(super) fn cmd_group_app(root: &Path, args: &[&str]) -> Result<(), String> {
    let check = args.contains(&"--check");
    let spec = spec(root)?;
    let outputs = [
        (OUT, page(root, &spec)?),
        (NODE_OUT, node_page(root, &spec)?),
        (DOC, doc(&spec)),
        (SKILL, skill(&spec)),
    ];
    let mut stale = Vec::new();
    for (path, body) in &outputs {
        let have = fs::read_to_string(root.join(path)).unwrap_or_default();
        if have != *body {
            stale.push(*path);
        }
    }
    if check {
        if stale.is_empty() {
            println!("the group application and the pattern's documents are current with {SPEC}");
            return Ok(());
        }
        return Err(format!(
            "{} out of date with {SPEC} and web/: run `cargo run -p xtask -- group-app` and commit them",
            stale.join(", ")
        ));
    }
    for (path, body) in &outputs {
        let p = root.join(path);
        if let Some(dir) = p.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        fs::write(&p, body).map_err(|e| format!("{path}: {e}"))?;
        println!("wrote {path} ({} bytes)", body.len());
    }
    Ok(())
}

fn spec(root: &Path) -> Result<toml::Value, String> {
    let text = fs::read_to_string(root.join(SPEC)).map_err(|e| format!("{SPEC}: {e}"))?;
    text.parse::<toml::Value>()
        .map_err(|e| format!("{SPEC}: {e}"))
}

/// The group's page: the shell filled with the stylesheet, the pattern, the
/// database engine and the code.
fn page(root: &Path, spec: &toml::Value) -> Result<String, String> {
    let shell = fs::read_to_string(root.join("web/pages/group.html"))
        .map_err(|e| format!("web/pages/group.html: {e}"))?;
    fill(root, spec, &shell, "group.js")
}

/// The node application: the same stylesheet, pattern and engine, its own
/// shell and code.
fn node_page(root: &Path, spec: &toml::Value) -> Result<String, String> {
    let shell = fs::read_to_string(root.join("web/pages/node-app.html"))
        .map_err(|e| format!("web/pages/node-app.html: {e}"))?;
    fill(root, spec, &shell, "napp.js")
}

fn fill(root: &Path, spec: &toml::Value, shell: &str, entry: &str) -> Result<String, String> {
    let app =
        fs::read_to_string(root.join("web/app.css")).map_err(|e| format!("web/app.css: {e}"))?;
    let own = fs::read_to_string(root.join("web/pages/group.css"))
        .map_err(|e| format!("web/pages/group.css: {e}"))?;
    let css = format!("{}\n{}", inline_fonts(root, &app)?, own);
    // Nothing the page carries may end its own script element early.
    let script = readers::bundle(root, entry)?.replace("</script", "<\\/script");
    let json = json(spec).replace("</", "<\\/");
    let (engine, wasm) = sqlite(root)?;
    // The method checker every node form carries, so the pseudocode is read
    // here exactly as the developer's tools read it.
    let method = fs::read(root.join("web/method.wasm.gz"))
        .map_err(|e| format!("web/method.wasm.gz: {e}"))?;
    let schema = fs::read_to_string(root.join(SCHEMA)).map_err(|e| format!("{SCHEMA}: {e}"))?;
    Ok(shell
        .replace("{{CSS}}", &css)
        .replace("{{SPEC}}", &json)
        .replace("{{SQLITE_WASM}}", &quote(&wasm))
        .replace(
            "{{METHOD_WASM}}",
            &quote(&vleo_sheet::template::base64(&method)),
        )
        .replace("{{SCHEMA}}", &quote(&schema).replace("</", "<\\/"))
        .replace("{{SQLITE}}", &engine)
        .replace("{{SCRIPT}}", &script))
}

/// The database engine the page carries: the vendored SQLite module, its one
/// `export` line made into a global (the page has no module loader), and its
/// wasm as base64. Each file is refused unless its hash is the one
/// web/vendor/sqlite/SOURCE.toml records.
fn sqlite(root: &Path) -> Result<(String, String), String> {
    let dir = root.join(VENDOR);
    let source: toml::Value = fs::read_to_string(dir.join("SOURCE.toml"))
        .map_err(|e| format!("{VENDOR}/SOURCE.toml: {e}"))?
        .parse()
        .map_err(|e| format!("{VENDOR}/SOURCE.toml: {e}"))?;
    let read = |name: &str, key: &str| -> Result<Vec<u8>, String> {
        let bytes = fs::read(dir.join(name)).map_err(|e| format!("{VENDOR}/{name}: {e}"))?;
        let want = source.get(key).and_then(|v| v.as_str()).unwrap_or("");
        let have = sha256_hex(&bytes);
        if have != want {
            return Err(format!(
                "{VENDOR}/{name} is not the file SOURCE.toml records ({key} {want}, the file is {have}): \
                 restore it, or follow SOURCE.toml's steps to update it"
            ));
        }
        Ok(bytes)
    };
    let mjs = String::from_utf8(read("sqlite3.mjs", "mjs_sha256")?)
        .map_err(|_| format!("{VENDOR}/sqlite3.mjs is not UTF-8"))?;
    let wasm = read("sqlite3.wasm.gz", "wasm_gz_sha256")?;
    let export = mjs
        .lines()
        .rfind(|l| l.starts_with("export {"))
        .ok_or_else(|| format!("{VENDOR}/sqlite3.mjs has no export line"))?;
    if !export.contains("sqlite3InitModule as default") {
        return Err(format!(
            "{VENDOR}/sqlite3.mjs no longer exports sqlite3InitModule"
        ));
    }
    let engine = mjs
        .replace(export, "window.sqlite3InitModule = sqlite3InitModule;")
        .replace("</script", "<\\/script");
    Ok((engine, vleo_sheet::template::base64(&wasm)))
}

/// SHA-256 (FIPS 180-4), for the vendored files' hashes. Small, and here so
/// the build needs no crate for one check.
pub(super) fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for block in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, c) in block.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for (a, b) in h.iter_mut().zip(v) {
            *a = a.wrapping_add(b);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

/// Every `url(fonts/…)` in the stylesheet, as the font itself.
fn inline_fonts(root: &Path, css: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = css;
    while let Some(at) = rest.find("url(fonts/") {
        out.push_str(&rest[..at]);
        let tail = &rest[at + 4..];
        let end = tail.find(')').ok_or("web/app.css: a url( with no )")?;
        let rel = &tail[..end];
        let bytes = fs::read(root.join("web").join(rel)).map_err(|e| format!("web/{rel}: {e}"))?;
        out.push_str(&format!(
            "url(data:font/woff2;base64,{})",
            vleo_sheet::template::base64(&bytes)
        ));
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// TOML as JSON: the subset the pattern uses, written out by hand as the rest
/// of this repository writes JSON.
fn json(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => quote(s),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => {
            if f.is_finite() {
                f.to_string()
            } else {
                "null".into()
            }
        }
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Datetime(d) => quote(&d.to_string()),
        toml::Value::Array(a) => format!("[{}]", a.iter().map(json).collect::<Vec<_>>().join(",")),
        toml::Value::Table(t) => format!(
            "{{{}}}",
            t.iter()
                .map(|(k, v)| format!("{}:{}", quote(k), json(v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn quote(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

// ── the two documents ──────────────────────────────────────────────────────

fn s<'a>(v: &'a toml::Value, k: &str) -> &'a str {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("").trim()
}

fn arr<'a>(v: &'a toml::Value, k: &str) -> &'a [toml::Value] {
    v.get(k)
        .and_then(|x| x.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[])
}

/// One line per file of the pattern, at a level, for both documents.
fn files_md(spec: &toml::Value, level: &str) -> String {
    let mut o = String::new();
    for f in arr(spec, "file") {
        let l = s(f, "level");
        if !(l == level || (level == "group" && l == "both")) {
            continue;
        }
        let path = if level == "node" {
            format!("nodes/<id>/{}", s(f, "path"))
        } else {
            s(f, "path").to_string()
        };
        let req = if f.get("required").and_then(|x| x.as_bool()) == Some(true) {
            match f.get("kinds").and_then(|x| x.as_array()) {
                Some(k) => format!(
                    "**required** for {} nodes",
                    k.iter()
                        .filter_map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                None => "**required**".to_string(),
            }
        } else if f.get("written_by_app").and_then(|x| x.as_bool()) == Some(true) {
            "written by the group application".to_string()
        } else {
            "optional".to_string()
        };
        o.push_str(&format!(
            "- `{path}` — {req}. {}\n",
            s(f, "says").replace('\n', " ")
        ));
        let cols = arr(f, "columns");
        if !cols.is_empty() {
            for c in cols {
                let opt = if c.get("optional").and_then(|x| x.as_bool()) == Some(true) {
                    " (optional)"
                } else {
                    ""
                };
                let one = arr(c, "one_of")
                    .iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>();
                let one = if one.is_empty() {
                    String::new()
                } else {
                    format!(" One of: {}.", one.join(", "))
                };
                o.push_str(&format!(
                    "  - `{}`{opt} — {}.{one}\n",
                    s(c, "name"),
                    s(c, "says")
                ));
            }
        }
    }
    o
}

fn texts_md(spec: &toml::Value) -> String {
    let mut o = String::new();
    for t in arr(spec, "text") {
        let heads = arr(t, "headings")
            .iter()
            .filter_map(|x| x.as_str())
            .map(|h| format!("`## {h}`"))
            .collect::<Vec<_>>()
            .join(", ");
        o.push_str(&format!(
            "### `{}` — the {}\n\n{}\n\nIts sections, in this order: {heads}.\n\n",
            s(t, "file"),
            s(t, "role"),
            s(t, "says").replace('\n', " ")
        ));
    }
    o
}

fn embeds_md(spec: &toml::Value) -> String {
    arr(spec, "embed")
        .iter()
        .map(|e| format!("- `{}` — {}\n", s(e, "form"), s(e, "says")))
        .collect()
}

fn conventions_md(spec: &toml::Value) -> String {
    let mut o = String::new();
    if let Some(t) = spec.get("conventions").and_then(|x| x.as_table()) {
        for (k, v) in t {
            o.push_str(&format!(
                "- **{k}.** {}\n",
                v.as_str().unwrap_or("").trim().replace('\n', " ")
            ));
        }
    }
    o
}

fn version(spec: &toml::Value) -> i64 {
    spec.get("version")
        .and_then(|x| x.as_integer())
        .unwrap_or(0)
}

fn doc(spec: &toml::Value) -> String {
    format!(
        "<!-- GENERATED from groups/SPEC.toml by `cargo run -p xtask -- group-app`. Do not edit. -->\n\
# The group folder, version {v}\n\n\
> **Answer first.** A group keeps everything it knows in one folder in this fixed pattern: tables as \
CSV, words as Markdown, the algorithm as pseudocode, and its own test results. The group application \
(`web/group.html`) opens the folder offline, shows it the way the VLEO application will, checks it \
against this pattern, records each member's sign-off and seals it for the developer.\n>\n\
> **Kind:** reference · **For:** group members, and any assistant that helps them\n\n\
## Said simply\n\n\
The folder is the design. Nobody types the design anywhere else: the developer compiles this folder \
into the database the application reads, and generates the engine from the pseudocode in it. So \
what the folder says is what everyone gets — and the group application lets a group see exactly that \
before anything is built.\n\n\
## The conventions\n\n{conv}\n\
## The two kinds of text\n\n{texts}\
## What a text can hold\n\n{embeds}\n\
## The group's files\n\n{group}\n\
## Each node's files\n\n{node}\n\
## Where the simple version breaks\n\n\
- **Nothing in the group application computes.** The pseudocode is shown, and shown as equations, \
never run. Whether it gives the group's results is decided by the developer's engine, after the seal.\n\
- **The checks are of the folder, not of the physics.** A folder can pass every check and still be \
wrong; the results and the evidence are what test it, and the developer's tests use both.\n\
- **A browser that cannot write into a folder downloads instead.** Sign-offs, packages and issues are \
then saved as downloads named for where they belong, and the member puts them in the folder.\n",
        v = version(spec),
        conv = conventions_md(spec),
        texts = texts_md(spec),
        embeds = embeds_md(spec),
        group = files_md(spec, "group"),
        node = files_md(spec, "node"),
    )
}

fn skill(spec: &toml::Value) -> String {
    format!(
        "---\n\
name: vleo-group-folder\n\
description: Help a member of a VLEO group fill their group folder (pattern version {v}) — turn their notes, \
code, photos of equations and spreadsheets into the folder's CSV, Markdown and pseudocode files, then check it. \
Use when someone asks to add or change a node, write an explanation or theory, record results, or fix what the \
group application's checks say.\n\
---\n\
<!-- GENERATED from groups/SPEC.toml by `cargo run -p xtask -- group-app`. Do not edit. -->\n\n\
# Filling a VLEO group folder\n\n\
You help a member of a group keep their **group folder**: the one place their part of the spacecraft \
design lives. The developer builds the application from it, so what you write is what everyone gets.\n\n\
## The rules you never bend\n\n\
1. **Never invent a number, an equation or a source.** Every value, relation and citation comes from \
the member's own material — their notes, their code, a paper they name. If something is missing, ask; \
leave it out rather than guess. A results table is the reference the developer's code is tested against, \
so a made-up row is a wrong test.\n\
2. **Results come from outside the developer's code.** `results/isolation.csv` is filled from the member's \
own code, a hand calculation, a spreadsheet or a paper, and its `origin` column says which. When it came from \
code, write `results/how-run.md`: language, tool and version, machine, date, command. You may run the \
member's code to produce the table only when they ask you to and the code is theirs.\n\
3. **Pseudocode states what the member's code or notes state.** You may transcribe it from their code; \
say so in a `#` comment on its first line (`# transcribed from code/f107.m by an assistant`). Every number \
carries its unit, every path ends in `return` or `refuse` (docs/PSEUDOCODE.md).\n\
4. **Write data, never HTML.** A picture that is data is a CSV plus a row of figures.csv; an animation is \
a `steps` table walking a `flow` table. The group application draws them. Images are only for pictures that \
are not data.\n\
5. **Explanation and theory never repeat each other.** The explanation is for understanding — answer first, \
the simple version, a picture, a question to guess, where it breaks, the common misreading — and holds no \
derivations. The theory is only the mathematics. Point from one to the other with `{{{{eq id}}}}`.\n\
6. **Finish by checking.** Open the folder in the group application (`web/group.html`) or ask the member to, \
and fix every error it lists before you hand back. Signing and sealing are the members' — never sign for anyone.\n\n\
## The conventions\n\n{conv}\n\
## The two kinds of text\n\n{texts}\
## What a text can hold\n\n{embeds}\n\
## Writing equations\n\n\
Store each equation once, in LaTeX, in `equations.csv`. When the member gives you an equation as a photo, \
a screenshot, Word's equation or plain words, write the LaTeX for it and read it back to them in words so \
they can confirm it. Plain shorthand (`v = sqrt(mu/r)`) can be turned into LaTeX with the application's \
equation helper.\n\n\
## The group's files\n\n{group}\n\
## Each node's files\n\n{node}\n\
## Adding a node, step by step\n\n\
1. Add its row to `nodes.csv`: id, question (ending in ?), kind, output, unit, range.\n\
2. Make `nodes/<id>/` with `explanation.md` and `theory.md`, each with its sections in order.\n\
3. For a computed node: `inputs.csv` (where each input comes from, its default and range), \
`pseudocode.txt`, and `results/isolation.csv` with the defaults row, three ordinary cases, both ends of every \
range and one refusal.\n\
4. Add a line to `flow.txt`: `<id> <- <input>, <input>`.\n\
5. Name the author in `members.csv`, and add a row to `versions.csv` when the version changes.\n\
6. Check it in the group application.\n",
        v = version(spec),
        conv = conventions_md(spec),
        texts = texts_md(spec),
        embeds = embeds_md(spec),
        group = files_md(spec, "group"),
        node = files_md(spec, "node"),
    )
}

// ── a group's folder, written from the tree as it stands ───────────────────

/// `group-export <group> [--out <dir>]`: the group's folder in the pattern,
/// filled from every sheet in the group, so a group starts from what the
/// design already holds rather than from nothing.
///
/// IT INVENTS NOTHING. What the tree has is written where the pattern puts
/// it; what it does not have — a pseudocode, the group's own results, a
/// picture, a guess-first question — is left empty, so the group application
/// lists exactly what the group still has to supply. Most of the tree's plain
/// words were transcribed by an assistant, and the folder's first version
/// says so, for the owner to confirm or replace.
pub(super) fn cmd_group_export(root: &Path, args: &[&str]) -> Result<(), String> {
    let id = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .ok_or("usage: group-export <group> [--out <dir>]")?
        .to_string();
    let out = args
        .iter()
        .position(|a| *a == "--out")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/groups").join(&id));
    let tree = load_all(root).map_err(|e| e.to_string())?;
    let files = export(root, &tree, &id)?;
    for (path, body) in &files {
        let p = out.join(path);
        if let Some(d) = p.parent() {
            fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
        fs::write(&p, body).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    println!(
        "wrote {} files for {id} into {}\nopen web/group.html and choose that folder to see what it still needs",
        files.len(),
        out.display()
    );
    Ok(())
}

fn csv_row(cells: &[String]) -> String {
    cells
        .iter()
        .map(|c| {
            if c.contains(',') || c.contains('"') || c.contains('\n') {
                format!("\"{}\"", c.replace('"', "\"\""))
            } else {
                c.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
        + "\n"
}

fn csv_table(head: &[&str], rows: &[Vec<String>]) -> String {
    let mut o = csv_row(&head.iter().map(|h| h.to_string()).collect::<Vec<_>>());
    for r in rows {
        o.push_str(&csv_row(r));
    }
    o
}

fn num_cell(v: f64) -> String {
    if v.is_finite() {
        format!("{v}")
    } else {
        String::new()
    }
}

fn first_sentence(s: &str) -> String {
    let t = s.trim().replace('\n', " ");
    match t.find(". ") {
        Some(i) => t[..=i].to_string(),
        None => t,
    }
}

/// A sentence that ends as one, so the next one does not run into it.
fn sentence(s: &str) -> String {
    let t = s.trim();
    if t.is_empty() || t.ends_with(['.', '?', '!', ':']) {
        t.to_string()
    } else {
        format!("{t}.")
    }
}

fn para(s: &str) -> String {
    s.trim()
        .split("\n\n")
        .map(|p| p.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Every file of the group's folder, as (path, contents).
fn export(root: &Path, tree: &Tree, id: &str) -> Result<Vec<(String, String)>, String> {
    let group = tree
        .groups
        .get(id)
        .ok_or_else(|| format!("there is no group {id} in layers/"))?;
    // A deprecated row is kept in the tree for its history, and nothing live
    // reads it: it is not part of the group's design, so not of its folder.
    let sheets: Vec<&vleo_sheet::model::Sheet> = tree
        .sheets
        .values()
        .filter(|s| s.parent == id && s.state != "deprecated")
        .collect();
    if sheets.is_empty() {
        return Err(format!("the group {id} has no nodes"));
    }
    let mine: BTreeSet<&str> = sheets.iter().map(|s| s.id.as_str()).collect();
    let mut files: Vec<(String, String)> = Vec::new();
    let date = std::process::Command::new("git")
        .args(["log", "-1", "--format=%cs"])
        .current_dir(root)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let kind = |s: &vleo_sheet::model::Sheet| match s.kind.as_str() {
        "declared" | "required" | "achieved" => s.kind.clone(),
        _ => "computed".to_string(),
    };

    // The group's own tables.
    let crossing = sheets.iter().find(|s| !s.crosses_to.is_empty());
    let summary = crossing
        .map(|s| first_sentence(&s.explain.simply))
        .filter(|s| !s.is_empty())
        .unwrap_or_default();
    files.push((
        "group.csv".into(),
        csv_table(
            &["id", "name", "owner", "version", "summary"],
            &[vec![
                id.into(),
                group.label.clone(),
                format!("{} team — name the owner", group.owner),
                "0.1".into(),
                summary,
            ]],
        ),
    ));
    files.push((
        "members.csv".into(),
        csv_table(
            &["name", "role", "nodes"],
            &[vec![
                format!("{} team — name the owner", group.owner),
                "owner".into(),
                "*".into(),
            ]],
        ),
    ));
    files.push((
        "nodes.csv".into(),
        csv_table(
            &[
                "id", "question", "kind", "output", "unit", "lower", "upper", "value",
            ],
            &sheets
                .iter()
                .map(|s| {
                    vec![
                        s.id.clone(),
                        s.question.trim().replace('\n', " "),
                        kind(s),
                        s.symbol.clone(),
                        unit_symbol(&s.unit),
                        num_cell(s.lower),
                        num_cell(s.upper),
                        s.value.map(num_cell).unwrap_or_default(),
                    ]
                })
                .collect::<Vec<_>>(),
        ),
    ));
    let mut flow =
        String::from("# The group's flow, from every node's inputs as the tree holds them.\n");
    for s in &sheets {
        if !s.inputs.is_empty() {
            let from: Vec<String> = s
                .inputs
                .iter()
                .map(|i| from_of(tree, &mine, &i.var))
                .collect();
            flow.push_str(&format!("{} <- {}\n", s.id, from.join(", ")));
        }
    }
    files.push(("flow.txt".into(), flow));
    let crossings: Vec<Vec<String>> = sheets
        .iter()
        .filter(|s| !s.crosses_to.is_empty())
        .map(|s| {
            vec![
                s.id.clone(),
                s.crosses_to.clone(),
                first_sentence(&s.explain.simply),
            ]
        })
        .collect();
    if !crossings.is_empty() {
        files.push((
            "publishes.csv".into(),
            csv_table(&["node", "to", "says"], &crossings),
        ));
    }
    let reqs: Vec<Vec<String>> = sheets
        .iter()
        .filter(|s| s.kind == "required")
        .map(|s| {
            let ach = s.id.replace("_req_", "_ach_");
            vec![
                s.id.clone(),
                s.id.clone(),
                if mine.contains(ach.as_str()) {
                    ach
                } else {
                    String::new()
                },
                s.sense.clone(),
                s.question.trim().replace('\n', " "),
            ]
        })
        .collect();
    if !reqs.is_empty() {
        files.push((
            "requirements.csv".into(),
            csv_table(&["id", "required", "achieved", "sense", "says"], &reqs),
        ));
    }

    // Sources: every one a node of the group cites.
    let mut cited = BTreeSet::new();
    for s in &sheets {
        if !s.source.is_empty() {
            cited.insert(s.source.clone());
        }
        for v in &s.versions {
            if !v.source.is_empty() {
                cited.insert(v.source.clone());
            }
        }
    }
    // Every node's fixtures: the values from outside this code its answer is
    // checked against. They are the node's isolation results — what the
    // developer's code must give — and those taken straight from a published
    // source or another tool are its evidence too.
    let mut fixtures: BTreeMap<String, Vec<Fx>> = BTreeMap::new();
    for s in &sheets {
        let path = root.join(&s.dir).join("fixtures.toml");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(v) = text.parse::<toml::Value>() else {
            continue;
        };
        let num = |x: &toml::Value| x.as_float().or_else(|| x.as_integer().map(|i| i as f64));
        let text_of =
            |f: &toml::Value, k: &str| f.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
        for f in v
            .get("fixture")
            .and_then(|x| x.as_array())
            .into_iter()
            .flatten()
        {
            let fx = Fx {
                inputs: f
                    .get("inputs")
                    .and_then(|x| x.as_table())
                    .map(|t| {
                        t.iter()
                            .filter_map(|(k, v)| num(v).map(|x| (k.clone(), x)))
                            .collect()
                    })
                    .unwrap_or_default(),
                expect: f.get("expect").and_then(num),
                tolerance: f.get("tolerance").and_then(num).unwrap_or(0.0),
                provenance: text_of(f, "provenance"),
                source: text_of(f, "source"),
                label: text_of(f, "label"),
                variable: text_of(f, "variable"),
            };
            if !fx.source.is_empty() {
                cited.insert(fx.source.clone());
            }
            fixtures.entry(s.id.clone()).or_default().push(fx);
        }
    }
    files.push((
        "sources.csv".into(),
        csv_table(
            &["id", "cite", "file", "url", "licence"],
            &cited
                .iter()
                .map(|c| {
                    let s = tree.sources.get(c);
                    vec![
                        c.clone(),
                        s.map(|s| format!("{} — {}", s.title, s.where_))
                            .unwrap_or_default(),
                        String::new(),
                        String::new(),
                        String::new(),
                    ]
                })
                .collect::<Vec<_>>(),
        ),
    ));
    files.push((
        "versions.csv".into(),
        csv_table(
            &["version", "date", "by", "believed", "tested", "learned", "changed", "risks"],
            &[vec![
                "0.1".into(),
                date,
                "xtask group-export".into(),
                "What the tree held for this group, as it stood.".into(),
                "Nothing yet: this folder is a transcription, not a review.".into(),
                "Most plain words and first versions in the tree were written by an assistant from the sheets; the owner confirms or replaces them before signing.".into(),
                "The folder was written from the tree. Pseudocode, the group's own results and its pictures are still to supply.".into(),
                String::new(),
            ]],
        ),
    ));
    let related: Vec<String> = tree
        .relations
        .iter()
        .filter(|r| r.from == id)
        .map(|r| {
            format!(
                "- It feeds **{}**: {}.",
                r.to,
                r.why.trim().trim_end_matches('.')
            )
        })
        .collect();
    files.push((
        "explanation.md".into(),
        format!(
            "## In one line\n\n{}\n\n## Said simply\n\n{}\n\n## Picture it\n\n## Guess first\n\n## Where it breaks\n\n## Common misreading\n",
            crossing.map(|s| first_sentence(&s.explain.simply)).unwrap_or_default(),
            related.join("\n")
        ),
    ));
    files.push((
        "theory.md".into(),
        "## Equations\n\n## Derivation\n\n## Assumptions\n\n## Validity\n".into(),
    ));

    // Each node's folder.
    for s in &sheets {
        let dir = format!("nodes/{}/", s.id);
        files.push((
            dir.clone() + "explanation.md",
            format!(
                "## In one line\n\n{}\n\n## Said simply\n\n{}\n\n## Picture it\n\n## Guess first\n\n## Where it breaks\n\n{}\n\n## Common misreading\n\n{}\n",
                first_sentence(&s.explain.simply),
                para(&s.explain.simply),
                para(&s.explain.breaks),
                para(&s.explain.wrong),
            ),
        ));
        let mut derivation = String::new();
        if !s.theory.why.trim().is_empty() {
            derivation.push_str(&para(&s.theory.why));
            derivation.push_str("\n\n");
        }
        for (i, st) in s.theory.steps.iter().enumerate() {
            // The step's maths on its own line of the same item, so the numbering runs on.
            let math = st.math.trim();
            derivation.push_str(&format!(
                "{}. {}{}\n",
                i + 1,
                para(&st.text).replace("\n\n", " "),
                if math.is_empty() {
                    String::new()
                } else {
                    format!(" `{}`", math.replace('`', "'"))
                }
            ));
        }
        let assumptions: String = s
            .assumptions
            .iter()
            .map(|a| {
                let w = para(&a.fails_when);
                format!(
                    "- {}{}\n",
                    sentence(&para(&a.text).replace("\n\n", " ")),
                    if w.is_empty() {
                        String::new()
                    } else {
                        format!(" Fails when: {}", w.replace("\n\n", " "))
                    }
                )
            })
            .collect();
        let mut validity = Vec::new();
        if s.lower.is_finite() || s.upper.is_finite() {
            validity.push(format!(
                "From {} to {} {}.{}{}",
                num_cell(s.lower),
                num_cell(s.upper),
                s.unit,
                if s.reason_lower.trim().is_empty() {
                    String::new()
                } else {
                    format!(
                        " Below: {}",
                        sentence(&para(&s.reason_lower).replace("\n\n", " "))
                    )
                },
                if s.reason_upper.trim().is_empty() {
                    String::new()
                } else {
                    format!(
                        " Above: {}",
                        sentence(&para(&s.reason_upper).replace("\n\n", " "))
                    )
                },
            ));
        }
        if !s.theory.reading.trim().is_empty() {
            validity.push(para(&s.theory.reading));
        }
        let equations = if s.expression.trim().is_empty() {
            String::new()
        } else {
            format!("```\n{}\n```\n", s.expression.trim())
        };
        files.push((
            dir.clone() + "theory.md",
            format!(
                "## Equations\n\n{equations}\n## Derivation\n\n{derivation}\n## Assumptions\n\n{assumptions}\n## Validity\n\n{}\n",
                validity.join("\n\n")
            ),
        ));
        if kind(s) == "computed" {
            let code = if !s.method.text.trim().is_empty() {
                s.method.text.clone()
            } else if !s.steps.is_empty() {
                let mut c = String::from("# The algorithm as the tree describes it, step by step. Write each step as\n# method-language lines (docs/PSEUDOCODE.md), ending in return or refuse.\n");
                for st in &s.steps {
                    c.push_str(&format!(
                        "# step {}: {}\n",
                        st.number,
                        st.text.split_whitespace().collect::<Vec<_>>().join(" ")
                    ));
                }
                c
            } else {
                String::new()
            };
            if !code.is_empty() {
                files.push((dir.clone() + "pseudocode.txt", code));
            }
            if !s.inputs.is_empty() {
                files.push((
                    dir.clone() + "inputs.csv",
                    csv_table(
                        &["name", "from", "unit", "default", "min", "max", "says"],
                        &s.inputs
                            .iter()
                            .map(|i| {
                                let base = i.var.split('.').next().unwrap_or("");
                                let up = tree.sheets.get(base);
                                // The default is the value the node's first
                                // fixture inside every input's range uses
                                // (else its first), else the source's own value.
                                let unit = up.map(|u| u.unit.as_str()).unwrap_or("");
                                let inside = |f: &&Fx| {
                                    s.inputs.iter().all(|j| {
                                        let b =
                                            tree.sheets.get(j.var.split('.').next().unwrap_or(""));
                                        f.inputs.iter().find(|(k, _)| *k == j.binding).is_some_and(
                                            |(_, v)| {
                                                let v = shown(
                                                    *v,
                                                    b.map(|b| b.unit.as_str()).unwrap_or(""),
                                                );
                                                b.is_none_or(|b| v >= b.lower && v <= b.upper)
                                            },
                                        )
                                    })
                                };
                                let first = fixtures
                                    .get(&s.id)
                                    .and_then(|f| f.iter().find(inside).or(f.first()))
                                    .and_then(|f| f.inputs.iter().find(|(k, _)| *k == i.binding))
                                    .map(|(_, v)| num_cell(shown(*v, unit)));
                                vec![
                                    i.binding.clone(),
                                    from_of(tree, &mine, &i.var),
                                    up.map(|u| unit_symbol(&u.unit)).unwrap_or_default(),
                                    // The first fixture's value first, so the
                                    // results hold a row at the defaults.
                                    first
                                        .or_else(|| up.and_then(|u| u.value).map(num_cell))
                                        .unwrap_or_default(),
                                    up.map(|u| num_cell(u.lower)).unwrap_or_default(),
                                    up.map(|u| num_cell(u.upper)).unwrap_or_default(),
                                    if i.var.contains('.') {
                                        format!("the member {}", i.var)
                                    } else {
                                        String::new()
                                    },
                                ]
                            })
                            .collect::<Vec<_>>(),
                    ),
                ));
            }
        }
        // Fixtures for this node's own answer; one naming another output
        // belongs to a node with several, which the folder holds as one.
        let own: Vec<&Fx> = fixtures
            .get(&s.id)
            .into_iter()
            .flatten()
            .filter(|f| f.variable.is_empty() || f.variable == s.symbol)
            .collect();
        let evidence: Vec<Vec<String>> = own
            .iter()
            .filter(|f| {
                matches!(
                    f.provenance.as_str(),
                    "published-source" | "other-tool" | "physical-bound"
                )
            })
            .map(|f| {
                vec![
                    f.inputs
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(" "),
                    f.expect
                        .map(|x| shown(x, &s.unit).to_string())
                        .unwrap_or_default(),
                    unit_symbol(&s.unit),
                    f.source.clone(),
                    sentence(&f.label),
                ]
            })
            .collect();
        if !evidence.is_empty() {
            files.push((
                dir.clone() + "evidence.csv",
                csv_table(&["inputs", "value", "unit", "source", "says"], &evidence),
            ));
        }
        // The unit an input's source declares, by its machine name.
        let unit_name = |i: &vleo_sheet::model::Input| {
            tree.sheets
                .get(i.var.split('.').next().unwrap_or(""))
                .map(|u| u.unit.clone())
                .unwrap_or_default()
        };
        if kind(s) == "computed" && !own.is_empty() {
            let unit_of = |b: &str| {
                s.inputs
                    .iter()
                    .find(|i| i.binding == b)
                    .and_then(|i| tree.sheets.get(i.var.split('.').next().unwrap_or("")))
                    .map(|u| unit_symbol(&u.unit))
                    .unwrap_or_default()
            };
            let mut head: Vec<String> = s
                .inputs
                .iter()
                .map(|i| with_unit(&i.binding, &unit_of(&i.binding)))
                .collect();
            head.push(with_unit("answer", &unit_symbol(&s.unit)));
            for h in ["tolerance", "refuses", "origin", "says"] {
                head.push(h.into());
            }
            let rows: Vec<Vec<String>> = own
                .iter()
                .map(|f| {
                    let mut r: Vec<String> = s
                        .inputs
                        .iter()
                        .map(|i| {
                            f.inputs
                                .iter()
                                .find(|(k, _)| *k == i.binding)
                                .map(|(_, v)| shown(*v, &unit_name(i)).to_string())
                                .unwrap_or_default()
                        })
                        .collect();
                    r.push(
                        f.expect
                            .map(|x| shown(x, &s.unit).to_string())
                            .unwrap_or_default(),
                    );
                    r.push(f.tolerance.to_string());
                    r.push("no".into());
                    r.push(origin_of(&f.provenance).into());
                    r.push(sentence(&f.label));
                    r
                })
                .collect();
            let head: Vec<&str> = head.iter().map(String::as_str).collect();
            files.push((
                dir.clone() + "results/isolation.csv",
                csv_table(&head, &rows),
            ));
            files.push((
                dir.clone() + "results/how-run.md",
                format!(
                    "These are the node's fixtures in the design (`{}/fixtures.toml`): values worked out outside the code that \
                     computes this node — by hand from the cited record, or read from a published source — each with the \
                     tolerance the design holds it to. `origin` says which. They are the reference the developer's code is tested against.\n",
                    s.dir.display()
                ),
            ));
        }
    }
    Ok(files)
}

/// One fixture of a node, as the folder needs it.
struct Fx {
    inputs: Vec<(String, f64)>,
    expect: Option<f64>,
    tolerance: f64,
    provenance: String,
    source: String,
    label: String,
    variable: String,
}

/// A unit as a reader writes it: the tree's machine name (`Day`) as its
/// symbol (`d`), and a pure number as `1`.
fn unit_symbol(name: &str) -> String {
    match vleo_units::Unit::from_name(name).map(|u| u.symbol()) {
        Some("-") => "1".into(),
        Some(sym) => sym.into(),
        None => name.into(),
    }
}

/// A value the tree holds in SI, in the unit a reader is shown (`Day` → days).
fn shown(si: f64, unit: &str) -> f64 {
    vleo_units::Unit::from_name(unit)
        .map(|u| si / u.si_factor())
        .unwrap_or(si)
}

fn with_unit(name: &str, unit: &str) -> String {
    if unit.is_empty() {
        name.into()
    } else {
        format!("{name} [{unit}]")
    }
}

/// Where a fixture's value came from, in the pattern's words.
fn origin_of(provenance: &str) -> &'static str {
    match provenance {
        "published-source" => "paper",
        "other-tool" => "code",
        _ => "hand",
    }
}

/// Where an input comes from, in the folder's terms: a node of this group by
/// its id, a node of another group as `group.node`.
fn from_of(tree: &Tree, mine: &BTreeSet<&str>, var: &str) -> String {
    let base = var.split('.').next().unwrap_or(var);
    if mine.contains(base) {
        return base.to_string();
    }
    match tree.sheets.get(base) {
        Some(s) => format!("{}.{}", s.parent, base),
        None => var.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_becomes_json_with_every_kind_of_value() {
        let v: toml::Value = "a = \"x\\\"y\"\nb = 2\nc = [true, 1.5]\n[d]\ne = \"line\\nbreak\""
            .parse()
            .unwrap();
        assert_eq!(
            json(&v),
            "{\"a\":\"x\\\"y\",\"b\":2,\"c\":[true,1.5],\"d\":{\"e\":\"line\\nbreak\"}}"
        );
    }

    #[test]
    fn sha256_matches_the_standard_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        // Two blocks: the padding crosses a block boundary.
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn the_committed_group_application_is_built_from_its_sources() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let spec = spec(root).unwrap();
        let built = page(root, &spec).unwrap();
        assert!(
            !built.contains("{{CSS}}")
                && !built.contains("{{SCRIPT}}")
                && !built.contains("{{SPEC}}")
                && !built.contains("{{SQLITE}}")
                && !built.contains("{{SQLITE_WASM}}")
                && !built.contains("{{SCHEMA}}")
                && !built.contains("{{METHOD_WASM}}")
        );
        assert!(
            !built.contains("\nexport {"),
            "the engine's export line is replaced"
        );
        assert!(!built.contains("url(fonts/"), "every font is inlined");
        // Compared without printing them: either side is the whole page.
        let node = node_page(root, &spec).unwrap();
        assert!(!node.contains("{{SCRIPT}}") && !node.contains("{{SQLITE}}"));
        for (path, want) in [
            (OUT, built),
            (NODE_OUT, node),
            (DOC, doc(&spec)),
            (SKILL, skill(&spec)),
        ] {
            assert!(
                fs::read_to_string(root.join(path)).unwrap_or_default() == want,
                "{path} is stale: run `cargo run -p xtask -- group-app` and commit it"
            );
        }
    }
}
